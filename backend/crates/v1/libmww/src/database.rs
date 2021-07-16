use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("{0}")]
    FailedOpen(anyhow::Error),
    #[error("{0}")]
    FailedTransactionBegin(anyhow::Error),
    #[error("{0}")]
    FailedTransactionRollback(anyhow::Error),
    #[error("{0}")]
    FailedTransactionCommit(anyhow::Error),
}

#[macro_export]
macro_rules! run_in_transaction {
    ($ex:expr,$tx:ident,$s:block) => {{
        let mut $tx = $ex.begin().await?;
        let r = $s;
        if r.is_ok() {
            $tx.commit().await?;
        } else {
            $tx.rollback().await?;
        }
        r
    }};
}

#[async_trait]
pub trait ConnectionFactory {
    type Transaction: Transaction;
    type Connection: Connection<Transaction = Self::Transaction>;

    async fn create(&self) -> Result<Self::Connection, DatabaseError>;
}

#[async_trait]
pub trait Connection {
    type Transaction: Transaction;

    async fn begin(&mut self) -> Result<Self::Transaction, DatabaseError>;
}

#[async_trait]
pub trait Transaction {
    async fn commit(self) -> Result<(), DatabaseError>;
    async fn rollback(self) -> Result<(), DatabaseError>;
}

pub enum Executor<'a, C: Connection, Tx: Transaction> {
    Connection(&'a mut C),
    Transaction(&'a mut Tx),
}

impl PartialEq for DatabaseError {
    fn eq(&self, t: &Self) -> bool {
        matches!(
            (self, t),
            (DatabaseError::FailedOpen(_), DatabaseError::FailedOpen(_))
                | (
                    DatabaseError::FailedTransactionBegin(_),
                    DatabaseError::FailedTransactionBegin(_),
                )
                | (
                    DatabaseError::FailedTransactionRollback(_),
                    DatabaseError::FailedTransactionRollback(_)
                )
                | (
                    DatabaseError::FailedTransactionCommit(_),
                    DatabaseError::FailedTransactionCommit(_)
                )
        )
    }
}
