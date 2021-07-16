use domain::ProjectRepository as Dpr;
use infrastructure::datastore::{ProjectRepository, Transaction};

pub async fn insert_fixture(tx: &mut Transaction, projects: &[domain::Project]) {
    let repo = ProjectRepository::default();
    for project in projects.iter() {
        repo.store(tx, project).await.unwrap();
    }
}
