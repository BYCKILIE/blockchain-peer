use tokio::{io, task};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use administrator::web_server;
use common::config::db_config::DbConfig;
use administrator::synchronizer::Synchronizer;
use common::config::webserver_config::WebServerConfig;
use common::memory::last_hash::LastHash;
use crate::administrator::{db_connector, notifier};
use crate::repo::alter_repo::AlterRepo;
use crate::service::alter_service::AlterService;
use crate::repo::read_repo::ReadRepo;
use crate::service::read_service::ReadService;

mod utils;
mod administrator;
mod dto;
mod http;
mod repo;
mod service;

pub async  fn power_module_db(webserver_config: WebServerConfig, db_config: DbConfig) -> io::Result<Vec<JoinHandle<()>>> {
    let db_pool = db_connector::create_pool(&db_config);
    
    let axum_pool = db_pool.clone();
    let axum_task = task::spawn(async move {
        let read_repo = ReadRepo { db_pool: axum_pool };
        let read_service = ReadService { repo: read_repo };

        web_server::start_server(webserver_config, read_service).await;
    });

    let (feedback_tx, feedback_rx) = mpsc::unbounded_channel::<Vec<(String, String)>>();
    
    let notifier_task = task::spawn(async move {
        notifier::start_notifier(feedback_rx).await;
    });
    
    let sync_pool = db_pool.clone();
    let synchronizer_task = task::spawn(async move {
        let alter_repo = AlterRepo { db_pool: sync_pool };
        let alter_service = AlterService { repo: alter_repo, last_hash: LastHash::new() };

        let mut synchronizer = Synchronizer {alter_service, feedback_sender: feedback_tx};
        synchronizer.power_synchronizer().await;
    });

    Ok(vec![axum_task, notifier_task, synchronizer_task])
}
