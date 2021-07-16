use thiserror::Error;

#[derive(Error, Debug)]
pub enum UsecaseError {
    #[error("{0}")]
    DomainError(domain::DomainError),
    #[error("{0}")]
    Notfound(String, anyhow::Error),
    #[error("{0}")]
    Fail(String, anyhow::Error),
    #[error("{0}")]
    DatabaseError(libmww::database::DatabaseError),
}

impl From<domain::DomainError> for UsecaseError {
    fn from(err: domain::DomainError) -> Self {
        UsecaseError::DomainError(err)
    }
}

impl From<libmww::database::DatabaseError> for UsecaseError {
    fn from(err: libmww::database::DatabaseError) -> Self {
        UsecaseError::DatabaseError(err)
    }
}

impl PartialEq for UsecaseError {
    fn eq(&self, t: &Self) -> bool {
        match (self, t) {
            (UsecaseError::DomainError(de), UsecaseError::DomainError(det)) => de == det,
            (UsecaseError::Notfound(m, _), UsecaseError::Notfound(mt, _)) => m == mt,
            (UsecaseError::Fail(m, _), UsecaseError::Fail(mt, _)) => m == mt,
            (UsecaseError::DatabaseError(de), UsecaseError::DatabaseError(det)) => de == det,
            _ => false,
        }
    }
}
