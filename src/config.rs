use serde::Deserialize;
use log::info;

use crate::error;
use crate::error::Error;

#[derive(Deserialize, Clone)]
pub struct Configuration
{
    pub download_dir: String,
    pub ydl_exec: String,
    pub static_dir: Option<String>,
    pub listen_address: String,
    pub listen_port: u16,
    pub log_timestamp: bool,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for Configuration
{
    fn default() -> Self
    {
        Self {
            download_dir: String::new(),
            ydl_exec: String::from("yt-dlp"),
            static_dir: None,
            listen_address: String::from("127.0.0.1"),
            listen_port: 8000,
            log_timestamp: false,
            extra_args: Vec::new(),
        }
    }
}

impl Configuration
{
    pub fn readFromFile(f: &std::path::Path) -> Result<Self, Error>
    {
        info!("Reading configuration from {:?}...", f);
        let contents = std::fs::read_to_string(f).map_err(
            |_| rterr!("Failed to read configuration file"))?;
        let result: Configuration = toml::from_str(&contents).map_err(
            |_| rterr!("Invalid configuration file"))?;
        Ok(result)
    }
}
