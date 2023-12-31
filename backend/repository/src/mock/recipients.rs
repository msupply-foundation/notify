use chrono::NaiveDateTime;

use crate::{NotificationType, RecipientRow};

pub fn mock_recipients() -> Vec<RecipientRow> {
    vec![
        mock_recipient_a(),
        mock_recipient_aa(),
        mock_recipient_b(),
        mock_recipient_c(),
        mock_recipient_d_deleted(),
    ]
}

pub fn mock_recipient_a() -> RecipientRow {
    RecipientRow {
        id: String::from("id_recipient_a"),
        name: String::from("recipient_a"),
        notification_type: NotificationType::Email,
        to_address: String::from("a@openmsupply.foundation"),
        deleted_datetime: None,
    }
}

pub fn mock_recipient_aa() -> RecipientRow {
    RecipientRow {
        id: String::from("id_recipient_aa"),
        name: String::from("recipient_aa"),
        notification_type: NotificationType::Email,
        to_address: String::from("aa@openmsupply.foundation"),
        deleted_datetime: None,
    }
}

pub fn mock_recipient_b() -> RecipientRow {
    RecipientRow {
        id: String::from("id_recipient_b"),
        name: String::from("recipient_b"),
        notification_type: NotificationType::Email,
        to_address: String::from("b@openmsupply.foundation"),
        deleted_datetime: None,
    }
}

pub fn mock_recipient_c() -> RecipientRow {
    RecipientRow {
        id: String::from("id_recipient_c"),
        name: String::from("recipient_c"),
        notification_type: NotificationType::Telegram,
        to_address: String::from("chat_id_c"),
        deleted_datetime: None,
    }
}

pub fn mock_recipient_d_deleted() -> RecipientRow {
    RecipientRow {
        id: String::from("id_recipient_d"),
        name: String::from("recipient_d"),
        notification_type: NotificationType::Email,
        to_address: String::from("d@openmsupply.foundation"),
        deleted_datetime: Some(
            NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ),
    }
}
