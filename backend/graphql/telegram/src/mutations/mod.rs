use crate::types::{TelegramMessageNode, TelegramMessageResponse};
use async_graphql::*;
use async_graphql::{Context, Object};
use graphql_core::{
    standard_graphql_error::{validate_auth, StandardGraphqlError},
    ContextExt,
};
use service::auth::{Resource, ResourceAccessRequest};

#[derive(Default, Clone)]
pub struct TelegramMutations;

#[Object]
impl TelegramMutations {
    async fn send_test_message(
        &self,
        ctx: &Context<'_>,
        chat_id: String,
    ) -> Result<TelegramMessageResponse> {
        let user = validate_auth(
            ctx,
            &ResourceAccessRequest {
                resource: Resource::ServerAdmin,
            },
        )?;

        let service_ctx = ctx.service_context(Some(&user))?;
        let telegram_service = &service_ctx.service_provider.telegram;

        match telegram_service {
            Some(telegram_service) => {
                let message = telegram_service
                    .send_html_message(&chat_id, "This is a test message from notify")
                    .await;
                match message {
                    Ok(message) => {
                        return Ok(TelegramMessageResponse::Response(TelegramMessageNode {
                            msg_json: message,
                        }))
                    }
                    Err(err) => {
                        return Err(StandardGraphqlError::InternalError(format!(
                            "Can't send message : {:?}",
                            err
                        ))
                        .extend())
                    }
                }
            }
            None => {
                return Err(StandardGraphqlError::InternalError(
                    "Telegram service not configured".to_string(),
                )
                .extend())
            }
        }
    }
}