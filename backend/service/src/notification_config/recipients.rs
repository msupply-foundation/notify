use repository::{
    EqualFilter, RecipientFilter, RecipientListMemberFilter, RecipientListMemberRepository,
    RecipientRepository,
};

use crate::{notification::enqueue::NotificationTarget, service_provider::ServiceContext};

use super::{query::NotificationConfig, ModifyNotificationConfigError};

pub fn get_notification_targets(
    ctx: &ServiceContext,
    notification_config: &NotificationConfig,
) -> Result<Vec<NotificationTarget>, ModifyNotificationConfigError> {
    let mut notification_targets: Vec<NotificationTarget> = Vec::new();

    // // Get the recipients based on recipient list ids
    let recipient_list_member_repository = RecipientListMemberRepository::new(&ctx.connection);

    // get the recipient list members
    let recipient_list_members = recipient_list_member_repository.query_by_filter(
        RecipientListMemberFilter::new().recipient_list_id(EqualFilter::equal_any(
            notification_config.recipient_list_ids.clone(),
        )),
    )?;
    let mut all_recipient_ids: Vec<String> = recipient_list_members
        .into_iter()
        .map(|row| row.recipient_id)
        .collect();

    // Add the configured recipient ids to the ones from any lists
    all_recipient_ids.extend(notification_config.recipient_ids.clone());

    let recipient_repository = RecipientRepository::new(&ctx.connection);
    // Get the recipients by recipient ids
    let recipients = recipient_repository
        .query_by_filter(RecipientFilter::new().id(EqualFilter::equal_any(all_recipient_ids)))?;

    // Convert the recipients into NotificationTargets
    let recipients: Vec<NotificationTarget> =
        recipients.into_iter().map(|row| row.into()).collect();
    notification_targets.extend(recipients);

    // // TODO: Get the sql recipient list ids
    // let sql_recipient_list_ids = match &notification_config.sql_recipient_list_ids {
    //     Some(ids) => ids,
    //     None => {
    //         [];
    //     }
    // };
    Ok(notification_targets)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use repository::{
        mock::{
            mock_recipient_a, mock_recipient_b, mock_recipient_list_with_recipient_members_a_and_b,
            MockDataInserts,
        },
        test_db::setup_all,
        NotificationConfigStatus,
    };
    use util::uuid::uuid;

    use crate::{service_provider::ServiceProvider, test_utils::get_test_settings};

    use super::*;

    #[actix_rt::test]
    async fn test_get_notification_targets() {
        let (_, _, connection_manager, _) = setup_all(
            "test_get_notification_targets",
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
        let context = ServiceContext::as_server_admin(service_provider).unwrap();

        // We'll use the mock recipients and recipient lists

        let recipient1 = mock_recipient_a();
        let recipient2 = mock_recipient_b();
        let recipient_list1 = mock_recipient_list_with_recipient_members_a_and_b();

        // 1. Check we get the 2 recipients with both recipient list and recipient ids

        // Create a notification config with recipient list and individual recipient ids
        let notification_config = NotificationConfig {
            id: uuid(),
            recipient_ids: vec![recipient1.id.clone(), recipient2.id.clone()],
            recipient_list_ids: vec![recipient_list1.id.clone()],
            sql_recipient_list_ids: vec![],
            status: NotificationConfigStatus::Enabled,
            ..Default::default()
        };

        // Call the function being tested
        let notification_targets =
            get_notification_targets(&context, &notification_config).unwrap();

        // Check that the correct recipients were returned
        assert_eq!(notification_targets.len(), 2); // Recipient A & B

        assert!(notification_targets.contains(&NotificationTarget::from(recipient1.clone())));
        assert!(notification_targets.contains(&NotificationTarget::from(recipient2.clone())));

        // 2. Check we get the result with just the recipient list
        let notification_config = NotificationConfig {
            id: uuid(),
            recipient_ids: vec![],
            recipient_list_ids: vec![recipient_list1.id.clone()],
            sql_recipient_list_ids: vec![],
            status: NotificationConfigStatus::Enabled,
            ..Default::default()
        };

        // Call the function being tested
        let notification_targets =
            get_notification_targets(&context, &notification_config).unwrap();

        // Check that the correct recipients were returned
        assert_eq!(notification_targets.len(), 2); // Recipient A & B

        assert!(notification_targets.contains(&NotificationTarget::from(recipient1.clone())));
        assert!(notification_targets.contains(&NotificationTarget::from(recipient2.clone())));

        // 3. Check we get the result with just the recipients
        let notification_config = NotificationConfig {
            id: uuid(),
            recipient_ids: vec![recipient1.id.clone(), recipient2.id.clone()],
            recipient_list_ids: vec![],
            sql_recipient_list_ids: vec![],
            status: NotificationConfigStatus::Enabled,
            ..Default::default()
        };

        // Call the function being tested
        let notification_targets =
            get_notification_targets(&context, &notification_config).unwrap();

        // Check that the correct recipients were returned
        assert_eq!(notification_targets.len(), 2); // Recipient A & B

        assert!(notification_targets.contains(&NotificationTarget::from(recipient1.clone())));
        assert!(notification_targets.contains(&NotificationTarget::from(recipient2.clone())));

        // TODO: Test SQL Recipients
    }
}
