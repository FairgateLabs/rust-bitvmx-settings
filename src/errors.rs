use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Bad configuration: {0}")]
    BadConfig(String),
    #[error("while trying to build configuration")]
    ConfigFileError(#[from] config::ConfigError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
