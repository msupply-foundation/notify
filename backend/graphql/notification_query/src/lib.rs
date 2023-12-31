mod mutations;
use self::mutations::*;
mod types;
use self::types::*;

use async_graphql::*;
use graphql_core::{
    pagination::PaginationInput,
    standard_graphql_error::{validate_auth, StandardGraphqlError},
    ContextExt,
};
use repository::NotificationQueryFilter;
use repository::PaginationOption;
use service::auth::{Resource, ResourceAccessRequest};

#[derive(Default, Clone)]
pub struct NotificationQueryQueries;

#[Object]
impl NotificationQueryQueries {
    pub async fn notification_queries(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Pagination option (first and offset)")] page: Option<PaginationInput>,
        #[graphql(desc = "Filter option")] filter: Option<NotificationQueryFilterInput>,
        #[graphql(desc = "Sort options (only first sort input is evaluated for this endpoint)")]
        sort: Option<Vec<NotificationQuerySortInput>>,
    ) -> Result<NotificationQueriesResponse> {
        let user = validate_auth(
            ctx,
            &ResourceAccessRequest {
                resource: Resource::ServerAdmin,
            },
        )?;

        let service_context = ctx.service_context(Some(&user))?;

        let configs = service_context
            .service_provider
            .notification_query_service
            .get_notification_queries(
                &service_context,
                page.map(PaginationOption::from),
                filter.map(NotificationQueryFilter::from),
                // Currently only one sort option is supported, use the first from the list.
                sort.and_then(|mut sort_list| sort_list.pop())
                    .map(|sort| sort.to_domain()),
            )
            .map_err(StandardGraphqlError::from_list_error)?;

        Ok(NotificationQueriesResponse::Response(
            NotificationQueryConnector::from_domain(configs),
        ))
    }
}

#[derive(Default, Clone)]
pub struct NotificationQueryMutations;

#[Object]
impl NotificationQueryMutations {
    async fn create_notification_query(
        &self,
        ctx: &Context<'_>,
        input: CreateNotificationQueryInput,
    ) -> Result<ModifyNotificationQueryResponse> {
        create_notification_query(ctx, input)
    }

    async fn update_notification_query(
        &self,
        ctx: &Context<'_>,
        input: UpdateNotificationQueryInput,
    ) -> Result<ModifyNotificationQueryResponse> {
        update_notification_query(ctx, input)
    }

    async fn delete_notification_query(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<DeleteNotificationQueryResponse> {
        delete_notification_query(ctx, &id)
    }
}
