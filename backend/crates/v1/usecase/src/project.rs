use crate::*;
use async_std::sync::Arc;

#[derive(new)]
pub struct ProjectUsecase<
    CF: ConnectionFactory,
    R: domain::ProjectRepository<CF::Connection, CF::Transaction>,
    F: domain::ProjectFactory,
> {
    connection_factory: Arc<CF>,
    repository: R,
    factory: F,
}

impl<
        CF: ConnectionFactory,
        R: domain::ProjectRepository<CF::Connection, CF::Transaction>,
        F: domain::ProjectFactory,
    > ProjectUsecase<CF, R, F>
{
    pub async fn get(&self, id: &str) -> Result<domain::Project, UsecaseError> {
        let mut conn = self.connection_factory.create().await?;

        let id = domain::Id::<domain::Project>::new(id);
        let project = self
            .repository
            .get(Executor::Connection(&mut conn), &id)
            .await;

        match project {
            Err(error) => match error.kind() {
                domain::RepositoryErrorKind::NotFound => {
                    Err(UsecaseError::Notfound("not found".into(), error.into()))
                }
                _ => Err(UsecaseError::Fail(
                    "internal_server_error".into(),
                    error.into(),
                )),
            },
            Ok(project) => Ok(project),
        }
    }

    pub async fn create(&self, name: String) -> Result<domain::Project, UsecaseError> {
        let project = self.factory.create(name).await?;

        let mut conn = self.connection_factory.create().await?;
        let result =
            run_in_transaction!(conn, tx, { self.repository.store(&mut tx, &project).await });

        match result {
            Err(error) => Err(UsecaseError::Fail(
                "failed to create project".into(),
                error.into(),
            )),
            Ok(_) => Ok(project),
        }
    }
}
