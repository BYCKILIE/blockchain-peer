use common::config::db_config::DbConfig;
use tokio_postgres::NoTls;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};

pub fn create_pool(db_config: &DbConfig) -> Pool {
    let mut cfg = Config::new();
    cfg.dbname = Some(db_config.db_name.clone());
    cfg.host = Some(db_config.host.clone());
    cfg.port = Some(db_config.port as u16);
    cfg.user = Some(db_config.user.clone());
    cfg.password = Some(db_config.password.clone());
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });

    cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
        .expect("Could not create pool")
}