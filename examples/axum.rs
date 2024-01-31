use std::net::SocketAddr;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    http::HeaderMap,
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use std::time::Duration;

#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    pub async fn health_check(&self, _ctx: &Context<'_>, _input: i32) -> Result<bool> {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(true)
    }
}
async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/api")))
}

async fn graphql_handler(
    schema: Extension<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
    _headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    req = req.data(async_graphql_logger::QueryInfo::new());
    schema.execute(req).await.into()
}

// Example query
// query {
//  healthCheck(input: 1)
// }
#[tokio::main]
async fn main() {
    env_logger::init();
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .extension(async_graphql_logger::Logger)
        .finish();
    let app = Router::new()
        .route("/api", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
    println!("Running http://127.0.0.1:8888/api");
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
