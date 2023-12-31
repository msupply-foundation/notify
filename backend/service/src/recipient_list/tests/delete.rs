#[cfg(test)]
mod recipient_list_delete_test {
    use std::sync::Arc;

    use repository::mock::mock_recipient_list_c;
    use repository::{mock::MockDataInserts, test_db::setup_all};
    use repository::{
        EqualFilter, RecipientListFilter, RecipientListMemberFilter, RecipientListMemberRepository,
        RecipientListRepository,
    };

    use crate::recipient_list::delete::DeleteRecipientListError;
    use crate::service_provider::ServiceContext;
    use crate::service_provider::ServiceProvider;
    use crate::test_utils::get_test_settings;

    #[actix_rt::test]
    async fn recipient_list_service_delete_errors() {
        let (_, _, connection_manager, _) = setup_all(
            "recipient_list_service_delete_errors",
            MockDataInserts::none(),
        )
        .await;

        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));
        let context = ServiceContext::new(service_provider).unwrap();
        let service = &context.service_provider.recipient_list_service;

        // RecipientList does not exist
        assert_eq!(
            service.delete_recipient_list(&context, "invalid_id",),
            Err(DeleteRecipientListError::RecipientListDoesNotExist)
        );
    }
    #[actix_rt::test]
    async fn recipient_list_service_delete_success() {
        let (_, _, connection_manager, _) = setup_all(
            "recipient_list_service_delete_success",
            MockDataInserts::none().recipient_list_members(),
        )
        .await;

        let connection = connection_manager.connection().unwrap();
        let recipient_list_repository = RecipientListRepository::new(&connection);
        let service_provider = Arc::new(ServiceProvider::new(
            connection_manager,
            get_test_settings(""),
        ));
        let context = ServiceContext::new(service_provider).unwrap();
        let service = &context.service_provider.recipient_list_service;

        assert_eq!(
            service.delete_recipient_list(&context, &mock_recipient_list_c().id),
            Ok(mock_recipient_list_c().id.clone())
        );

        assert_eq!(
            recipient_list_repository
                .query_by_filter(
                    RecipientListFilter::new()
                        .id(EqualFilter::equal_to(&mock_recipient_list_c().id))
                )
                .unwrap(),
            vec![]
        );
        assert_eq!(
            RecipientListMemberRepository::new(&connection)
                .query_by_filter(
                    RecipientListMemberFilter::new()
                        .recipient_list_id(EqualFilter::equal_to(&mock_recipient_list_c().id))
                )
                .unwrap(),
            vec![]
        );
    }
}
