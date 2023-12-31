use async_graphql::{SimpleObject, Union};
use telegram::TelegramMessage;

#[derive(Union)]
pub enum TelegramMessageResponse {
    Response(TelegramMessageNode),
}

#[derive(PartialEq, Debug, SimpleObject)]
pub struct TelegramMessageNode {
    pub username: String,
    pub message: String,
    pub chat_name: String,
    pub chat_id: String,
}

impl TelegramMessageNode {
    pub fn from_domain(message: TelegramMessage) -> Self {
        TelegramMessageNode {
            username: message
                .from
                .unwrap_or_default()
                .username
                .unwrap_or_default(),
            message: message.text.unwrap_or_default(),
            chat_name: message.chat.name(),
            chat_id: message.chat.id.to_string(),
        }
    }
}
