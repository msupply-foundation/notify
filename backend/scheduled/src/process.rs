use std::collections::HashMap;
use tera::{Context, Tera};

use chrono::{DateTime, NaiveDateTime, Utc};
use repository::{NotificationConfigKind, NotificationConfigRowRepository};
use service::{
    notification::enqueue::{create_notification_events, NotificationContext, TemplateDefinition},
    notification_config::{
        parameters::get_notification_parameters, query::NotificationConfig,
        recipients::get_notification_targets,
    },
    service_provider::ServiceContext,
};

use crate::{
    parse::ScheduledNotificationPluginConfig, query::get_notification_query_results,
    NotificationError,
};

pub fn process_scheduled_notifications(
    ctx: &ServiceContext,
    current_time: NaiveDateTime,
) -> Result<usize, NotificationError> {
    log::info!("Processing scheduled notifications due at {}", current_time);

    // Check if any scheduled notifications are due according to the database
    let scheduled_notifications = ctx
        .service_provider
        .notification_config_service
        .find_all_due_by_kind(ctx, NotificationConfigKind::Scheduled, current_time)
        .map_err(|e| NotificationError::InternalError(format!("{:?}", e)))?;
    let notifications_processed = scheduled_notifications.len();
    let mut successful_notifications = 0;
    let mut errored_notifications = 0;
    let mut skipped_notifications = 0;
    for scheduled_notification in scheduled_notifications {
        let start_time = Utc::now();
        let notification_name = scheduled_notification.title.clone();
        log::info!(
            "Processing scheduled notification: {} - {}",
            scheduled_notification.title,
            scheduled_notification.id,
        );

        match try_process_scheduled_notifications(ctx, scheduled_notification, current_time) {
            Err(e) => {
                log::error!("{:?}", e);
                errored_notifications += 1;
            }
            Ok(ProcessingResult::Skipped(message)) => {
                log::info!("{}", message);
                skipped_notifications += 1;
            }
            Ok(ProcessingResult::Success) => {
                log::info!("Successfully created notification events");
                successful_notifications += 1;
            }
        }
        let end_time = Utc::now();
        log::info!(
            "Processed {} Notification in {}s",
            notification_name,
            (end_time - start_time).num_seconds()
        );
    }
    // Return the number of notifications processed
    log::info!(
        "Processed {} out of {} scheduled notifications, skipped {} and errored {}",
        successful_notifications,
        notifications_processed,
        skipped_notifications,
        errored_notifications
    );
    Ok(notifications_processed)
}

#[derive(Debug, PartialEq)]
enum ProcessingResult {
    Success,
    Skipped(String),
}

fn try_process_scheduled_notifications(
    ctx: &ServiceContext,
    scheduled_notification: NotificationConfig,
    now: NaiveDateTime,
) -> Result<ProcessingResult, NotificationError> {
    // Load the notification config
    let config =
        ScheduledNotificationPluginConfig::from_string(&scheduled_notification.configuration_data)?;

    let previous_due_datetime = scheduled_notification.next_due_datetime;

    // Get next notification due date
    let next_due_datetime = config.next_due_date(DateTime::from_utc(now, Utc))?;

    // Update the last_checked time and next_check time
    // We do this before checking if the notification is due so that if the notification is skipped, we still set a good next check time
    NotificationConfigRowRepository::new(&ctx.connection)
        .set_last_run_by_id(
            &scheduled_notification.id,
            now,
            Some(next_due_datetime.naive_utc()),
        )
        .map_err(|e| NotificationError::InternalError(format!("{:?}", e)))?;

    // Should notification run ?
    let previous_due_datetime = match previous_due_datetime {
        Some(dt) => dt,
        None => {
            return Ok(ProcessingResult::Skipped(format!(
                "No next due time for scheduled notification {}, setting to {}",
                scheduled_notification.id, next_due_datetime
            )));
        }
    };

    if previous_due_datetime > now {
        return Ok(ProcessingResult::Skipped(format!(
            "Scheduled notification {} is not due yet, skipping",
            scheduled_notification.id
        )));
    }

    let param_results = get_notification_parameters(ctx, &scheduled_notification);
    let mut all_params = match param_results {
        Ok(val) => val,
        Err(e) => {
            return Err(NotificationError::InternalError(format!(
                "Failed to fetch parameters: {:?}",
                e
            )))
        }
    };

    if all_params.len() == 0 {
        // If no parameters are provided, create a single empty parameter set
        all_params = vec![HashMap::new()];
    }

    for mut template_params in all_params {
        // Put sql queries and appropriate data into Json Value for template
        let sql_params = serde_json::to_value(&template_params).map_err(|e| {
            NotificationError::InternalError(format!("Failed to parse sql params data: {:?}", e))
        })?;

        log::info!("Processing parameter set: {}", sql_params);

        // Get the recipients
        let notification_targets = get_notification_targets(
            ctx,
            &scheduled_notification,
            sql_params.clone(),
        )
        .map_err(|e| {
            NotificationError::InternalError(format!("Failed to get notification targets: {:?}", e))
        })?;

        // If there are no recipients, skip this parameter set
        if notification_targets.is_empty() {
            log::info!("No notification targets, skipping");
            continue;
        }

        let sql_query_parameters = get_notification_query_results(ctx, sql_params, &config)?;

        // Template data should include the notification config parameters, plus the results of any queries
        template_params.extend(sql_query_parameters);

        let template_data = serde_json::to_value(template_params).map_err(|e| {
            NotificationError::InternalError(format!("Failed to parse template data: {:?}", e))
        })?;

        // Check if the notification conditions are met
        if config.conditional {
            // Create a tera instance for this notification

            let tera_context = Context::from_value(template_data.clone()).map_err(|e| {
                NotificationError::InternalError(format!(
                    "Failed to convert template data to tera context: {}",
                    e.to_string()
                ))
            })?;

            let condition_template_result =
                Tera::one_off(&config.condition_template, &tera_context, false).map_err(|e| {
                    NotificationError::InternalError(format!(
                        "Failed to render condition template: {:?}",
                        e
                    ))
                })?;

            let condition_met = condition_template_result.contains("true")
                && !condition_template_result.contains("false");

            if !condition_met {
                log::info!("Notification condition is false, skipping");
                continue;
            }
        }

        // Send the notification
        let notification = NotificationContext {
            title_template: Some(TemplateDefinition::Template(
                config.subject_template.clone(),
            )),
            body_template: TemplateDefinition::Template(config.body_template.clone()),
            template_data: template_data,
            recipients: notification_targets,
        };

        create_notification_events(ctx, Some(scheduled_notification.id.clone()), notification)
            .map_err(|e| NotificationError::InternalError(format!("{:?}", e)))?;
    }

    Ok(ProcessingResult::Success)
}

#[cfg(test)]
mod test {

    use std::sync::Arc;

    use chrono::Days;
    use repository::mock::{
        mock_notification_query_with_params, mock_recipient_a,
        mock_recipient_list_with_recipient_members_a_and_b, mock_sql_recipient_list_with_no_param,
        mock_sql_recipient_list_with_param,
    };
    use repository::NotificationEventRowRepository;
    use repository::{mock::MockDataInserts, test_db::setup_all};
    use service::test_utils::email_test::send_test_emails;
    use service::test_utils::get_test_settings;

    use service::service_provider::ServiceProvider;
    use service::test_utils::telegram_test::send_test_notifications;

    use super::*;

    #[tokio::test]
    async fn test_process_scheduled_notifications() {
        let (_, _, connection_manager, _) =
            setup_all("process_scheduled_notifications", MockDataInserts::none()).await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));

        // Check it runs with no scheduled notifications
        let result = process_scheduled_notifications(
            &ServiceContext::new(service_provider).unwrap(),
            chrono::Utc::now().naive_utc(),
        )
        .unwrap();

        assert_eq!(result, 0);

        // TODO: More tests!

        // Test that it runs with a scheduled notification with no due date
        // Test that it skips a scheduled notification with a due date in the future
        // Test that it runs a scheduled notification with a due date in the past
    }

    // Test that we get a notification when we have a recipient list configured
    #[tokio::test]
    async fn test_try_process_scheduled_notifications_with_recipient_list() {
        let (_, _, connection_manager, _) = setup_all(
            "test_try_process_scheduled_notifications_with_recipient_list",
            MockDataInserts::none()
                .recipients()
                .recipient_lists()
                .recipient_list_members(),
        )
        .await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));

        let service_context = ServiceContext::new(service_provider).unwrap();

        // Daily Scheduled Notification that started this time yesterday
        let sch_config = ScheduledNotificationPluginConfig {
            body_template: "TestTemplate - Markdown \n- item 1\n- item 2".to_string(),
            subject_template: "TestSubject - Recipient List - Markdown".to_string(),
            schedule_frequency: "daily".to_string(),
            schedule_start_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            ..Default::default()
        };

        // Create a notification config with a recipient list
        let notification_config = NotificationConfig {
            id: "notification_config_1".to_string(),
            kind: NotificationConfigKind::Scheduled,
            recipient_ids: vec![],
            recipient_list_ids: vec![mock_recipient_list_with_recipient_members_a_and_b().id],
            sql_recipient_list_ids: vec![],
            next_due_datetime: Some(chrono::Utc::now().naive_utc()),
            configuration_data: serde_json::to_string(&sch_config).unwrap(),
            ..Default::default()
        };

        // Try to process the notification (Should be due now)
        let result = try_process_scheduled_notifications(
            &service_context,
            notification_config,
            chrono::Utc::now().naive_utc(),
        )
        .unwrap();

        assert_eq!(result, ProcessingResult::Success);

        // Query the database to check that we have a notification events for each recipient
        let repo = NotificationEventRowRepository::new(&service_context.connection);
        let notification_events = repo.un_sent().unwrap();

        assert_eq!(notification_events.len(), 2);

        send_test_notifications(&service_context).await;
    }

    // Test that we get a notification when we have a recipient configured
    #[tokio::test]
    async fn test_try_process_scheduled_notifications_with_recipient() {
        let (_, _, connection_manager, _) = setup_all(
            "test_try_process_scheduled_notifications_with_recipient",
            MockDataInserts::none().recipients(),
        )
        .await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));

        let service_context = ServiceContext::new(service_provider).unwrap();

        // Daily Scheduled Notification that started this time yesterday
        let sch_config = ScheduledNotificationPluginConfig {
            body_template: "TestTemplate".to_string(),
            subject_template: "TestSubject".to_string(),
            schedule_frequency: "daily".to_string(),
            schedule_start_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            ..Default::default()
        };

        // Create a notification config with a recipient list
        let notification_config = NotificationConfig {
            id: "notification_config_1".to_string(),
            kind: NotificationConfigKind::Scheduled,
            recipient_ids: vec![mock_recipient_a().id],
            recipient_list_ids: vec![],
            sql_recipient_list_ids: vec![],
            next_due_datetime: Some(chrono::Utc::now().naive_utc()),
            configuration_data: serde_json::to_string(&sch_config).unwrap(),
            ..Default::default()
        };

        // Try to process the notification (Should be due now)
        let result = try_process_scheduled_notifications(
            &service_context,
            notification_config,
            chrono::Utc::now().naive_utc(),
        )
        .unwrap();

        assert_eq!(result, ProcessingResult::Success);

        // Query the database to check that we have a notification events for the 1 configured recipient
        let repo = NotificationEventRowRepository::new(&service_context.connection);
        let notification_events = repo.un_sent().unwrap();

        assert_eq!(notification_events.len(), 1);
    }

    // Test that we get a notification when we have a sql recipient list configured
    #[tokio::test]
    async fn test_try_process_scheduled_notifications_with_sql_recipients() {
        let (_, _, connection_manager, _) = setup_all(
            "test_try_process_scheduled_notifications_with_sql_recipients",
            MockDataInserts::none().sql_recipient_lists(),
        )
        .await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));

        let service_context = ServiceContext::new(service_provider).unwrap();

        // Daily Scheduled Notification that started this time yesterday
        let sch_config = ScheduledNotificationPluginConfig {
            body_template: "TestTemplate".to_string(),
            subject_template: "TestSubject".to_string(),
            schedule_frequency: "daily".to_string(),
            schedule_start_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            ..Default::default()
        };

        // Create a notification config with 2 sql recipient lists
        let notification_config = NotificationConfig {
            id: "notification_config_1".to_string(),
            kind: NotificationConfigKind::Scheduled,
            recipient_ids: vec![],
            recipient_list_ids: vec![],
            sql_recipient_list_ids: vec![
                mock_sql_recipient_list_with_no_param().id,
                mock_sql_recipient_list_with_param().id,
            ],
            parameters: "[{\"email_address\":\"test-user@example.com\"}]".to_string(),
            next_due_datetime: Some(chrono::Utc::now().naive_utc()),
            configuration_data: serde_json::to_string(&sch_config).unwrap(),
            ..Default::default()
        };

        // Try to process the notification (Should be due now)
        let result = try_process_scheduled_notifications(
            &service_context,
            notification_config,
            chrono::Utc::now().naive_utc(),
        )
        .unwrap();

        assert_eq!(result, ProcessingResult::Success);

        // Query the database to check that we have a notification events
        let repo = NotificationEventRowRepository::new(&service_context.connection);
        let notification_events = repo.un_sent().unwrap();

        assert_eq!(notification_events.len(), 2);
    }

    // Test that we get a notification when we have a sql recipient list & notification query
    #[tokio::test]
    async fn test_try_process_scheduled_notifications_with_queries() {
        let (_, _, connection_manager, _) = setup_all(
            "test_try_process_scheduled_notifications_with_queries",
            MockDataInserts::none()
                .sql_recipient_lists()
                .notification_queries(),
        )
        .await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));

        let service_context = ServiceContext::new(service_provider).unwrap();

        // Daily Scheduled Notification that started this time yesterday
        let sch_config = ScheduledNotificationPluginConfig {
            body_template: "Sensor Limit {{ sensor_limit }}, Latest Temperature: {{ latest_temperature }}, is over limit ? {{ query1.0.is_above_limit }}".to_string(),
            subject_template: "Sensor Data".to_string(),
            schedule_frequency: "daily".to_string(),
            schedule_start_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            notification_query_ids: vec![mock_notification_query_with_params().id],
            ..Default::default()
        };

        // Create a notification config with 2 sql recipient lists
        let notification_config = NotificationConfig {
            id: "notification_config_1".to_string(),
            kind: NotificationConfigKind::Scheduled,
            recipient_ids: vec![],
            recipient_list_ids: vec![],
            sql_recipient_list_ids: vec![
                mock_sql_recipient_list_with_no_param().id,
                mock_sql_recipient_list_with_param().id,
            ],
            parameters: "[{\"email_address\":\"test-user@example.com\",\"sensor_limit\":\"8\",\"latest_temperature\":\"8.5\"}]".to_string(),
            next_due_datetime: Some(chrono::Utc::now().naive_utc()),
            configuration_data: serde_json::to_string(&sch_config).unwrap(),
            ..Default::default()
        };

        // Try to process the notification (Should be due now)
        let result = try_process_scheduled_notifications(
            &service_context,
            notification_config,
            chrono::Utc::now().naive_utc(),
        )
        .unwrap();

        assert_eq!(result, ProcessingResult::Success);

        // Query the database to check that we have a notification events
        let repo = NotificationEventRowRepository::new(&service_context.connection);
        let notification_events = repo.un_sent().unwrap();

        assert_eq!(notification_events.len(), 2);

        assert_eq!(
            notification_events[0].message,
            "Sensor Limit 8, Latest Temperature: 8.5, is over limit ? true"
        );

        send_test_notifications(&service_context).await;
        send_test_emails(&service_context);
    }

    // Test that we don't send notifications if template fails to render
    #[tokio::test]
    async fn test_try_process_scheduled_notifications_with_template_error() {
        let (_, _, connection_manager, _) = setup_all(
            "test_try_process_scheduled_notifications_with_template_error",
            MockDataInserts::none()
                .sql_recipient_lists()
                .notification_queries(),
        )
        .await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));

        let service_context = ServiceContext::new(service_provider).unwrap();

        // Daily Scheduled Notification that started this time yesterday
        let sch_config = ScheduledNotificationPluginConfig {
            body_template: "Sensor Limit {{ sensor_limit }}, Latest Temperature: {{ latest_temperature }}, is over limit ? {{ query1.is_above_limit }}".to_string(), // This will cause a template render error, Need to get the first query row with .0
            subject_template: "Sensor Data".to_string(),
            schedule_frequency: "daily".to_string(),
            schedule_start_time: Utc::now().checked_sub_days(Days::new(1)).unwrap(),
            notification_query_ids: vec![mock_notification_query_with_params().id],
            ..Default::default()
        };

        // Create a notification config with 2 sql recipient lists
        let notification_config = NotificationConfig {
            id: "notification_config_1".to_string(),
            kind: NotificationConfigKind::Scheduled,
            recipient_ids: vec![],
            recipient_list_ids: vec![],
            sql_recipient_list_ids: vec![
                mock_sql_recipient_list_with_no_param().id,
                mock_sql_recipient_list_with_param().id,
            ],
            parameters: "[{\"email_address\":\"test-user@example.com\",\"sensor_limit\":\"8\",\"latest_temperature\":\"8.5\"}]".to_string(),
            next_due_datetime: Some(chrono::Utc::now().naive_utc()),
            configuration_data: serde_json::to_string(&sch_config).unwrap(),
            ..Default::default()
        };

        // Try to process the notification (Should be due now)
        let result = try_process_scheduled_notifications(
            &service_context,
            notification_config,
            chrono::Utc::now().naive_utc(),
        )
        .unwrap();

        assert_eq!(result, ProcessingResult::Success); // TODO: This shouldn't be a success I think!

        // Query the database to check that we have no unsent notification events (There is a template error so nothing should be sent!)
        let repo = NotificationEventRowRepository::new(&service_context.connection);
        let notification_events = repo.un_sent().unwrap();

        assert_eq!(notification_events.len(), 0);
    }
}
