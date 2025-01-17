use config as settings;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Bad configuration: {0}")]
    BadConfig(String),
    #[error("while trying to build configuration")]
    ConfigFileError(#[from] settings::ConfigError),
}
