use crate::*;
use async_std::sync::Arc;

pub fn new_project(cf: Arc<ConnectionFactory>) -> ProjectUsecase {
    usecase::ProjectUsecase::new(
        cf.clone(),
        create_project_repository(),
        create_project_factory(cf),
    )
}
