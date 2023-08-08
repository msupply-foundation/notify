use serde_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramChat {
    pub id: serde_json::Value,
    pub title: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUser {
    pub username: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessage {
    pub message_id: serde_json::Value,
    pub text: String,
    pub from: TelegramUser,
    pub chat: TelegramChat,
}

pub struct TelegramClient {
    http_client: reqwest::Client,
    base_url: String,
}

#[derive(Debug)]
pub enum TemporaryErrorType {
    TimedOut(String),
    TooManyRequests,
    InternalServerError(String),
    Other(String),
}

#[derive(Debug)]
pub enum TelegramError {
    Fatal(String),
    Temporary(TemporaryErrorType),
}

impl From<reqwest::Error> for TelegramError {
    fn from(error: reqwest::Error) -> TelegramError {
        // TODO, revisit fatal vs non fatal errors?
        if error.is_timeout() {
            return TelegramError::Temporary(TemporaryErrorType::TimedOut(error.to_string()));
        }
        if let Some(status) = error.status() {
            match status {
                reqwest::StatusCode::TOO_MANY_REQUESTS => {
                    return TelegramError::Temporary(TemporaryErrorType::TooManyRequests);
                }
                reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                    return TelegramError::Temporary(TemporaryErrorType::InternalServerError(
                        error.to_string(),
                    ));
                }
                _ => return TelegramError::Fatal(error.to_string()),
            }
        }

        TelegramError::Fatal(error.to_string())
    }
}

impl TelegramClient {
    pub fn new(token: String) -> TelegramClient {
        let http_client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}", token);
        TelegramClient {
            http_client: http_client,
            base_url: url,
        }
    }

    pub async fn get_name(&self) -> Result<String, TelegramError> {
        let url = format!("{}/getMyName", self.base_url);
        let response = self.http_client.get(&url).send().await?;
        let response_text = response.text().await?;
        response_text
            .parse::<serde_json::Value>()
            .map_err(|e| TelegramError::Fatal(e.to_string()))?
            .get("result")
            .ok_or_else(|| TelegramError::Fatal("No result in response".to_string()))?
            .get("name")
            .ok_or_else(|| TelegramError::Fatal("No name in response".to_string()))?
            .as_str()
            .ok_or_else(|| TelegramError::Fatal("name is not a string".to_string()))
            .map(|s| s.to_string())
    }

    pub async fn send_html_message(
        &self,
        chat_id: &str,
        html: &str,
    ) -> Result<TelegramMessage, TelegramError> {
        let params = [("chat_id", chat_id), ("text", html), ("parse_mode", "HTML")];
        let url = format!("{}/sendMessage", self.base_url);

        let response = self.http_client.post(&url).form(&params).send().await?;
        let response_text = response.text().await?;

        let response_json = response_text
            .parse::<serde_json::Value>()
            .map_err(|e| TelegramError::Fatal(e.to_string()))?;

        // If telegram gets an error we get a response something like this
        // {\"ok\":false,\"error_code\":400,\"description\":\"Bad Request: chat not found\"}
        if let Some(error_code) = response_json.get("error_code") {
            let error_message = match response_json.get("description") {
                Some(description) => description.to_string(),
                None => format!(
                    "Error: {} - Description missing in telegram API response",
                    error_code
                ),
            };

            return Err(TelegramError::Fatal(error_message));
        }
        // Otherwise we should have a json something like this:
        // "{\"ok\":true,\"result\":{\"message_id\":23,\"from\":{\"id\":6544022299,\"is_bot\":true,\"first_name\":\"jmb-notify\",\"username\":\"jmb_notify_bot\"},\"chat\":{\"id\":-914917543,\"title\":\"James & jmb-notify\",\"type\":\"group\",\"all_members_are_administrators\":true},\"date\":1691470973,\"text\":\"This is a test message from notify\"}}"
        // We want to make a TelegramMessage from this 'result' object
        let message: TelegramMessage = match response_json.get("result") {
            Some(result) => serde_json::from_value(result.to_owned()).map_err(|e| {
                TelegramError::Fatal(format!(
                    "Unable to interpret telegram response. {}",
                    e.to_string()
                ))
            })?,
            None => return Err(TelegramError::Fatal("No result in response".to_string())),
        };

        Ok(message)
    }
}

#[cfg(test)]
#[cfg(feature = "telegram-tests")]
mod test {
    use super::*;

    fn get_telegram_token_from_env() -> String {
        std::env::var("TELEGRAM_TOKEN")
            .expect("Please set the TELEGRAM_TOKEN environment variable to run the telegram tests")
    }

    fn get_telegram_chat_id_from_env() -> String {
        std::env::var("TELEGRAM_CHAT_ID").expect(
            "Please set the TELEGRAM_CHAT_ID environment variable to run the telegram tests",
        )
    }

    #[tokio::test]
    async fn test_get_name() {
        let client = TelegramClient::new(get_telegram_token_from_env());
        let name = client.get_name().await;
        if !name.is_ok() {
            println!(
                "Unable to get name of bot, your environment might not be setup: {:?}",
                name
            );
        }

        assert!(name.is_ok());
        println!("My name is {}", name.unwrap());
    }

    #[tokio::test]
    async fn test_send_html_message() {
        let client = TelegramClient::new(get_telegram_token_from_env());
        client
            .send_html_message(
                &get_telegram_chat_id_from_env(),
                "<a href=\"https://www.msupply.foundation\">Visit the mSupply Foundation Website</a>",
            )
            .await
            .unwrap();
    }
}
