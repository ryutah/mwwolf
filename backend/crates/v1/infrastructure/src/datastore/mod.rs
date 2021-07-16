use anyhow::anyhow;
use async_std::sync::Arc;
use async_std::sync::Mutex;
use libmww::database::Executor;
use std::borrow::Borrow;

use crate::*;

mod proto_api;

use proto_api::{api, Client, FromValue, IntoEntity, Key};

mod id;
mod project;
use id::*;
pub use project::*;

#[derive(Clone)]
pub struct Transaction {
    project_id: String,
    namespace: String,
    transaction: prost::alloc::vec::Vec<u8>,
    client: Arc<Mutex<Client>>,
    mutations: prost::alloc::vec::Vec<api::Mutation>,
}

impl Transaction {
    pub async fn new(
        project_id: String,
        namespace: String,
        transaction: prost::alloc::vec::Vec<u8>,
        client: Arc<Mutex<Client>>,
    ) -> Result<Transaction, database::DatabaseError> {
        Ok(Transaction {
            project_id,
            namespace,
            transaction,
            client,
            mutations: vec![],
        })
    }

    pub async fn get<T: FromValue>(
        &mut self,
        key: impl Borrow<Key>,
    ) -> Result<Option<T>, proto_api::Error> {
        self.client
            .lock()
            .await
            .get(key, Some(self.transaction.clone()))
            .await
    }

    pub async fn get_all<T, K, I>(&mut self, keys: I) -> Result<Vec<T>, proto_api::Error>
    where
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
        T: FromValue,
    {
        self.client
            .lock()
            .await
            .get_all(keys, Some(self.transaction.clone()))
            .await
    }

    pub async fn put(&mut self, entity: impl IntoEntity) -> Result<(), proto_api::Error> {
        self.put_all(Some(entity)).await
    }

    pub async fn put_all<T, I>(&mut self, entities: I) -> Result<(), proto_api::Error>
    where
        I: IntoIterator<Item = T>,
        T: IntoEntity,
    {
        let mutations = proto_api::generate_mutations(&self.project_id, entities)?;
        self.mutations.extend(mutations);
        Ok(())
    }
}

#[async_trait]
impl database::Transaction for Transaction {
    async fn commit(mut self) -> Result<(), database::DatabaseError> {
        let commit_request = api::CommitRequest {
            project_id: self.project_id,
            mode: api::commit_request::Mode::Transactional.into(),
            mutations: self.mutations,
            transaction_selector: Some(api::commit_request::TransactionSelector::Transaction(
                self.transaction,
            )),
        };
        let response = self
            .client
            .lock()
            .await
            .service
            .commit(commit_request)
            .await
            .map_err(|e| database::DatabaseError::FailedTransactionCommit(anyhow!("{}", e)))?;
        let commit_response = response.into_inner();
        for result in commit_response.mutation_results.iter() {
            if result.conflict_detected {
                return Err(database::DatabaseError::FailedTransactionCommit(anyhow!(
                    "conflit detected"
                )));
            }
        }
        Ok(())
    }

    async fn rollback(mut self) -> Result<(), database::DatabaseError> {
        let rollback_request = api::RollbackRequest {
            project_id: self.project_id,
            transaction: self.transaction,
        };
        let _ = self
            .client
            .lock()
            .await
            .service
            .rollback(rollback_request)
            .await
            .map_err(|e| database::DatabaseError::FailedTransactionRollback(anyhow!("{}", e)))?;
        Ok(())
    }
}

#[derive(new)]
pub struct Connection {
    project_id: String,
    namespace: String,
    client: Arc<Mutex<Client>>,
}

impl Connection {
    pub async fn get<T: FromValue>(
        &mut self,
        key: impl Borrow<Key>,
    ) -> Result<Option<T>, proto_api::Error> {
        self.client.lock().await.get(key, None).await
    }

    pub async fn get_all<T, K, I>(&mut self, keys: I) -> Result<Vec<T>, proto_api::Error>
    where
        I: IntoIterator<Item = K>,
        K: Borrow<Key>,
        T: FromValue,
    {
        self.client.lock().await.get_all(keys, None).await
    }

    pub async fn put(&mut self, entity: impl IntoEntity) -> Result<Option<Key>, proto_api::Error> {
        self.client.lock().await.put(entity).await
    }

    pub async fn put_all<T, I>(&mut self, entities: I) -> Result<Vec<Option<Key>>, proto_api::Error>
    where
        I: IntoIterator<Item = T>,
        T: IntoEntity,
    {
        self.client.lock().await.put_all(entities).await
    }

    pub async fn allocate_ids(&self, keys: &[Key]) -> Result<Vec<api::Key>, proto_api::Error> {
        let request = api::AllocateIdsRequest {
            project_id: self.project_id.clone(),
            keys: keys
                .iter()
                .map(|key| proto_api::convert_key(&self.project_id, key))
                .collect(),
        };
        let response = self
            .client
            .lock()
            .await
            .service
            .allocate_ids(request)
            .await?;
        Ok(response.into_inner().keys)
    }
}

#[async_trait]
impl database::Connection for Connection {
    type Transaction = Transaction;
    async fn begin(&mut self) -> Result<Self::Transaction, database::DatabaseError> {
        let begin_transaction_request = api::BeginTransactionRequest {
            project_id: self.project_id.clone(),
            transaction_options: Some(api::TransactionOptions {
                mode: Some(api::transaction_options::Mode::ReadWrite(
                    api::transaction_options::ReadWrite::default(),
                )),
            }),
        };
        let response = self
            .client
            .lock()
            .await
            .service
            .begin_transaction(begin_transaction_request)
            .await
            .map_err(|e| database::DatabaseError::FailedTransactionBegin(anyhow!("{}", e)))?;
        let tx_response = response.into_inner();

        Transaction::new(
            self.project_id.clone(),
            self.namespace.clone(),
            tx_response.transaction,
            self.client.clone(),
        )
        .await
    }
}

#[derive(new)]
pub struct ConnectionFactory {
    project_id: String,
    namespace: String,
}

#[async_trait]
impl database::ConnectionFactory for ConnectionFactory {
    type Connection = Connection;
    type Transaction = Transaction;
    async fn create(&self) -> Result<Self::Connection, database::DatabaseError> {
        Ok(Connection::new(
            self.project_id.clone(),
            self.namespace.clone(),
            Arc::new(Mutex::new(new_client(self.project_id.clone()).await?)),
        ))
    }
}

async fn new_client(project_name: impl Into<String>) -> Result<Client, database::DatabaseError> {
    Client::new(project_name)
        .await
        .map_err(convert_datastore_error_database_error)
}

fn convert_datastore_error_database_error(err: proto_api::Error) -> database::DatabaseError {
    database::DatabaseError::FailedOpen(err.into())
}

fn namespace_from_executor<'a>(executor: &'a Executor<Connection, Transaction>) -> &'a str {
    match executor {
        Executor::<Connection, Transaction>::Connection(conn) => &conn.namespace,
        Executor::<Connection, Transaction>::Transaction(tx) => &tx.namespace,
    }
}
#[cfg(test)]
mod tests {}
