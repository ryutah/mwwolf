use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::str::FromStr;
use std::sync::Arc;

use async_std::sync::Mutex;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::{IntoRequest, Request};

use crate::datastore::proto_api::api;
use crate::datastore::proto_api::api::datastore_client::DatastoreClient;
use crate::datastore::proto_api::api::value::ValueType;
use crate::datastore::proto_api::authorize::{ApplicationCredentials, TokenManager, TLS_CERTS};
use crate::datastore::proto_api::{
    Entity, Error, Filter, FromValue, IntoEntity, Key, KeyID, Order, Query, Value,
};
use api::read_options::{ConsistencyType, ReadConsistency};
#[derive(Clone)]
pub struct Client {
    pub(crate) project_name: String,
    pub(crate) service: DatastoreClient<Channel>,
    pub(crate) token_manager: Arc<Mutex<TokenManager>>,
}

impl Client {
    pub(crate) const DOMAIN_NAME: &'static str = "datastore.googleapis.com";
    pub(crate) const ENDPOINT: &'static str = "https://datastore.googleapis.com";
    pub(crate) const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/datastore",
    ];

    pub(crate) async fn construct_request<T: IntoRequest<T>>(
        &mut self,
        request: T,
    ) -> Result<Request<T>, Error> {
        let mut request = request.into_request();
        let token = self.token_manager.lock().await.token().await?;
        let metadata = request.metadata_mut();
        metadata.insert("authorization", token.parse().unwrap());
        Ok(request)
    }

    pub async fn new(project_name: impl Into<String>) -> Result<Client, Error> {
        let path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
        let file = File::open(path)?;
        let creds = json::from_reader(file)?;

        Client::from_credentials(project_name, creds).await
    }

    pub async fn from_credentials(
        project_name: impl Into<String>,
        creds: ApplicationCredentials,
    ) -> Result<Client, Error> {
        let channel = if let Ok(host) = std::env::var("DATASTORE_EMULATOR_HOST") {
            let url = format!("http://{}", host);
            Channel::builder(http::Uri::from_str(&url).unwrap())
                .connect()
                .await?
        } else {
            let tls_config = ClientTlsConfig::new()
                .ca_certificate(Certificate::from_pem(TLS_CERTS))
                .domain_name(Client::DOMAIN_NAME);
            Channel::from_static(Client::ENDPOINT)
                .tls_config(tls_config)?
                .connect()
                .await?
        };

        Ok(Client {
            project_name: project_name.into(),
            service: DatastoreClient::new(channel),
            token_manager: Arc::new(Mutex::new(TokenManager::new(
                creds,
                Client::SCOPES.as_ref(),
            ))),
        })
    }

    pub async fn get<T, K>(
        &mut self,
        key: K,
        transaction: Option<prost::alloc::vec::Vec<u8>>,
    ) -> Result<Option<T>, Error>
    where
        K: Borrow<Key>,
        T: FromValue,
    {
        let results = self.get_all(Some(key.borrow()), transaction).await?;
        Ok(results.into_iter().next().map(T::from_value).transpose()?)
    }

    pub async fn get_all<T, K, I>(
        &mut self,
        keys: I,
        transaction: Option<prost::alloc::vec::Vec<u8>>,
    ) -> Result<Vec<T>, Error>
    where
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
        T: FromValue,
    {
        let og_keys: Vec<K> = keys.into_iter().collect();
        let mut keys: Vec<_> = og_keys
            .iter()
            .map(|key| convert_key(self.project_name.as_str(), key.borrow()))
            .collect();
        let mut found = HashMap::new();

        while !keys.is_empty() {
            let request = api::LookupRequest {
                keys,
                project_id: self.project_name.clone(),
                read_options: transaction.as_ref().map(|transaction| api::ReadOptions {
                    consistency_type: Some(ConsistencyType::Transaction(transaction.to_vec())),
                }),
            };
            let request = self.construct_request(request).await?;
            let response = self.service.lookup(request).await?;
            let response = response.into_inner();

            found.extend(
                response
                    .found
                    .into_iter()
                    .map(|val| val.entity.unwrap())
                    .map(Entity::from)
                    .map(|entity| (entity.key, entity.properties)),
            );
            // let missing = response.missing;
            keys = response.deferred;
        }

        let values: Vec<T> = og_keys
            .into_iter()
            .flat_map(|key| found.remove(key.borrow()))
            .map(FromValue::from_value)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(values)
    }

    pub async fn put(&mut self, entity: impl IntoEntity) -> Result<Option<Key>, Error> {
        let result = self.put_all(Some(entity)).await?;
        Ok(result.into_iter().next().flatten())
    }

    pub async fn put_all<T, I>(&mut self, entities: I) -> Result<Vec<Option<Key>>, Error>
    where
        I: IntoIterator<Item = T>,
        T: IntoEntity,
    {
        let mutations = generate_mutations(&self.project_name, entities)?;

        let request = api::CommitRequest {
            mutations,
            mode: api::commit_request::Mode::NonTransactional as i32,
            transaction_selector: None,
            project_id: self.project_name.clone(),
        };
        let request = self.construct_request(request).await?;
        let response = self.service.commit(request).await?;
        let response = response.into_inner();
        let keys = response
            .mutation_results
            .into_iter()
            .map(|result| result.key.map(Key::from))
            .collect();

        Ok(keys)
    }

    pub async fn delete(&mut self, key: impl Borrow<Key>) -> Result<(), Error> {
        self.delete_all(Some(key.borrow())).await
    }

    pub async fn delete_all<T, I>(&mut self, keys: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = T>,
        T: Borrow<Key>,
    {
        let mutations = keys
            .into_iter()
            .map(|key| convert_key(self.project_name.as_str(), key.borrow()))
            .map(|key| api::Mutation {
                operation: Some(api::mutation::Operation::Delete(key)),
                conflict_detection_strategy: None,
            })
            .collect();

        let request = api::CommitRequest {
            mutations,
            mode: api::commit_request::Mode::NonTransactional as i32,
            transaction_selector: None,
            project_id: self.project_name.clone(),
        };
        let request = self.construct_request(request).await?;
        self.service.commit(request).await?;

        Ok(())
    }

    pub async fn query(&mut self, query: Query) -> Result<Vec<Entity>, Error> {
        let mut output = Vec::new();

        let mut cur_query = query.clone();
        let mut cursor = Vec::new();
        loop {
            let projection = cur_query
                .projections
                .into_iter()
                .map(|name| api::Projection {
                    property: Some(api::PropertyReference { name }),
                })
                .collect();
            let filter = convert_filter(self.project_name.as_str(), cur_query.filters);
            let order = cur_query
                .ordering
                .into_iter()
                .map(|order| {
                    use api::property_order::Direction;
                    let (name, direction) = match order {
                        Order::Asc(name) => (name, Direction::Ascending),
                        Order::Desc(name) => (name, Direction::Descending),
                    };
                    api::PropertyOrder {
                        property: Some(api::PropertyReference { name }),
                        direction: direction as i32,
                    }
                })
                .collect();
            let api_query = api::Query {
                kind: vec![api::KindExpression {
                    name: cur_query.kind,
                }],
                projection,
                filter,
                order,
                offset: cur_query.offset,
                limit: cur_query.limit,
                start_cursor: cursor,
                end_cursor: Vec::new(),
                distinct_on: cur_query
                    .distinct_on
                    .into_iter()
                    .map(|name| api::PropertyReference { name })
                    .collect(),
            };
            let request = api::RunQueryRequest {
                partition_id: Some(api::PartitionId {
                    project_id: self.project_name.clone(),
                    namespace_id: cur_query.namespace.unwrap_or_else(String::new),
                }),
                query_type: Some(api::run_query_request::QueryType::Query(api_query)),
                read_options: Some({
                    api::ReadOptions {
                        consistency_type: Some(ConsistencyType::ReadConsistency(
                            if cur_query.eventual {
                                ReadConsistency::Eventual as i32
                            } else {
                                ReadConsistency::Strong as i32
                            },
                        )),
                    }
                }),
                project_id: self.project_name.clone(),
            };
            let request = self.construct_request(request).await?;
            let results = self.service.run_query(request).await?;
            let results = results.into_inner().batch.unwrap();

            output.extend(
                results
                    .entity_results
                    .into_iter()
                    .map(|el| Entity::from(el.entity.unwrap())),
            );

            if results.more_results
                != (api::query_result_batch::MoreResultsType::NotFinished as i32)
            {
                break Ok(output);
            }

            cur_query = query.clone();
            cursor = results.end_cursor;
        }
    }
}

pub(crate) fn convert_key(project_id: &str, key: &Key) -> api::Key {
    api::Key {
        partition_id: Some(api::PartitionId {
            project_id: String::from(project_id),
            namespace_id: key.get_namespace().map(String::from).unwrap_or_default(),
        }),
        path: {
            let mut key = Some(key);
            let mut path = Vec::new();
            while let Some(current) = key {
                path.push(api::key::PathElement {
                    kind: String::from(current.get_kind()),
                    id_type: match current.get_id() {
                        KeyID::Incomplete => None,
                        KeyID::IntID(id) => Some(api::key::path_element::IdType::Id(*id)),
                        KeyID::StringID(id) => {
                            Some(api::key::path_element::IdType::Name(id.clone()))
                        }
                    },
                });
                key = current.get_parent();
            }
            path.reverse();
            path
        },
    }
}

fn convert_entity(project_name: &str, entity: Entity) -> api::Entity {
    let key = convert_key(project_name, &entity.key);
    let properties = match entity.properties {
        Value::EntityValue(properties) => properties,
        _ => panic!("unexpected non-entity datastore value"),
    };
    let properties = properties
        .into_iter()
        .map(|(k, v)| (k, convert_value(project_name, v)))
        .collect();
    api::Entity {
        key: Some(key),
        properties,
    }
}

fn convert_value(project_name: &str, value: Value) -> api::Value {
    let value_type = match value {
        Value::BooleanValue(val) => ValueType::BooleanValue(val),
        Value::IntegerValue(val) => ValueType::IntegerValue(val),
        Value::DoubleValue(val) => ValueType::DoubleValue(val),
        Value::TimestampValue(val) => ValueType::TimestampValue(prost_types::Timestamp {
            seconds: val.timestamp(),
            nanos: val.timestamp_subsec_nanos() as i32,
        }),
        Value::KeyValue(key) => ValueType::KeyValue(convert_key(project_name, &key)),
        Value::StringValue(val) => ValueType::StringValue(val),
        Value::BlobValue(val) => ValueType::BlobValue(val),
        Value::GeoPointValue(latitude, longitude) => ValueType::GeoPointValue(api::LatLng {
            latitude,
            longitude,
        }),
        Value::EntityValue(properties) => ValueType::EntityValue({
            api::Entity {
                key: None,
                properties: properties
                    .into_iter()
                    .map(|(k, v)| (k, convert_value(project_name, v)))
                    .collect(),
            }
        }),
        Value::ArrayValue(values) => ValueType::ArrayValue(api::ArrayValue {
            values: values
                .into_iter()
                .map(|value| convert_value(project_name, value))
                .collect(),
        }),
    };
    api::Value {
        meaning: 0,
        exclude_from_indexes: false,
        value_type: Some(value_type),
    }
}

pub fn generate_mutations<T, I>(
    project_name: impl AsRef<str>,
    entities: I,
) -> Result<Vec<api::Mutation>, Error>
where
    I: IntoIterator<Item = T>,
    T: IntoEntity,
{
    let mutations: Vec<api::Mutation> = entities
        .into_iter()
        .map(IntoEntity::into_entity)
        .map(|entity| match entity {
            Err(e) => Err(e),
            Ok(entity) => {
                let is_incomplete = entity.key.is_incomplete();
                let entity = convert_entity(project_name.as_ref(), entity);
                Ok(api::Mutation {
                    operation: if is_incomplete {
                        Some(api::mutation::Operation::Insert(entity))
                    } else {
                        Some(api::mutation::Operation::Upsert(entity))
                    },
                    conflict_detection_strategy: None,
                })
            }
        })
        .collect::<Result<_, _>>()?;
    Ok(mutations)
}

fn convert_filter(project_name: &str, filters: Vec<Filter>) -> Option<api::Filter> {
    use api::filter::FilterType;

    if !filters.is_empty() {
        let filters = filters
            .into_iter()
            .map(|filter| {
                use api::property_filter::Operator;
                let (name, op, value) = match filter {
                    Filter::Equal(name, value) => (name, Operator::Equal, value),
                    Filter::GreaterThan(name, value) => (name, Operator::GreaterThan, value),
                    Filter::LesserThan(name, value) => (name, Operator::LessThan, value),
                    Filter::GreaterThanOrEqual(name, value) => {
                        (name, Operator::GreaterThanOrEqual, value)
                    }
                    Filter::LesserThanEqual(name, value) => {
                        (name, Operator::LessThanOrEqual, value)
                    }
                };

                api::Filter {
                    filter_type: Some(FilterType::PropertyFilter(api::PropertyFilter {
                        op: op as i32,
                        property: Some(api::PropertyReference { name }),
                        value: Some(convert_value(project_name, value)),
                    })),
                }
            })
            .collect();

        Some(api::Filter {
            filter_type: Some(FilterType::CompositeFilter(api::CompositeFilter {
                op: api::composite_filter::Operator::And as i32,
                filters,
            })),
        })
    } else {
        None
    }
}
