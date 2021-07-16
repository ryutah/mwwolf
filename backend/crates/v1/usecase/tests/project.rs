use anyhow::anyhow;
use libmww::database::*;
use test_case::test_case;
use usecase::UsecaseError;

#[test_case("1",Some(domain::Project::new(domain::Id::new("1"),domain::ProjectName::try_new("project1").unwrap()))=>Ok(()))]
#[test_case("2",None=>Err(UsecaseError::Notfound("not found".to_string(),anyhow!(""))))]
#[async_std::test]
async fn get_project(
    id: &str,
    expected_project: Option<domain::Project>,
) -> Result<(), UsecaseError> {
    let cf = testmww::integration_test::init_test_database()
        .await
        .unwrap();

    {
        let mut conn = cf.as_ref().create().await.unwrap();
        let mut tx = conn.begin().await.unwrap();

        if let Some(ref expected_project) = expected_project {
            testmww::integration_project_test::insert_fixture(
                &mut tx,
                &[domain::Project::new(
                    domain::Id::new(id),
                    expected_project.name().clone(),
                )],
            )
            .await;
        }
        tx.commit().await?;
    }

    let got = di::new_project(cf.as_ref().clone()).get(id).await?;
    let expected_project = expected_project.unwrap();
    assert_eq!(expected_project.id(), got.id());
    assert_eq!(expected_project.name(), got.name());
    Ok(())
}

#[test_case("test_projec")]
#[test_case("test_projec2")]
#[async_std::test]
async fn create_project_test(name: &str) -> Result<(), UsecaseError> {
    let cf = testmww::integration_test::init_test_database()
        .await
        .unwrap();

    let usecase = di::new_project(cf.as_ref().clone());
    let got = usecase.create(name.into()).await?;
    assert_ne!("", got.id().raw_id());
    assert_eq!(&domain::ProjectName::try_new(name).unwrap(), got.name());

    let got_project = usecase.get(got.id().raw_id()).await?;
    assert_eq!(got.id(), got_project.id());
    assert_eq!(got.name(), got_project.name());

    Ok(())
}
