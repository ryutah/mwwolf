use std::borrow::Borrow;

use crate::datastore::proto_api::api;
use crate::datastore::proto_api::api::key::path_element::IdType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyID {
    StringID(String),

    IntID(i64),

    Incomplete,
}

impl KeyID {
    pub fn is_incomplete(&self) -> bool {
        matches!(self, KeyID::Incomplete)
    }
}

impl From<i64> for KeyID {
    fn from(id: i64) -> KeyID {
        KeyID::IntID(id)
    }
}

impl From<&str> for KeyID {
    fn from(id: &str) -> KeyID {
        KeyID::from(String::from(id))
    }
}

impl From<String> for KeyID {
    fn from(id: String) -> KeyID {
        KeyID::StringID(id)
    }
}

impl From<IdType> for KeyID {
    fn from(id_type: IdType) -> KeyID {
        match id_type {
            IdType::Id(id) => KeyID::IntID(id),
            IdType::Name(id) => KeyID::StringID(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub(crate) kind: String,
    pub(crate) id: KeyID,
    pub(crate) parent: Option<Box<Key>>,
    pub(crate) namespace: Option<String>,
}

impl Key {
    pub fn new(kind: impl Into<String>) -> Key {
        Key {
            kind: kind.into(),
            id: KeyID::Incomplete,
            parent: None,
            namespace: None,
        }
    }

    pub fn get_kind(&self) -> &str {
        self.kind.as_str()
    }

    pub fn id(mut self, id: impl Into<KeyID>) -> Key {
        self.id = id.into();
        self
    }

    pub fn get_id(&self) -> &KeyID {
        &self.id
    }

    pub fn parent(mut self, parent: impl Into<Box<Key>>) -> Key {
        self.parent = Some(parent.into());
        self
    }

    pub fn get_parent(&self) -> Option<&Key> {
        self.parent.as_ref().map(|inner| inner.borrow())
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Key {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn get_namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn is_incomplete(&self) -> bool {
        self.get_id().is_incomplete()
    }
}

impl From<api::Key> for Key {
    fn from(key: api::Key) -> Key {
        let data = key.partition_id.unwrap();
        let key = key.path.into_iter().fold(None, |acc, el| {
            let key_id = match el.id_type {
                None => KeyID::Incomplete,
                Some(id_type) => KeyID::from(id_type),
            };
            let key = Key::new(el.kind);
            let key = if data.namespace_id.is_empty() {
                key
            } else {
                key.namespace(data.namespace_id.as_str())
            };
            let key = key.id(key_id);

            if let Some(ancestor) = acc {
                Some(key.parent(ancestor))
            } else {
                Some(key)
            }
        });

        //? There should always be at least one.
        key.unwrap()
    }
}
