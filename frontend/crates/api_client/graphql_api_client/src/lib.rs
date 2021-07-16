#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate derive_new;

use anyhow::*;
use graphql_client::GraphQLQuery;

#[async_trait]
pub trait HttpClient {
    async fn send_reqeust(&self, body: surf::Body) -> Result<surf::Response>;
}

#[derive(new)]
pub struct HttpClientImpl {
    url: String,
}

#[async_trait]
impl HttpClient for HttpClientImpl {
    async fn send_reqeust(&self, body: surf::Body) -> Result<surf::Response> {
        surf::post(&self.url)
            .body(body)
            .await
            .map_err(|e| anyhow!("status:{},err:{}", e.status(), e.to_string()))
    }
}

#[derive(new)]
pub struct ApiClient<H: HttpClient> {
    http_client: H,
}

impl<H: HttpClient> ApiClient<H> {
    pub async fn send_query<GQ: GraphQLQuery>(
        &self,
        variables: GQ::Variables,
    ) -> Result<GQ::ResponseData> {
        let query = GQ::build_query(variables);
        let body = surf::Body::from_json(&query).unwrap();
        let mut response = self.http_client.send_reqeust(body).await?;
        let payload: graphql_client::Response<GQ::ResponseData> = response
            .body_json()
            .await
            .map_err(|e| anyhow!("status:{},err:{}", e.status(), e.to_string()))?;
        payload.data.ok_or_else(|| anyhow!("failed read data"))
    }
}
