use async_graphql::*;
use graphql_core::standard_graphql_error::StandardGraphqlError::*;
use graphql_core::{standard_graphql_error::validate_auth, ContextExt};
use service::{
    auth::{Resource, ResourceAccessRequest},
    datasource::DatasourceServiceError,
};
mod types;
use self::types::*;


pub fn map_error(error: DatasourceServiceError) -> Result<QueryResultResponse> {
    let graphql_error = match error {
        DatasourceServiceError::InternalError(e) => InternalError(e),
        DatasourceServiceError::BadUserInput(e) => BadUserInput(e),
    };

    Err(graphql_error.extend())
}

#[derive(Default, Clone)]
pub struct DatasourceQueries;

#[Object]
impl DatasourceQueries {
    pub async fn run_sql_query(&self, ctx: &Context<'_>, sql_query: String) -> Result<QueryResultResponse> {
        let user = validate_auth(
            ctx,
            &ResourceAccessRequest {
                resource: Resource::ServerAdmin,
            },
        )?;

        let service_ctx = ctx.service_context(Some(&user))?;
        let datasource_service = &service_ctx.service_provider.datasource_service;
        // TODO some kind of query validation?

        // Query datasource service and return result
        match datasource_service.run_sql_query(sql_query) {
            Ok(result) => Ok(QueryResultResponse::Response(QueryResultNode::from_domain(result))),
            Err(error) => map_error(error),
        }
    }

    pub async fn run_sql_query_with_parameters(
        &self,
        ctx: &Context<'_>,
        sql_query: String,
        parameters: String,
    ) -> Result<QueryResultResponse> {
        let user = validate_auth(
            ctx,
            &ResourceAccessRequest {
                resource: Resource::ServerAdmin,
            },
        )?;

        let service_ctx = ctx.service_context(Some(&user))?;
        let datasource_service = &service_ctx.service_provider.datasource_service;

        // convert parameters to json
        let parameters: serde_json::Value = serde_json::from_str(&parameters).map_err(|err| {
            BadUserInput(format!(
                "Invalid parameters string: {}. Error: {}",
                parameters, err
            ))
        })?;

        // Query datasource service and return result
        match datasource_service.run_sql_query_with_parameters(sql_query, parameters) {
            Ok(result) => Ok(QueryResultResponse::Response(QueryResultNode::from_domain(result))),
            Err(error) => map_error(error),
        }
    }
}
