use crate::error::{DomainError, RepositoryError};

pub type DomainResult<T> = std::result::Result<T, DomainError>;

pub type RepositoryResult<T> = std::result::Result<T, RepositoryError>;
