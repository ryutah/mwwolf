use std::collections::HashMap;

use super::proto_api::*;
use super::*;
use database::ConnectionFactory as CF;

#[derive(Default)]
pub struct ProjectRepository;

const KIND: &str = "project";

const PROJECT_FIELDS_NAME: &str = "name";

#[async_trait]
impl domain::ProjectRepository<Connection, Transaction> for ProjectRepository {
    async fn get<'a>(
        &'a self,
        executor: database::Executor<'a, Connection, Transaction>,
        id: &'a domain::Id<domain::Project>,
    ) -> domain::RepositoryResult<domain::Project> {
        let key = new_key(namespace_from_executor(&executor)).id(id.to_string());

        let entity: Option<HashMap<String, Value>> = match executor {
            database::Executor::Connection(conn) => conn.get(&key).await.map_err(|e| {
                domain::RepositoryError::new_with_source(
                    domain::RepositoryErrorKind::Fail,
                    "failed_to_get_entity",
                    e.into(),
                )
            }),
            database::Executor::Transaction(tx) => tx.get(&key).await.map_err(|e| {
                domain::RepositoryError::new_with_source(
                    domain::RepositoryErrorKind::Fail,
                    "failed_to_get_entity",
                    e.into(),
                )
            }),
        }?;

        match entity {
            Some(mut entity) => {
                let project_name = entity.remove(PROJECT_FIELDS_NAME).unwrap();
                let id = match key.get_id() {
                    KeyID::StringID(s) => Ok(s),
                    _ => Err(domain::RepositoryError::new(
                        domain::RepositoryErrorKind::Fail,
                        "unknown id value",
                    )),
                }?;
                Ok(domain::Project::new(
                    domain::Id::new(id),
                    domain::ProjectName::try_new(String::from_value(project_name).unwrap())
                        .unwrap(),
                ))
            }
            None => Err(domain::RepositoryError::new(
                domain::RepositoryErrorKind::NotFound,
                "not found",
            )),
        }
    }

    async fn store(
        &self,
        tx: &mut Transaction,
        project: &domain::Project,
    ) -> domain::RepositoryResult<()> {
        let key = new_key(&tx.namespace).id(project.id().to_string());
        let mut values = HashMap::new();
        values.insert(
            PROJECT_FIELDS_NAME.to_owned(),
            project.name().raw().clone().into_value(),
        );

        let entity = Entity::new(key, values).unwrap();
        tx.put(entity).await.map_err(|e| {
            domain::RepositoryError::new_with_source(
                domain::RepositoryErrorKind::Fail,
                "failed to put",
                e.into(),
            )
        })
    }
}
#[derive(new)]
pub struct ProjectFactory {
    connection_factory: Arc<ConnectionFactory>,
}

#[async_trait]
impl domain::ProjectFactory for ProjectFactory {
    async fn create(&self, project_name: String) -> domain::DomainResult<domain::Project> {
        let conn = self.connection_factory.create().await.map_err(|e| {
            domain::DomainError::new_with_source(
                domain::DomainErrorKind::Fail,
                "failed connection to datastore",
                e.into(),
            )
        })?;
        let mut ids =
            allocate_ids::<domain::Project>(&conn, &[new_key(&self.connection_factory.namespace)])
                .await?;
        let id = ids.remove(0);
        let project = domain::Project::new(id, domain::ProjectName::try_new(project_name)?);
        Ok(project)
    }
}

fn new_key(namespace: impl Into<String>) -> Key {
    Key::new(KIND).namespace(namespace)
}
