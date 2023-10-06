use chrono::Utc;
use repository::{
    NotificationEventRow, NotificationEventRowRepository, NotificationEventStatus,
    NotificationType, RecipientRow,
};
use serde::Serialize;
use tera::{Context, Tera};
use util::uuid::uuid;

use crate::service_provider::ServiceContext;

use super::NotificationServiceError;

// This struct is intended to be able to be created by a plugin from a datasource, and defines what a template can expect from a recipient
// Often it will be derived RecipientRow which is why we implement From<RecipientRow> for NotificationRecipient
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NotificationTarget {
    pub name: String,
    pub to_address: String,
    pub notification_type: NotificationType,
}

impl From<RecipientRow> for NotificationTarget {
    fn from(recipient: RecipientRow) -> Self {
        NotificationTarget {
            name: recipient.name,
            notification_type: recipient.notification_type.into(),
            to_address: recipient.to_address,
        }
    }
}

#[derive(Debug)]
pub enum TemplateDefinition {
    TemplateName(String),
    Template(String),
}

#[derive(Debug)]
pub struct NotificationContext {
    pub title_template: Option<TemplateDefinition>,
    pub body_template: TemplateDefinition,
    pub recipients: Vec<NotificationTarget>,
    pub template_data: serde_json::Value,
}

pub fn create_notification_events(
    ctx: &ServiceContext,
    config_id: Option<String>,
    notification: NotificationContext,
) -> Result<(), NotificationServiceError> {
    let repo = NotificationEventRowRepository::new(&ctx.connection);

    // TODO: Should this function use a notification config to get the template, users, etc?

    // Create a tera instance for this notification
    let mut tera = Tera::default();
    // Add any configured templates to out new tera instance
    tera.extend(ctx.service_provider.notification_service.tera())
        .map_err(|e| {
            NotificationServiceError::InternalError(format!(
                "Failed to extend tera instance with notification service templates: {:?}",
                e
            ))
        })?;

    let title_template_name = match notification.title_template {
        Some(TemplateDefinition::TemplateName(title_template_name)) => title_template_name,
        Some(TemplateDefinition::Template(title_template)) => {
            // Add the title template to the tera instance
            tera.add_raw_template("title_template", &title_template)
                .map_err(|e| {
                    NotificationServiceError::InternalError(format!(
                        "Failed to add title template to tera instance: {:?}",
                        e
                    ))
                })?;
            "title_template".to_string()
        }
        None => "default/title.html".to_string(),
    };

    let body_template_name = match notification.body_template {
        TemplateDefinition::TemplateName(body_template_name) => body_template_name,
        TemplateDefinition::Template(body_template) => {
            // Add the body template to the tera instance
            tera.add_raw_template("body_template", &body_template)
                .map_err(|e| {
                    NotificationServiceError::InternalError(format!(
                        "Failed to add body template to tera instance: {:?}",
                        e
                    ))
                })?;
            "body_template".to_string()
        }
    };

    let mut tera_context = Context::from_value(notification.template_data)?;

    // Loop through recipients and create a notification for each
    for recipient in notification.recipients {
        let notification_type = recipient.notification_type.clone();

        // Replace the recipient data in the template context
        tera_context.insert("recipient", &recipient);

        let base_row = NotificationEventRow {
            id: uuid(),
            to_address: recipient.to_address,
            created_at: Utc::now().naive_utc(),
            sent_at: None,
            error_message: None,
            send_attempts: 0,
            updated_at: Utc::now().naive_utc(),
            notification_config_id: config_id.clone(),
            notification_type,
            retry_at: None,
            ..Default::default()
        };

        let base_row_with_title = match tera.render(&title_template_name, &tera_context) {
            Ok(title) => NotificationEventRow {
                title: Some(title),
                ..base_row
            },
            Err(e) => {
                log::error!("Failed to render notification title template: {:?}", e);
                NotificationEventRow {
                    status: NotificationEventStatus::Errored,
                    error_message: Some(format!("{:?}", e)),
                    ..base_row
                }
            }
        };

        let notification_queue_row = match tera.render(&body_template_name, &tera_context) {
            Ok(body) => NotificationEventRow {
                message: body,
                ..base_row_with_title
            },
            Err(e) => {
                log::error!("Failed to render notification body template: {:?}", e);
                NotificationEventRow {
                    status: NotificationEventStatus::Errored,
                    error_message: Some(format!("{:?}", e)),
                    ..base_row_with_title
                }
            }
        };

        repo.insert_one(&notification_queue_row)
            .map_err(|e| NotificationServiceError::DatabaseError(e))?;

        // TODO: trigger async notification send?
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use repository::{
        mock::MockDataInserts, test_db::setup_all, NotificationEventRowRepository, NotificationType,
    };

    use crate::{
        notification::enqueue::{
            create_notification_events, NotificationContext, NotificationTarget, TemplateDefinition,
        },
        service_provider::{ServiceContext, ServiceProvider},
        test_utils::get_test_settings,
    };

    #[actix_rt::test]
    async fn test_create_notification_events_email() {
        let (_, _, connection_manager, _) = setup_all(
            "test_create_notification_events_email",
            MockDataInserts::none(),
        )
        .await;

        let connection = connection_manager.connection().unwrap();
        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));
        let context = ServiceContext::as_server_admin(service_provider).unwrap();

        let result = create_notification_events(
            &context,
            None,
            NotificationContext {
                title_template: Some(TemplateDefinition::TemplateName(
                    "test_message/email_subject.html".to_string(),
                )),
                body_template: TemplateDefinition::TemplateName(
                    "test_message/email.html".to_string(),
                ),
                recipients: vec![NotificationTarget {
                    name: "test".to_string(),
                    to_address: "test@example.com".to_string(),
                    notification_type: NotificationType::Email,
                }],
                template_data: serde_json::json!({}),
            },
        );

        assert!(result.is_ok());

        // Check we have a notification event
        let notification_event_row_repository = NotificationEventRowRepository::new(&connection);
        let notification_event_rows = notification_event_row_repository.un_sent().unwrap();

        assert_eq!(notification_event_rows.len(), 1);
        assert_eq!(
            notification_event_rows[0].to_address,
            "test@example.com".to_string()
        );
        assert!(notification_event_rows[0].title.is_some());
    }

    #[actix_rt::test]
    async fn test_create_notification_events_telegram() {
        let (_, _, connection_manager, _) = setup_all(
            "test_create_notification_events_telegram",
            MockDataInserts::none(),
        )
        .await;

        let connection = connection_manager.connection().unwrap();
        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));
        let context = ServiceContext::as_server_admin(service_provider).unwrap();

        let result = create_notification_events(
            &context,
            None,
            NotificationContext {
                title_template: None,
                body_template: TemplateDefinition::TemplateName(
                    "test_message/telegram.html".to_string(),
                ),
                recipients: vec![NotificationTarget {
                    name: "telegram".to_string(),
                    to_address: "-12345".to_string(),
                    notification_type: NotificationType::Telegram,
                }],
                template_data: serde_json::json!({}),
            },
        );

        assert!(result.is_ok());

        // Check we have a notification event with no title but does have a message
        let notification_event_row_repository = NotificationEventRowRepository::new(&connection);
        let notification_event_rows = notification_event_row_repository.un_sent().unwrap();

        assert_eq!(notification_event_rows.len(), 1);
        assert_eq!(notification_event_rows[0].to_address, "-12345".to_string());
        assert_ne!(notification_event_rows[0].message, "");
    }
}
