use super::{proto_api::*, *};
pub async fn allocate_ids<T>(
    connection: &Connection,
    keys: &[Key],
) -> domain::DomainResult<Vec<domain::Id<T>>> {
    connection
        .allocate_ids(keys)
        .await
        .map_err(|e| {
            domain::DomainError::new_with_source(
                domain::DomainErrorKind::Fail,
                "failed allocated ids",
                e.into(),
            )
        })
        .map(|keys| {
            keys.into_iter()
                .map(|key| key_to_id(Key::from(key)))
                .collect::<Vec<_>>()
        })
}

fn key_to_id<T>(key: Key) -> domain::Id<T> {
    match key.get_id() {
        KeyID::IntID(id) => domain::Id::new(id.to_string()),
        id => panic!("unexpected id:{:?}", id),
    }
}
