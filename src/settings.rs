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
pub struct Args {
    #[arg(short, long)]
    pub configuration: Option<String>,
}

pub fn load<T: for<'a> Deserialize<'a>>() -> Result<T, ConfigError> {
    parse_config(&get_config_file())
}

fn get_config_file() -> String {
    let args = Args::parse();
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
    let config = Config::builder()
        .add_source(config::File::with_name(config))
        .build()
        .map_err(ConfigError::ConfigFileError)?;

    config
        .try_deserialize::<T>()
        .map_err(ConfigError::ConfigFileError)
}
