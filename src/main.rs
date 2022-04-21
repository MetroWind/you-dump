#![allow(non_snake_case)]
use std::path::Path;

#[macro_use]
mod error;
mod config;
mod download;
mod app;

use error::Error;

fn findConfig() -> Result<config::Configuration, Error>
{
    let p = Path::new("you-dump.toml");
    if p.exists()
    {
        config::Configuration::readFromFile(p)
    }
    else
    {
        let p = Path::new("/etc/you-dump.toml");
        if p.exists()
        {
            config::Configuration::readFromFile(p)
        }
        else
        {
            Ok(config::Configuration::default())
        }
    }
}

fn main() -> Result<(), Error>
{
    let config = findConfig()?;
    if !config.log_timestamp
    {
        env_logger::builder().format_timestamp(None).init();
    }
    else
    {
        env_logger::init();
    }
    let a = app::App::new(config);
    tokio::runtime::Runtime::new().unwrap().block_on(a.serve())?;
    Ok(())
}
