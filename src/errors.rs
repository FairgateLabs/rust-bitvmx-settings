use config::ConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Bad configuration: {0}")]
    BadConfig(String),
    #[error("while trying to build configuration")]
    ConfigFileError(#[from] ConfigError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
