use crate::service_provider::ServiceContext;
use crate::settings::Settings;
use async_trait::async_trait;
use chrono::{ Utc, Duration };
use lettre::address::AddressError;
use repository::{
    NotificationEventRowRepository, NotificationEventStatus, NotificationType, RepositoryError,
};
use serde_json::json;
use telegram::TelegramError;
use tera::Tera;

pub mod enqueue;
pub mod renderer;

pub static MAX_SEND_ATTEMPTS: i32 = 3;
pub static RETRY_DELAY_MINUTES: i64 = 15; // Doubles each retry

// We use a trait for NotificationService to allow mocking in tests
#[async_trait(?Send)]
pub trait NotificationServiceTrait: Send + Sync {
    fn render(
        &self,
        template_name: &str,
        params: &serde_json::Value,
    ) -> Result<String, NotificationServiceError>;

    fn render_no_params(&self, template_name: &str) -> Result<String, NotificationServiceError>;

    fn tera(&self) -> &Tera;

    async fn send_queued_notifications(
        &self,
        ctx: &ServiceContext,
    ) -> Result<usize, NotificationServiceError>;
}

pub struct NotificationService {
    pub tera: Tera,
}

#[derive(Debug)]
pub enum NotificationServiceError {
    InternalError(String),
    RenderError(tera::Error),
    AddressError(AddressError),
    LettreError(lettre::error::Error),
    SmtpError(lettre::transport::smtp::Error),
    DatabaseError(RepositoryError),
}

impl From<RepositoryError> for NotificationServiceError {
    fn from(error: RepositoryError) -> Self {
        NotificationServiceError::DatabaseError(error)
    }
}

impl From<tera::Error> for NotificationServiceError {
    fn from(error: tera::Error) -> Self {
        NotificationServiceError::RenderError(error)
    }
}

impl NotificationService {
    pub fn new(settings: Settings) -> Self {
        let template_path = match settings.server.base_dir {
            Some(base_dir) => format!("{}/templates/**/*", base_dir),
            None => "templates/**/*".to_string(), // Assume base dir relative to current dir
        };
        let tera = Tera::new(&template_path)
            .expect(format!("Unable to create tera with path {}", template_path).as_str());

        NotificationService { tera }
    }
}

#[async_trait(?Send)]
impl NotificationServiceTrait for NotificationService {
    fn render(
        &self,
        template_name: &str,
        params: &serde_json::Value,
    ) -> Result<String, NotificationServiceError> {
        Ok(renderer::render_template(
            &self.tera,
            template_name,
            params,
        )?)
    }

    fn render_no_params(&self, template_name: &str) -> Result<String, NotificationServiceError> {
        Ok(renderer::render_template(
            &self.tera,
            template_name,
            json!({}),
        )?)
    }
    fn tera(&self) -> &Tera {
        &self.tera
    }

    async fn send_queued_notifications(
        &self,
        ctx: &ServiceContext,
    ) -> Result<usize, NotificationServiceError> {
        log::debug!("Sending queued notifications");

        let repo = NotificationEventRowRepository::new(&ctx.connection);
        let queued_notifications = repo.un_sent()?;

        let mut error_count = 0;
        let mut sent_count = 0;

        for mut notification in queued_notifications {
            match notification.notification_type {
                NotificationType::Unknown => {
                    // This should only happen with a misconfigured sql recipient list query.
                    // If you get this error in the logs you need to fix the sql query.
                    error_count += 1;
                    log::error!(
                        "Unknown Notification Type {} to {} !!!!!",
                        notification.id,
                        notification.to_address,
                    );
                    notification.error_message = Some(format!(
                        "Unknown Notification Type for address {}",
                        notification.to_address,
                    ));
                    notification.status = NotificationEventStatus::Failed;

                    repo.update_one(&notification)?;
                }
                NotificationType::Email => {
                    // Try to send via email
                    let text_body = notification.message.clone();
                    let parser = pulldown_cmark::Parser::new(&notification.message);
                    let mut email_body = String::new();
                    pulldown_cmark::html::push_html(&mut email_body, parser);

                    let result = ctx.service_provider.email_service.send_email(
                        notification.to_address.clone(),
                        notification
                            .title
                            .clone()
                            .unwrap_or("Notification".to_string()),
                        email_body,
                        text_body,
                    );

                    match result {
                        Ok(_) => {
                            // Successfully Sent
                            notification.error_message = None;
                            notification.status = NotificationEventStatus::Sent;
                            notification.send_attempts += 1;
                            notification.sent_at = Some(Utc::now().naive_utc());
                            notification.updated_at = Utc::now().naive_utc();
                            repo.update_one(&notification)?;
                            sent_count += 1;
                        }
                        Err(send_error) => {
                            // Failed to send
                            notification.updated_at = Utc::now().naive_utc();
                            notification.send_attempts += 1;
                            if notification.send_attempts >= MAX_SEND_ATTEMPTS {
                                log::error!(
                                    "Failed to send email {} to {} after {} attempts - {:?}",
                                    notification.id,
                                    notification.to_address,
                                    MAX_SEND_ATTEMPTS,
                                    send_error
                                );
                                notification.error_message = Some(format!(
                                    "Failed to send email after {} attempts - {:?}",
                                    MAX_SEND_ATTEMPTS, send_error
                                ));
                                notification.status = NotificationEventStatus::Failed;
                            } else if send_error.is_permanent() {
                                log::error!(
                                    "Permanently failed to send email {} to {}",
                                    notification.id,
                                    notification.to_address,
                                );
                                notification.error_message = Some(format!("{:?}", send_error));
                                notification.status = NotificationEventStatus::Failed;
                            } else {
                                log::error!(
                                    "Temporarily unable to send email {} to {} - {:?}",
                                    notification.id,
                                    notification.to_address,
                                    send_error
                                );
                                notification.error_message = Some(format!("{:?}", send_error));
                                notification.status = NotificationEventStatus::Errored;
                                notification.retry_at = Some(
                                    Utc::now().naive_utc() +
                                    Duration::minutes(
                                        RETRY_DELAY_MINUTES *
                                        i64::pow(2, notification.send_attempts as u32 - 1)
                                    )
                                )
                            }
                            error_count += 1;
                            repo.update_one(&notification)?;
                            continue;
                        }
                    }
                }
                NotificationType::Telegram => {
                    // Try to send via telegram
                    if let Some(telegram) = &ctx.service_provider.telegram {
                        let telegram_markdown_v2 =
                            telegram::service::markdown::cmark_to_telegram_v2(
                                &notification.message,
                            );

                        let result = telegram
                            .send_markdown_message(&notification.to_address, &telegram_markdown_v2)
                            .await;

                        match result {
                            Ok(_) => {
                                log::info!("Sent telegram message to {}", notification.to_address);
                                notification.error_message = None;
                                notification.status = NotificationEventStatus::Sent;
                                notification.send_attempts += 1;
                                notification.sent_at = Some(Utc::now().naive_utc());
                                notification.updated_at = Utc::now().naive_utc();
                                repo.update_one(&notification)?;
                                sent_count += 1;
                            }
                            Err(TelegramError::Fatal(e)) => {
                                log::error!(
                                    "Permanently fail to send telegram message to {}: {:?}",
                                    notification.to_address,
                                    e
                                );
                                notification.send_attempts += 1;
                                notification.error_message = Some(format!("{:?}", e));
                                notification.status = NotificationEventStatus::Failed;
                                notification.updated_at = Utc::now().naive_utc();
                                repo.update_one(&notification)?;
                                error_count += 1;
                            }
                            Err(TelegramError::Temporary(e)) => {
                                notification.send_attempts += 1;
                                if notification.send_attempts >= MAX_SEND_ATTEMPTS {
                                    log::error!(
                                    "Failed to send telegram message {} to {} after {} attempts - {:?}",
                                    notification.id,
                                    notification.to_address,
                                    MAX_SEND_ATTEMPTS,
                                    e
                                );
                                    notification.error_message = Some(format!("{:?}", e));
                                    notification.status = NotificationEventStatus::Failed;
                                } else {
                                    log::error!(
                                    "Temporarily unable to send telegram message {} to {} - {:?}",
                                    notification.id,
                                    notification.to_address,
                                    e
                                );
                                    notification.error_message = Some(format!("{:?}", e));
                                    notification.status = NotificationEventStatus::Errored;
                                }

                                notification.updated_at = Utc::now().naive_utc();
                                repo.update_one(&notification)?;
                                error_count += 1;
                            }
                        }
                    } else {
                        log::error!(
                            "Telegram not configured, you are missing telegram notifications!!!!"
                        );
                        notification.error_message = Some("Telegram Not Configured".to_string());
                        notification.status = NotificationEventStatus::Errored;
                        notification.updated_at = Utc::now().naive_utc();
                        repo.update_one(&notification)?;
                        error_count += 1;
                    }
                }
            }
        }

        log::debug!("Sent {} notifications, {} errors", sent_count, error_count);

        Ok(sent_count)
    }
}
