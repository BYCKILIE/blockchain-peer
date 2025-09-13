pub mod peer_config;
pub mod db_config;
pub mod nats_config;
pub mod webserver_config;

use std::env;
use config::{Config, File};
use serde::Deserialize;
use std::error::Error;

use {peer_config::PeerConfig, db_config::DbConfig, nats_config::NatsConfig};
use crate::config::webserver_config::WebServerConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub peer: PeerConfig,
    pub webserver: WebServerConfig,
    pub database: DbConfig,
    pub nats: NatsConfig,
}

impl AppConfig {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let settings = Config::builder()
            .add_source(File::with_name(&format!("{}/AppConfig.toml", path)).required(true))
            .build()
            .map_err(|_| "No config found")?;

        let mut config: AppConfig = settings
            .try_deserialize()
            .map_err(|e| format!("Invalid config format: {:?}", e))?;

        config.apply_environment();
        Ok(config)
    }

    fn apply_environment(&mut self) {
        // peer
        match env::var("PEER_HOST") {
            Ok(val) => {
                self.peer.host = val.clone();
            }
            Err(_) => {},
        }

        match env::var("PEER_PORT") {
            Ok(val) => {
                self.peer.port = val.parse::<u32>().expect("Invalid NODE_PORT");
            }
            Err(_) => {},
        }

        match env::var("PEER_THREAD_POOL") {
            Ok(val) => {
                self.peer.thread_pool = val.parse::<u32>().expect("NODE_THREAD_POOL");
            }
            Err(_) => {},
        }

        match env::var("PEER_OUT_DOMAIN") {
            Ok(val) => {
                self.peer.out_name = val.clone();
            }
            Err(_) => {},
        }

        match env::var("PEER_OUT_SUBDOMAIN") {
            Ok(val) => {
                self.peer.out_sub_name = val.parse::<u32>().expect("Invalid OUT_SUBDOMAIN");
            },
            Err(_) => {},
        }

        match env::var("PEER_OUT_PORT") {
            Ok(val) => {
                self.peer.out_port = val.parse::<u32>().expect("Invalid OUT_PORT");
            }
            Err(_) => {},
        }

        match env::var("PEER_MIN_REF") {
            Ok(val) => {
                self.peer.min_ref = val.parse::<u32>().expect("Invalid PEER_MIN_REF");
            }
            Err(_) => {},
        }

        match env::var("PEER_MAX_REF") {
            Ok(val) => {
                self.peer.max_ref = val.parse::<u32>().expect("Invalid PEER_MAX_REF");
            }
            Err(_) => {},
        }

        match env::var("PEER_CONNECTIONS") {
            Ok(val) => {
                self.peer.peer_connections = val.parse::<u32>().expect("Invalid PEER_CONNECTIONS");
            }
            Err(_) => {},
        }

        match env::var("PEER_COMMITTEE_SIZE") {
            Ok(val) => {
                self.peer.committee_size = val.parse::<u32>().expect("Invalid PEER_COMMITTEE_SIZE");
            }
            Err(_) => {},
        }

        match env::var("PEER_KEYS_PATH") {
            Ok(val) => {
                self.peer.keys_path = val.clone();
            }
            Err(_) => {},
        }
        
        // webserver
        match env::var("WEB_HOST") {
            Ok(val) => {
                self.webserver.host = val.clone();
            },
            Err(_) => {},
        }

        match env::var("WEB_PORT") {
            Ok(val) => {
                self.webserver.port = val.parse::<u32>().expect("Invalid WEB_PORT");
            },
            Err(_) => {},
        }

        match env::var("WEB_THREAD_POOL") {
            Ok(val) => {
                self.webserver.thread_pool = val.parse::<u32>().expect("Invalid WEB_THREAD_POOL");
            },
            Err(_) => {},
        }

        // database
        match env::var("DB_HOST") {
            Ok(val) => {
                self.database.host = val.clone();
            },
            Err(_) => {},
        }

        match env::var("DB_PORT") {
            Ok(val) => {
                self.database.port = val.parse::<u32>().expect("Invalid DB_PORT");
            },
            Err(_) => {},
        }

        match env::var("DB_USERNAME") {
            Ok(val) => {
                self.database.user = val.clone();
            },
            Err(_) => {},
        }

        match env::var("DB_PASSWORD") {
            Ok(val) => {
                self.database.password = val.clone();
            },
            Err(_) => {},
        }

        match env::var("DB_NAME") {
            Ok(val) => {
                self.database.db_name = val.clone();
            },
            Err(_) => {},
        }

        match env::var("DB_THREAD_POOL") {
            Ok(val) => {
                self.database.thread_pool = val.parse::<u32>().expect("Invalid DB_THREAD_POOL");
            },
            Err(_) => {},
        }

        // nats
        match env::var("NATS_HOST") {
            Ok(val) => {
                self.nats.host = val.clone();
            },
            Err(_) => {},
        }

        match env::var("NATS_PORT") {
            Ok(val) => {
                self.nats.port = val.parse::<u32>().expect("Invalid NATS_PORT");
            },
            Err(_) => {},
        }

        match env::var("NATS_NO_CHANNELS") {
            Ok(val) => {
                self.nats.no_channels = val.parse::<u32>().expect("Invalid NATS_NO_CHANNELS");
            },
            Err(_) => {},
        }

        match env::var("NATS_THREAD_POOL") {
            Ok(val) => {
                self.nats.thread_pool = val.parse::<u32>().expect("Invalid NATS_THREAD_POOL");
            },
            Err(_) => {},
        }

        match env::var("NATS_CERT_PATH") {
            Ok(val) => {
                self.nats.cert_store_path = val.clone();
            },
            Err(_) => {},
        }
        
    }
}