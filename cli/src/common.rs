//! # Common
//!
//! `common` contains the CLI common functionalities.

use crate::error::Error;
use crate::result::Result;
use clap::{App, AppSettings, Arg};
use config::Config;
use models::stage::Stage;
use models::version::VERSION;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use store::traits::Store;
use store::PoolFactory;
use store::StoreFactory;

/// `app` creates and returns a clap `App`.
pub fn app(name: &'static str, about: &'static str) -> App<'static, 'static> {
    App::new(name).version(VERSION).about(about).settings(&[
        AppSettings::ArgsNegateSubcommands,
        AppSettings::ArgRequiredElseHelp,
        AppSettings::ColorAuto,
        AppSettings::DontCollapseArgsInUsage,
        AppSettings::DeriveDisplayOrder,
        AppSettings::GlobalVersion,
        AppSettings::StrictUtf8,
        AppSettings::VersionlessSubcommands,
    ])
}

/// `add_verbose` adds a verbose option to a command.
pub fn add_verbose(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name("verbose")
            .help("Turns on verbose output")
            .short("V")
            .long("verbose")
            .takes_value(false)
            .required(false),
    )
}

/// `create_dir` creates a directory.
pub fn create_dir(path: &str) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| e.into())
}

/// `destroy_dir` destroys a directory.
pub fn destroy_dir(path: &str) -> Result<()> {
    fs::remove_dir(path).map_err(|e| e.into())
}

/// `write_file` writes a file.
pub fn write_file(path: &str, buf: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    file.write_all(buf).map_err(|e| e.into())
}

/// `read_file` reads a file.
pub fn read_file(path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    Ok(buf)
}

/// `destroy_file` destroys a file.
pub fn destroy_file(path: &str) -> Result<()> {
    fs::remove_file(path).map_err(|e| e.into())
}

/// `data_dir` returns the data directory.
pub fn data_dir() -> Result<String> {
    let mut path = env::current_dir()?;
    path.push("data");

    if let Some(path) = path.to_str() {
        Ok(path.into())
    } else {
        let err = Error::InvalidPath;
        Err(err)
    }
}

/// `destroy_data_dir` destroys the data directory.
pub fn destroy_data_dir() -> Result<()> {
    destroy_dir(&data_dir()?)
}

/// `config_dir` returns the Alsacoin configs directory.
pub fn config_dir() -> Result<String> {
    let mut path = env::current_dir()?;
    path.push("data");
    path.push("config");

    if let Some(path) = path.to_str() {
        Ok(path.into())
    } else {
        let err = Error::InvalidPath;
        Err(err)
    }
}

/// `config_path` returns an Alsacoin config path.
pub fn config_path(stage: Stage) -> Result<String> {
    let mut path = env::current_dir()?;
    path.push("data");
    path.push("config");
    path.push(&format!("{}.toml", stage));

    if let Some(path) = path.to_str() {
        Ok(path.into())
    } else {
        let err = Error::InvalidPath;
        Err(err)
    }
}

/// `create_config_dir` creates the Alsacoin configs directory if missing.
pub fn create_config_dir() -> Result<()> {
    create_dir(&config_dir()?)
}

/// `destroy_config_dir` destroys the Alsacoin configs directory.
pub fn destroy_config_dir() -> Result<()> {
    destroy_dir(&config_dir()?)
}

/// `write_config` writes an Alsacoin config.
pub fn write_config(stage: Stage, config: &Config) -> Result<()> {
    config.validate()?;

    let path = config_path(stage)?;
    let buf = config.to_toml()?.into_bytes();

    write_file(&path, &buf)
}

/// `create_config` creates a new Alsacoin config.
pub fn create_config(stage: Stage) -> Result<()> {
    let config = Config::default();
    write_config(stage, &config)
}

/// `read_config` reads an Alsacoin config.
pub fn read_config(stage: Stage) -> Result<Config> {
    let path = config_path(stage)?;
    let buf = read_file(&path)?;

    let contents = String::from_utf8(buf)?;
    Config::from_toml(&contents).map_err(|e| e.into())
}

/// `store_dir` returns the Alsacoin stores directory.
pub fn store_dir() -> Result<String> {
    let mut path = env::current_dir()?;
    path.push("data");
    path.push("store");

    if let Some(path) = path.to_str() {
        Ok(path.into())
    } else {
        let err = Error::InvalidPath;
        Err(err)
    }
}

/// `store_path` returns an Alsacoin store path.
pub fn store_path(stage: Stage) -> Result<String> {
    let mut path = env::current_dir()?;
    path.push("data");
    path.push("store");
    path.push(&format!("{}.store", stage));

    if let Some(path) = path.to_str() {
        Ok(path.into())
    } else {
        let err = Error::InvalidPath;
        Err(err)
    }
}

/// `create_store_dir` creates the Alsacoin stores directory if missing.
pub fn create_store_dir() -> Result<()> {
    create_dir(&store_dir()?)
}

/// `destroy_store_dir` destroys the Alsacoin stores directory.
pub fn destroy_store_dir() -> Result<()> {
    destroy_dir(&store_dir()?)
}

/// `create_store` creates an Alsacoin store.
pub fn create_store(stage: Stage, config: &Config) -> Result<()> {
    config.validate()?;

    let kind = config.store.kind.clone().unwrap();

    let path = if &kind == "persistent" {
        Some(store_path(stage)?)
    } else {
        None
    };

    StoreFactory::create(path, &config.store)
        .map(|_| ())
        .map_err(|e| e.into())
}

/// `open_store` opens an Alsacoin store.
pub fn open_store(stage: Stage, config: &Config) -> Result<Box<dyn Store>> {
    config.validate()?;

    let kind = config.store.kind.clone().unwrap();

    let path = if &kind == "persistent" {
        Some(store_path(stage)?)
    } else {
        None
    };

    StoreFactory::create(path, &config.store).map_err(|e| e.into())
}

/// `open_pool` opens an Alsacoin pool.
pub fn open_pool(config: &Config) -> Result<Box<dyn Store>> {
    config.validate()?;

    PoolFactory::create(&config.pool).map_err(|e| e.into())
}

/// `init_config` inits the Alsacoin config of a specific stage.
pub fn init_config(stage: Stage) -> Result<()> {
    create_config_dir()?;

    if read_config(stage).is_err() {
        create_config(stage)?;
    }

    Ok(())
}

/// `init_configs` inits the Alsacoin configs.
pub fn init_configs() -> Result<()> {
    create_config_dir()?;
    init_config(Stage::Development)?;
    init_config(Stage::Testing)?;
    init_config(Stage::Production)
}

/// `init_store` inits the Alsacoin store of a specific stage.
pub fn init_store(stage: Stage) -> Result<()> {
    create_store_dir()?;

    let config = read_config(stage)?;

    create_store(stage, &config)
}

/// `init_stores` inits the Alsacoin stores.
pub fn init_stores() -> Result<()> {
    init_store(Stage::Development)?;
    init_store(Stage::Testing)?;
    init_store(Stage::Production)
}

/// `init` inits the Alsacoin CLI environment.
pub fn init() -> Result<()> {
    init_configs()?;
    init_stores()
}

/// `destroy_store` destroys the Alsacoin store of a specific stage.
pub fn destroy_store(stage: Stage) -> Result<()> {
    let path = store_path(stage)?;
    destroy_file(&path)
}

/// `destroy_stores` destroys the Alsacoin stores.
pub fn destroy_stores() -> Result<()> {
    destroy_store(Stage::Development)?;
    destroy_store(Stage::Testing)?;
    destroy_store(Stage::Production)
}

/// `destroy_config` destroys the Alsacoin config of a specific stage.
pub fn destroy_config(stage: Stage) -> Result<()> {
    let path = config_path(stage)?;
    destroy_file(&path)
}

/// `destroy_configs` destroys the Alsacoin configs.
pub fn destroy_configs() -> Result<()> {
    destroy_config(Stage::Development)?;
    destroy_config(Stage::Testing)?;
    destroy_config(Stage::Production)
}

/// `destroy` destroys the Alsacoin CLI environment.
pub fn destroy() -> Result<()> {
    destroy_data_dir()
}

/// `reset_store` resets the Alsacoin store of a specific stage.
pub fn reset_store(stage: Stage) -> Result<()> {
    destroy_store(stage)?;
    init_store(stage)
}

/// `reset_stores` resets the Alsacoin stores.
pub fn reset_stores() -> Result<()> {
    destroy_stores()?;
    init_stores()
}

/// `reset_config` resets the Alsacoin config of a specific stage.
pub fn reset_config(stage: Stage) -> Result<()> {
    destroy_config(stage)?;
    init_config(stage)
}

/// `reset_configs` resets the Alsacoin configs.
pub fn reset_configs() -> Result<()> {
    reset_config(Stage::Development)?;
    reset_config(Stage::Testing)?;
    reset_config(Stage::Production)
}

/// `reset` resets the Alsacoin CLI environment.
pub fn reset() -> Result<()> {
    reset_stores()?;
    reset_configs()
}
