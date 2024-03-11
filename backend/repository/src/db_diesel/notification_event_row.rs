use super::StorageConnection;
use crate::{repository_error::RepositoryError, NotificationType};
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use notification_event::dsl as notification_event_dsl;

/*
CREATE TABLE
    IF NOT EXISTS notification_event (
        id TEXT PRIMARY KEY,
        notification_config_id TEXT NULL, -- Null as we allow plugins to create notifications without a config definition
        notification_type TEXT NOT NULL,
        to_address TEXT NOT NULL, -- Telegram chatid, email address, something else in future?
        title TEXT NULL, -- May be null as some plugins might not need a title, mainly this is needed for email subjects, but could be used for other purposes
        message TEXT NOT NULL,
        status TEXT NOT NULL,
        sent_at TIMESTAMP NULL,
        created_at TIMESTAMP NOT NULL,
        updated_at TIMESTAMP NOT NULL,
        retry_at TIMESTAMP NULL,
        send_attempts INTEGER NOT NULL DEFAULT 0,
        error_message TEXT NULL
    );
 */

table! {
    notification_event (id) {
        id -> Text,
        notification_config_id -> Nullable<Text>,
        notification_type -> crate::db_diesel::recipient_row::NotificationTypeMapping,
        to_address -> Text,
        title -> Nullable<Text>,
        message -> Text,
        status -> crate::db_diesel::notification_event_row::NotificationEventStatusMapping,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        sent_at -> Nullable<Timestamp>,
        retry_at -> Nullable<Timestamp>,
        send_attempts -> Integer,
        error_message -> Nullable<Text>,
        context -> Nullable<Text>,
    }
}

#[derive(DbEnum, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum NotificationEventStatus {
    #[default]
    Queued,
    Sent,
    Errored, // Errored will be re-tried
    Failed,  // Failed will not be re-tried
}

#[derive(
    Clone, Queryable, Insertable, Identifiable, Debug, PartialEq, Eq, AsChangeset, Default,
)]
#[table_name = "notification_event"]
pub struct NotificationEventRow {
    pub id: String,
    pub notification_config_id: Option<String>,
    pub notification_type: NotificationType,
    pub to_address: String,
    pub title: Option<String>,
    pub message: String,
    pub status: NotificationEventStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub sent_at: Option<chrono::NaiveDateTime>,
    pub retry_at: Option<chrono::NaiveDateTime>,
    pub send_attempts: i32,
    pub error_message: Option<String>,
    pub context: Option<String>, // JSON object, the tera context for the event
}

pub struct NotificationEventRowRepository<'a> {
    connection: &'a StorageConnection,
}

impl<'a> NotificationEventRowRepository<'a> {
    pub fn new(connection: &'a StorageConnection) -> Self {
        NotificationEventRowRepository { connection }
    }

    pub fn insert_one(
        &self,
        email_queue_row: &NotificationEventRow,
    ) -> Result<(), RepositoryError> {
        diesel::insert_into(notification_event_dsl::notification_event)
            .values(email_queue_row)
            .execute(&self.connection.connection)?;

        Ok(())
    }

    pub fn update_one(
        &self,
        email_queue_row: &NotificationEventRow,
    ) -> Result<(), RepositoryError> {
        diesel::update(email_queue_row)
            .set(email_queue_row)
            .execute(&self.connection.connection)?;

        Ok(())
    }

    pub fn un_sent(&self) -> Result<Vec<NotificationEventRow>, RepositoryError> {
        let result = notification_event_dsl::notification_event
            .filter(
                notification_event_dsl::status
                    .eq(NotificationEventStatus::Queued)
                    .or(notification_event_dsl::status.eq(NotificationEventStatus::Errored)
                        .and(notification_event_dsl::retry_at.le(diesel::dsl::now))),
            )
            .load::<NotificationEventRow>(&self.connection.connection)?;
        Ok(result)
    }
}
