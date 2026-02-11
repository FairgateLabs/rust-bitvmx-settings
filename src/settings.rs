use crate::errors::ConfigError;
use clap::Parser;
use config::Config;
use serde::Deserialize;
use std::env;
use tracing::{info, warn};

static DEFAULT_CONFIG: &str = "development";
static CONFIG_PATH: &str = "config";

/* TODO: Potential macro to merge structs
#[macro_export]
macro_rules! generate_merge_function {
    ($struct_name:ident { $( $field:ident ),* }) => {
        impl $struct_name {
            /// Merges another instance into `self`, prioritizing non-`None` values from `other`.
            pub fn merge_with(&mut self, other: Self) {
                $(
                    if other.$field.is_some() {
                        self.$field = other.$field;
                    }
                )*
            }
        }
    };
}*/

#[derive(Parser, Debug, Deserialize)]
pub struct ConfigurationFile {
    #[arg(short, long)]
    pub configuration: Option<String>,
}

pub fn load<T: for<'a> Deserialize<'a>>() -> Result<T, ConfigError> {
    parse_config(&get_env())
}

pub fn load_config_file<T: for<'a> Deserialize<'a>>(
    config_file: Option<String>,
) -> Result<T, ConfigError> {
    let config_file = config_file.unwrap_or_else(get_env);
    parse_config(&config_file)
}

pub fn load_and_check_args<T: for<'a> Deserialize<'a>>() -> Result<T, ConfigError> {
    parse_config(&get_config_file())
}

fn get_config_file() -> String {
    let args = ConfigurationFile::parse();
    if let Some(config) = args.configuration {
        info!("Using configuration: {}", config);
        return config;
    }
    get_env()
}

fn get_env() -> String {
    env::var("BITVMX_ENV").unwrap_or_else(|_| {
        let config_path = format!("{}/{}.yaml", CONFIG_PATH, DEFAULT_CONFIG);
        warn!(
            "BITVMX_ENV not set. Using default configuration: {}",
            config_path
        );
        config_path
    })
}

fn parse_config<T: for<'a> Deserialize<'a>>(config: &str) -> Result<T, ConfigError> {
    let mut config_built = None;

    #[cfg(feature = "encrypted")]
    {
        use config::FileFormat;
        use std::fs;
        if let Ok(secret_key) = std::env::var("BITVMX_AGE_KEY") {
            let builder = Config::builder();
            let encrypted = fs::read(config)?;

            let plaintext = decrypt_age_in_memory(&encrypted, &secret_key)?;
            config_built = Some(
                builder
                    .add_source(config::File::from_str(&plaintext, FileFormat::Yaml))
                    .build()?,
            );
        }
    }

    if config_built.is_none() {
        let builder = Config::builder();
        config_built = Some(
            builder
                .add_source(config::File::with_name(config))
                .build()
                .map_err(ConfigError::ConfigFileError)?,
        );
    }

    config_built
        .unwrap()
        .try_deserialize::<T>()
        .map_err(ConfigError::ConfigFileError)
}

#[cfg(feature = "encrypted")]
fn decrypt_age_in_memory(ciphertext: &[u8], secret_key: &str) -> Result<String, ConfigError> {
    use std::io::Read;

    use age::x25519;
    use age::Decryptor;

    // BITVMX_AGE_KEY expected like: "AGE-SECRET-KEY-...."
    let identity: x25519::Identity = secret_key
        .trim()
        .parse()
        .map_err(|e| ConfigError::BadConfig(format!("invalid BITVMX_AGE_KEY: {e}")))?;

    let decryptor = Decryptor::new(ciphertext)
        .map_err(|e| ConfigError::BadConfig(format!("invalid age payload: {e}")))?;

    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|e| ConfigError::BadConfig(format!("decrypt failed: {e}")))?;

    let mut out = String::new();
    reader
        .read_to_string(&mut out)
        .map_err(|e| ConfigError::BadConfig(format!("plaintext is not valid UTF-8: {e}")))?;

    Ok(out)
}
