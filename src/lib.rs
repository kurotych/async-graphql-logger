use async_graphql::{
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextValidation,
    },
    parser::types::{ExecutableDocument, OperationType, Selection},
    Response, ServerError, ServerResult, ValidationResult, Variables,
};
use chrono::{DateTime, Utc};
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Logger;

impl ExtensionFactory for Logger {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(LoggerExtension {})
    }
}

struct LoggerExtension {}

#[derive(Debug, Default)]
pub struct QueryInfo {
    inner: Arc<Mutex<QueryInfoInner>>,
}
impl QueryInfo {
    pub fn new() -> QueryInfo {
        let mut rng = rand::thread_rng();
        QueryInfo {
            inner: Arc::new(Mutex::new(QueryInfoInner {
                id: rng.gen_range(100_000_000..=999_999_999),
                start_time: Utc::now(),
                is_schema: false,
            })),
        }
    }
}

#[derive(Debug, Default)]
pub struct QueryInfoInner {
    id: u64,
    start_time: DateTime<Utc>,
    is_schema: bool,
}

#[async_trait::async_trait]
impl Extension for LoggerExtension {
    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let res = next.run(ctx).await;
        match res {
            Ok(_) => res,
            Err(ref errors) => match ctx.data_opt::<QueryInfo>() {
                Some(qinfo) => {
                    let qinfo = qinfo.inner.lock().await;
                    for e in errors {
                        log::info!(
                            target: "gql_logger",
                            "[QueryID: {}] Validation is failed with reason: {}",
                            qinfo.id,
                            e.message
                        )
                    }
                    res
                }
                None => res,
            },
        }
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let document = next.run(ctx, query, variables).await?;
        if let Some(qinfo) = ctx.data_opt::<QueryInfo>() {
            let is_schema = document
            .operations
            .iter()
            .filter(|(_, operation)| operation.node.ty == OperationType::Query)
            .any(|(_, operation)| operation.node.selection_set.node.items.iter().any(
                    |selection| matches!(&selection.node, Selection::Field(field) if field.node.name.node == "__schema")));
            let mut qinfo = qinfo.inner.lock().await;
            qinfo.is_schema = is_schema;
            if is_schema {
                return Ok(document);
            }
            log::info!(target: "gql_logger", "[QueryID: {}] {}", qinfo.id, ctx.stringify_execute_doc(&document, variables));
        }
        Ok(document)
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let resp = next.run(ctx, operation_name).await;
        if let Some(qinfo) = ctx.data_opt::<QueryInfo>() {
            let qinfo = qinfo.inner.lock().await;
            if qinfo.is_schema {
                return resp;
            }
            let query_id = qinfo.id;

            if resp.is_err() {
                for err in &resp.errors {
                    log::info!(
                        target: "gql_logger",
                        "[QueryID: {query_id}] [Error] {}", err.message,
                    );
                }
            }
            let query_start_time = qinfo.start_time;
            let duration = Utc::now() - query_start_time;
            log::debug!(target: "gql_logger",
                        "[QueryID: {query_id}] Response: {}", resp.data);
            log::info!(
                target: "gql_logger",
                "[QueryID: {query_id}] Duration: {}ms",
                duration.num_milliseconds()
            );
        }

        resp
    }
}
