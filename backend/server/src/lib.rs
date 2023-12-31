use crate::{
    auto_backup::auto_backup, configuration::get_or_create_token_secret, cors::cors_policy,
    scheduled_tasks::scheduled_task_runner, serve_frontend::config_server_frontend,
    static_files::config_static_files,
};

use self::middleware::{compress as compress_middleware, logger as logger_middleware};
use graphql_core::loader::{get_loaders, LoaderRegistry};

use graphql::config as graphql_config;
use log::{error, info};
use middleware::{add_authentication_context, limit_content_length};
use repository::{get_storage_connection_manager, run_db_migrations, StorageConnectionManager};

use actix_web::{web::Data, App, HttpServer};
use coldchain::ColdChainPlugin;
use scheduled::ScheduledNotificationPlugin;
use std::{
    ops::DerefMut,
    sync::{Arc, RwLock},
};
use tokio::sync::{mpsc, Mutex};

use service::{
    auth_data::AuthData,
    plugin::PluginTrait,
    recipient::telegram::update_telegram_recipients,
    service_provider::{ServiceContext, ServiceProvider},
    settings::{is_develop, ServerSettings, Settings},
    token_bucket::TokenBucket,
};

use telegram::{service::TelegramService, TelegramClient};

mod auto_backup;
pub mod configuration;
pub mod cors;
pub mod environment;
pub mod logging;
pub mod middleware;
mod scheduled_tasks;
mod serve_frontend;
pub mod static_files;

fn auth_data(
    _server_settings: &ServerSettings,
    token_bucket: Arc<RwLock<TokenBucket>>,
    token_secret: String,
) -> Data<AuthData> {
    Data::new(AuthData {
        auth_token_secret: token_secret,
        token_bucket,
    })
}
async fn run_server(
    config_settings: Settings,
    off_switch: Arc<Mutex<mpsc::Receiver<()>>>,
    token_bucket: Arc<RwLock<TokenBucket>>,
    token_secret: String,
    connection_manager: StorageConnectionManager,
) -> std::io::Result<bool> {
    let auth_data = auth_data(
        &config_settings.server,
        token_bucket.clone(),
        token_secret.clone(),
    );

    let (restart_switch, mut restart_switch_receiver) = tokio::sync::mpsc::channel::<bool>(1);

    let connection_manager_data_app = Data::new(connection_manager.clone());

    let service_provider =
        ServiceProvider::new(connection_manager.clone(), config_settings.clone());
    let service_provider_data = Data::new(service_provider);

    let loaders = get_loaders(&connection_manager, service_provider_data.clone()).await;
    let loader_registry_data = Data::new(LoaderRegistry { loaders });

    let settings_data = Data::new(config_settings.clone());

    let restart_switch = Data::new(restart_switch);

    let auto_backup_context = ServiceContext::new(service_provider_data.clone().into_inner());
    let auto_backup_context = match auto_backup_context {
        Ok(auto_backup_context) => auto_backup_context,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Error unable to create auto backup task context: {:?}",
                    error
                ),
            ));
        }
    };
    let auto_backup_handle = actix_web::rt::spawn(async move {
        auto_backup(auto_backup_context).await;
    });

    let scheduled_task_context = ServiceContext::new(service_provider_data.clone().into_inner());
    let scheduled_task_context = match scheduled_task_context {
        Ok(scheduled_task_context) => scheduled_task_context,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error unable to create scheduled task context: {:?}", error),
            ));
        }
    };

    // Setup plugins
    let plugins: Vec<Box<dyn service::plugin::PluginTrait>> = vec![
        Box::new(ScheduledNotificationPlugin::new()),
        Box::new(ColdChainPlugin::new()),
    ];

    let mut scheduled_task_handle = actix_web::rt::spawn(async move {
        scheduled_task_runner(scheduled_task_context, plugins).await;
    });

    // Setup a channel to receive telegram messages, which we want to handle in recipient service
    let telegram_token = config_settings.clone().telegram.token;
    let telegram_update_handler_option = match telegram_token {
        None => None,
        Some(telegram_token) => {
            let telegram_service = TelegramService::new(
                TelegramClient::new(telegram_token),
                config_settings.server.app_url.clone(),
            );
            let telegram_update_channel = telegram_service.init().await;

            // Handle telegram updates in recipient service
            let telegram_update_context =
                ServiceContext::new(service_provider_data.clone().into_inner());
            let telegram_update_context = match telegram_update_context {
                Ok(telegram_update_context) => telegram_update_context,
                Err(error) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "Error unable to create telegram update task context: {:?}",
                            error
                        ),
                    ));
                }
            };
            let telegram_update_handler = actix_web::rt::spawn(async move {
                update_telegram_recipients(telegram_update_context, &telegram_update_channel).await
            });
            Some(telegram_update_handler)
        }
    };

    let log_context = ServiceContext::new(service_provider_data.clone().into_inner());
    let log_context = match log_context {
        Ok(log_context) => log_context,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Error unable to create log context: {:?}", error),
            ));
        }
    };
    let log_service = &log_context.service_provider.log_service;
    let log_level = log_service.get_log_level(&log_context).unwrap();

    if config_settings.logging.is_some() {
        log_service
            .set_log_directory(
                &log_context,
                config_settings.logging.clone().unwrap().directory,
            )
            .unwrap();

        log_service
            .set_log_file_name(
                &log_context,
                config_settings.logging.clone().unwrap().filename,
            )
            .unwrap();
    }

    if log_level.is_none() && config_settings.logging.is_some() {
        log_service
            .update_log_level(&log_context, config_settings.logging.clone().unwrap().level)
            .unwrap();
    }

    let http_server_config_settings = config_settings.clone();
    let mut http_server = HttpServer::new(move || {
        let cors = cors_policy(&http_server_config_settings);
        App::new()
            .app_data(Data::new(http_server_config_settings.clone()))
            .app_data(service_provider_data.clone())
            .wrap(add_authentication_context(auth_data.clone()))
            .wrap(logger_middleware().exclude("/graphql")) // Exclude graphql requests, as they are logged from async-graphql
            .wrap(cors)
            .wrap(compress_middleware())
            .configure(graphql_config(
                connection_manager_data_app.clone(),
                loader_registry_data.clone(),
                service_provider_data.clone(),
                auth_data.clone(),
                settings_data.clone(),
                restart_switch.clone(),
            ))
            .configure(config_static_files)
            .wrap(limit_content_length())
            .configure(config_server_frontend)
    })
    .disable_signals();

    http_server = http_server.bind(config_settings.server.address())?;

    let running_sever = http_server.run();
    let server_handle = running_sever.handle();
    // run server in another task so that we can handle restart/off events here
    actix_web::rt::spawn(running_sever);

    let mut off_switch = off_switch.lock().await;
    let off_switch = off_switch.deref_mut();
    let ctrl_c = tokio::signal::ctrl_c();
    let restart = tokio::select! {
        _ = ctrl_c => false,
        Some(_) = off_switch.recv() => false,
        _ = restart_switch_receiver.recv() => true,
        scheduled_error = &mut scheduled_task_handle => {
            error!("Scheduled task stopped unexpectedly: {:?}", scheduled_error);
            false
        }
    };

    server_handle.stop(true).await;
    scheduled_task_handle.abort();
    auto_backup_handle.abort();
    if let Some(telegram_update_handler) = telegram_update_handler_option {
        telegram_update_handler.abort();
    }
    Ok(restart)
}

/// Starts the server
///
/// This method doesn't return until a message is send to the off_switch.
pub async fn start_server(
    config_settings: Settings,
    off_switch: tokio::sync::mpsc::Receiver<()>,
) -> std::io::Result<()> {
    info!(
        "Server starting in {} mode",
        if is_develop() {
            "Development"
        } else {
            "Production"
        }
    );

    let connection_manager = get_storage_connection_manager(&config_settings.database);

    if let Some(init_sql) = &config_settings.database.full_init_sql() {
        connection_manager.execute(init_sql).unwrap();
    }

    info!("Run DB migrations...");
    match run_db_migrations(&connection_manager.connection().unwrap()) {
        Ok(_) => info!("DB migrations succeeded"),
        Err(err) => {
            let msg = format!("Failed to run DB migrations: {}", err);
            error!("{}", msg);
            panic!("{}", msg);
        }
    };

    // allow the off_switch to be passed around during multiple server stages
    let off_switch = Arc::new(Mutex::new(off_switch));

    let token_bucket = Arc::new(RwLock::new(TokenBucket::new()));

    let token_secret = get_or_create_token_secret(&connection_manager.connection().unwrap());

    loop {
        match run_server(
            config_settings.clone(),
            off_switch.clone(),
            token_bucket.clone(),
            token_secret.clone(),
            connection_manager.clone(),
        )
        .await
        {
            Ok(restart) => {
                if !restart {
                    break;
                }

                // restart the server in next loop
                info!("Restart server");
            }
            Err(err) => return Err(err),
        }
    }

    info!("Remote server stopped");
    Ok(())
}
