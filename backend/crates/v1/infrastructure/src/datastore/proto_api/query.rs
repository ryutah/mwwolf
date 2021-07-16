use crate::datastore::proto_api::{Key, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Asc(String),

    Desc(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Equal(String, Value),

    GreaterThan(String, Value),

    LesserThan(String, Value),

    GreaterThanOrEqual(String, Value),

    LesserThanEqual(String, Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub(crate) kind: String,
    pub(crate) eventual: bool,
    pub(crate) keys_only: bool,
    pub(crate) offset: i32,
    pub(crate) limit: Option<i32>,
    pub(crate) ancestor: Option<Key>,
    pub(crate) namespace: Option<String>,
    pub(crate) projections: Vec<String>,
    pub(crate) distinct_on: Vec<String>,
    pub(crate) ordering: Vec<Order>,
    pub(crate) filters: Vec<Filter>,
}

impl Query {
    pub fn new(kind: impl Into<String>) -> Query {
        Query {
            kind: kind.into(),
            eventual: false,
            keys_only: false,
            offset: 0,
            limit: None,
            ancestor: None,
            namespace: None,
            projections: Vec::new(),
            distinct_on: Vec::new(),
            ordering: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn eventually_consistent(mut self) -> Query {
        self.eventual = true;
        self
    }

    pub fn keys_only(mut self) -> Query {
        self.keys_only = true;
        self
    }

    pub fn offset(mut self, offset: i32) -> Query {
        self.offset = offset;
        self
    }

    pub fn limit(mut self, limit: i32) -> Query {
        self.limit = Some(limit);
        self
    }

    pub fn ancestor(mut self, key: Key) -> Query {
        self.ancestor = Some(key);
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Query {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn project<T, I>(mut self, projections: I) -> Query
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.projections.clear();
        self.projections
            .extend(projections.into_iter().map(Into::into));
        self
    }

    pub fn distinct_on<T, I>(mut self, fields: I) -> Query
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.distinct_on.clear();
        self.distinct_on.extend(fields.into_iter().map(Into::into));
        self
    }

    pub fn filter(mut self, filter: Filter) -> Query {
        self.filters.push(filter);
        self
    }

    pub fn order(mut self, order: Order) -> Query {
        self.ordering.push(order);
        self
    }
}
