use actix_web::middleware::Logger;
use actix_web::{guard, web, App, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

use async_graphql::EmptySubscription;
use async_graphql_actix_web::{Request, Response};
use graphql::KzSchema;

async fn index(schema: web::Data<KzSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Playground: http://localhost:8000");
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    HttpServer::new(move || {
        let schema = di::create_schema(
            graphql::Query::default(),
            graphql::Mutation::default(),
            EmptySubscription,
        );
        let cors = actix_cors::Cors::permissive();
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(actix_web::web::Data::new(schema))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
