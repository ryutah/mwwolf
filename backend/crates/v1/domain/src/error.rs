use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum DomainErrorKind {
    InvalidInput,
    Conflict,
    Notfound,
    Forbidden,
    Fail,
}

#[derive(Debug, Getters)]
pub struct DomainError {
    kind: DomainErrorKind,
    message: String,
    source: Option<anyhow::Error>,
    sub_errors: Vec<DomainError>,
}

#[derive(Debug, PartialEq)]
pub enum RepositoryErrorKind {
    NotFound,
    Conflict,
    Fail,
}

#[derive(Debug, Getters)]
pub struct RepositoryError {
    kind: RepositoryErrorKind,
    message: String,
    source: Option<anyhow::Error>,
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "kind{:?},message:{},source,:{:?},sub_errors:{:?}",
            self.kind, self.message, self.source, self.sub_errors
        )
    }
}

impl DomainError {
    pub fn new<M: Into<String>>(kind: DomainErrorKind, message: M) -> Self {
        Self::inner_new(kind, message, None, vec![])
    }
    pub fn new_with_source<M: Into<String>>(
        kind: DomainErrorKind,
        message: M,
        source: anyhow::Error,
    ) -> Self {
        Self::inner_new(kind, message, Some(source), vec![])
    }

    pub fn new_with_sub_errors<M: Into<String>>(
        kind: DomainErrorKind,
        message: M,
        sub_errors: Vec<DomainError>,
    ) -> Self {
        Self::inner_new(kind, message, None, sub_errors)
    }

    fn inner_new<M: Into<String>>(
        kind: DomainErrorKind,
        message: M,
        source: Option<anyhow::Error>,
        sub_errors: Vec<DomainError>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            source,
            sub_errors,
        }
    }
}

impl std::error::Error for DomainError {}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref source) = self.source {
            write!(
                f,
                "kind:{:?},message:{},source:{:?}",
                self.kind, self.message, source,
            )
        } else {
            write!(
                f,
                "kind:{:?},message:{},source:None",
                self.kind, self.message
            )
        }
    }
}

impl std::error::Error for RepositoryError {}

impl RepositoryError {
    pub fn new<M: Into<String>>(kind: RepositoryErrorKind, message: M) -> Self {
        Self::inner_new(kind, message, None)
    }
    pub fn new_with_source<M: Into<String>>(
        kind: RepositoryErrorKind,
        message: M,
        source: anyhow::Error,
    ) -> Self {
        Self::inner_new(kind, message, Some(source))
    }

    fn inner_new<M: Into<String>>(
        kind: RepositoryErrorKind,
        message: M,
        source: Option<anyhow::Error>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            source,
        }
    }
}

impl PartialEq for DomainError {
    fn eq(&self, t: &Self) -> bool {
        self.kind == t.kind && self.message == t.message
    }
}
impl PartialEq for RepositoryError {
    fn eq(&self, t: &Self) -> bool {
        self.kind == t.kind && self.message == t.message
    }
}
