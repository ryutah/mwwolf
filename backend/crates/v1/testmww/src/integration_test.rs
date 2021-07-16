use async_std::sync::Arc;
use infrastructure::datastore;
use std::env;
use uuid::Uuid;

pub struct ConnectionFactoryGurad {
    cf: Arc<datastore::ConnectionFactory>,
}

impl AsRef<Arc<datastore::ConnectionFactory>> for ConnectionFactoryGurad {
    fn as_ref(&self) -> &Arc<datastore::ConnectionFactory> {
        &self.cf
    }
}

impl AsMut<Arc<datastore::ConnectionFactory>> for ConnectionFactoryGurad {
    fn as_mut(&mut self) -> &mut Arc<datastore::ConnectionFactory> {
        &mut self.cf
    }
}

pub async fn init_test_database() -> Result<ConnectionFactoryGurad, anyhow::Error> {
    let db_uuid = Uuid::new_v4();
    let namespace = db_uuid.to_string().replace("-", "");
    let cf = Arc::new(datastore::ConnectionFactory::new(
        env::var("GOOGLE_CLOUD_PROJECT").unwrap(),
        namespace,
    ));
    let cf = ConnectionFactoryGurad { cf };
    Ok(cf)
}
