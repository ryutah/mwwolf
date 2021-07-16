use async_graphql::*;

#[derive(SimpleObject, new)]
pub struct Project {
    id: String,
    name: String,
}

#[derive(Default)]
pub struct ProjectQuery;

#[Object]
impl ProjectQuery {
    async fn project(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "id of the project")] id: String,
    ) -> Result<Project> {
        let usecase = di::Injector::project_usecase(ctx);
        let project = usecase.get(&id).await?;
        Ok(Project::new(
            project.id().raw_id().into(),
            project.name().raw().into(),
        ))
    }
}

#[derive(Default)]
pub struct ProjectMutaion;

#[Object]
impl ProjectMutaion {
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "name of new project")] name: String,
    ) -> Result<Project> {
        let usecase = di::Injector::project_usecase(ctx);
        let new_project = usecase.create(name).await?;
        Ok(Project::new(
            new_project.id().raw_id().into(),
            new_project.name().raw().into(),
        ))
    }
}
