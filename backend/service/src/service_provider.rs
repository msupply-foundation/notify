use std::sync::Arc;

use repository::{RepositoryError, StorageConnection, StorageConnectionManager};
use telegram::TelegramClient;

use crate::{
    auth::{AuthService, AuthServiceTrait},
    datasource::{DatasourceService, DatasourceServiceTrait},
    email::{EmailService, EmailServiceTrait},
    log_service::{LogService, LogServiceTrait},
    notification::{NotificationService, NotificationServiceTrait},
    notification_config::{NotificationConfigService, NotificationConfigServiceTrait},
    notification_event::{NotificationEventService, NotificationEventServiceTrait},
    notification_query::{NotificationQueryService, NotificationQueryServiceTrait},
    plugin_store::{PluginService, PluginServiceTrait},
    recipient::{RecipientService, RecipientServiceTrait},
    recipient_list::{RecipientListService, RecipientListServiceTrait},
    settings::Settings,
    sql_recipient_list::{SqlRecipientListService, SqlRecipientListServiceTrait},
    user_account::{UserAccountService, UserAccountServiceTrait},
};

pub struct ServiceProvider {
    pub connection_manager: StorageConnectionManager,
    pub datasource_service: Box<dyn DatasourceServiceTrait>,
    pub email_service: Box<dyn EmailServiceTrait>,
    pub validation_service: Box<dyn AuthServiceTrait>,
    pub user_account_service: Box<dyn UserAccountServiceTrait>,
    pub notification_config_service: Box<dyn NotificationConfigServiceTrait>,
    pub recipient_service: Box<dyn RecipientServiceTrait>,
    pub recipient_list_service: Box<dyn RecipientListServiceTrait>,
    pub sql_recipient_list_service: Box<dyn SqlRecipientListServiceTrait>,
    pub notification_query_service: Box<dyn NotificationQueryServiceTrait>,
    pub notification_event_service: Box<dyn NotificationEventServiceTrait>,
    pub notification_service: Box<dyn NotificationServiceTrait>,
    pub plugin_service: Box<dyn PluginServiceTrait>,
    pub settings: Settings,
    pub telegram: Option<TelegramClient>,
    pub log_service: Box<dyn LogServiceTrait>,
}

pub struct ServiceContext {
    pub connection: StorageConnection,
    pub service_provider: Arc<ServiceProvider>,
    pub user_id: String,
}

impl ServiceContext {
    pub fn new(service_provider: Arc<ServiceProvider>) -> Result<ServiceContext, RepositoryError> {
        let connection = service_provider.connection_manager.connection()?;
        Ok(ServiceContext {
            connection,
            service_provider,
            user_id: "".to_string(),
        })
    }

    pub fn with_user(
        service_provider: Arc<ServiceProvider>,
        user_id: String,
    ) -> Result<ServiceContext, RepositoryError> {
        let connection = service_provider.connection_manager.connection()?;
        Ok(ServiceContext {
            connection,
            service_provider,
            user_id,
        })
    }

    pub fn as_server_admin(
        service_provider: Arc<ServiceProvider>,
    ) -> Result<ServiceContext, RepositoryError> {
        let connection = service_provider.connection_manager.connection()?;
        Ok(ServiceContext {
            connection,
            service_provider,
            user_id: "9cd8ce10-969b-45c4-871e-3a744c75ddf0".to_string(), // Admin user id is hardcoded in the database migration
        })
    }
}

impl ServiceProvider {
    pub fn new(connection_manager: StorageConnectionManager, settings: Settings) -> Self {
        let telegram = match &settings.telegram.token {
            Some(token) => Some(TelegramClient::new(token.clone())),
            None => None,
        };

        ServiceProvider {
            connection_manager,
            email_service: Box::new(EmailService::new(settings.clone())),
            datasource_service: Box::new(DatasourceService::new(settings.clone())),
            validation_service: Box::new(AuthService::new()),
            user_account_service: Box::new(UserAccountService {}),
            notification_config_service: Box::new(NotificationConfigService {}),
            recipient_service: Box::new(RecipientService {}),
            recipient_list_service: Box::new(RecipientListService {}),
            sql_recipient_list_service: Box::new(SqlRecipientListService {}),
            notification_query_service: Box::new(NotificationQueryService {}),
            notification_event_service: Box::new(NotificationEventService {}),
            notification_service: Box::new(NotificationService::new(settings.clone())),
            plugin_service: Box::new(PluginService {}),
            settings,
            telegram,
            log_service: Box::new(LogService {}),
        }
    }

    /// Establishes a new DB connection
    pub fn connection(&self) -> Result<StorageConnection, RepositoryError> {
        self.connection_manager.connection()
    }
}
