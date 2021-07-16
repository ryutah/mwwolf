use crate::error;

use crate::*;

/// TODO(ryutah): 空白文字を許容しない共通タイプが合ったほうが良さげ
///   - TryFrom trait実装する形がRust的に自然かも
///   - https://doc.rust-lang.org/std/convert/trait.TryFrom.html
#[derive(Debug, Getters, Clone, PartialEq)]
pub struct ProjectName {
    raw: String,
}

impl ProjectName {
    pub fn try_new<T: Into<String>>(raw: T) -> DomainResult<Self> {
        let raw: String = raw.into();
        if raw.is_empty() {
            Err(error::DomainError::new(
                error::DomainErrorKind::InvalidInput,
                "project_name is empty",
            ))
        } else {
            Ok(ProjectName { raw })
        }
    }
}

impl From<ProjectName> for String {
    fn from(project_name: ProjectName) -> Self {
        project_name.raw
    }
}

#[derive(new, Getters, Debug, Clone)]
pub struct Project {
    id: Id<Project>,
    name: ProjectName,
}

#[async_trait]
pub trait ProjectRepository<C: database::Connection, Tx: database::Transaction> {
    async fn get<'a>(
        &'a self,
        executor: database::Executor<'a, C, Tx>,
        id: &'a Id<Project>,
    ) -> RepositoryResult<Project>;

    async fn store(&self, tx: &mut Tx, project: &Project) -> RepositoryResult<()>;
}

#[async_trait]
pub trait ProjectFactory {
    async fn create(&self, name: String) -> DomainResult<Project>;
}
