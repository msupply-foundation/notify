use chrono::Utc;
use repository::{
    LogType, NotificationConfigKind, NotificationConfigRow, NotificationConfigRowRepository,
    NotificationConfigStatus, StorageConnection,
};

use crate::{audit_log::audit_log_entry, service_provider::ServiceContext};

use super::{
    query::{get_notification_config, NotificationConfig},
    validate::check_notification_config_does_not_exist,
    ModifyNotificationConfigError,
};

#[derive(Clone)]
pub struct CreateNotificationConfig {
    pub id: String,
    pub title: String,
    pub kind: NotificationConfigKind,
}

pub fn create_notification_config(
    ctx: &ServiceContext,
    new_config: CreateNotificationConfig,
) -> Result<NotificationConfig, ModifyNotificationConfigError> {
    let notification_config = ctx
        .connection
        .transaction_sync(|connection| {
            validate(&new_config, connection)?;
            let new_config_row = generate(new_config.clone())?;

            NotificationConfigRowRepository::new(connection).insert_one(&new_config_row)?;

            get_notification_config(ctx, new_config_row.id)
                .map_err(ModifyNotificationConfigError::from)
        })
        .map_err(|error| error.to_inner_error())?;

    audit_log_entry(
        &ctx,
        LogType::NotificationConfigCreated,
        Some(new_config.id),
        Utc::now().naive_utc(),
    )?;

    Ok(notification_config)
}

pub fn validate(
    new_config: &CreateNotificationConfig,
    connection: &StorageConnection,
) -> Result<(), ModifyNotificationConfigError> {
    if !check_notification_config_does_not_exist(&new_config.id, connection)? {
        return Err(ModifyNotificationConfigError::NotificationConfigAlreadyExists);
    }
    Ok(())
}

pub fn generate(
    CreateNotificationConfig { id, title, kind }: CreateNotificationConfig,
) -> Result<NotificationConfigRow, ModifyNotificationConfigError> {
    Ok(NotificationConfigRow {
        id,
        title: title.trim().to_string(),
        kind,
        configuration_data: "{}".to_string(),
        status: NotificationConfigStatus::Disabled,
        parameters: "{}".to_string(),
        parameter_query_id: None,
        recipient_ids: "[]".to_string(),
        recipient_list_ids: "[]".to_string(),
        sql_recipient_list_ids: "[]".to_string(),
        last_run_datetime: None,
        next_due_datetime: None,
    })
}
