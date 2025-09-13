use config::{Config, File};
use serde::Deserialize;
use std::error::Error;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct BuildInfo {
    pub root: RootInfo,
    pub app: AppInfo,
}

#[derive(Debug, Deserialize)]
pub struct RootInfo {
    pub word_before: String,
    pub logo: String,
    pub details: String,
}

#[derive(Debug, Deserialize)]
pub struct AppInfo {
    pub word_before: String,
    pub logo: String,
    pub details: String,
}

impl BuildInfo {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let settings = Config::builder()
            .add_source(File::with_name(path).required(true))
            .build()
            .map_err(|_| "No build info file found")?;

        let config: BuildInfo = settings
            .try_deserialize()
            .map_err(|_| "Invalid toml format")?;
        Ok(config)
    }
}

pub fn print_info(path: &str) {
    match BuildInfo::new(&format!("{}/BuildInfo.toml", path)) {
        Ok(info) => {
            print!("{}", info.root.word_before);
            println!("{}", info.root.logo);
            println!("{}", info.root.details);
            thread::sleep(Duration::from_secs(2));

            print!("{}", info.app.word_before);
            println!("{}", info.app.logo);
            println!("{}", info.app.details);
            thread::sleep(Duration::from_secs(2));
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }
}
