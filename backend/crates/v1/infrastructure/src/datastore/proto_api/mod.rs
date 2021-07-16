pub mod api {
    pub mod r#type {
        tonic::include_proto!("google.r#type");
    }
    pub mod datastore {
        pub mod v1 {
            tonic::include_proto!("google.datastore.v1");
        }
    }
    pub use datastore::v1::*;
    pub use r#type::*;
}
mod authorize;
mod client;
mod entity;
mod error;
mod key;
mod query;
mod value;

pub use self::client::*;
pub use self::entity::*;
pub use self::key::*;
pub use self::query::*;
pub use self::value::*;

pub type Error = error::Error;
