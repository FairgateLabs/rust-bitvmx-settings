use crate::errors::ConfigError;
use clap::Parser;
use config::Config;
use config::FileFormat;
use serde::Deserialize;
use std::env;
use std::fs;
use tracing::{info, warn};
use zeroize::Zeroizing;

static DEFAULT_CONFIG: &str = "development";
static CONFIG_PATH: &str = "config";

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
    let builder = Config::builder();
    let config = builder
        .add_source(config::File::from_str(
            &decrypt_or_read_file(config)?,
            FileFormat::Yaml,
        ))
        .build()?;

    // Resolve [env:NAME] patterns with environment variable values
    let mut value: serde_json::Value = config
        .try_deserialize()
        .map_err(ConfigError::ConfigFileError)?;

    resolve_env_vars(&mut value)?;

    serde_json::from_value(value).map_err(|e| {
        ConfigError::BadConfig(format!(
            "Failed to deserialize config after env resolution: {e}"
        ))
    })
}

fn resolve_env_vars(value: &mut serde_json::Value) -> Result<(), ConfigError> {
    match value {
        serde_json::Value::String(s) => {
            if s.starts_with("(env:") && s.ends_with(')') {
                let var_name = &s[5..s.len() - 1];
                *s = env::var(var_name).map_err(|_| {
                    ConfigError::BadConfig(format!("Environment variable '{var_name}' not found"))
                })?;
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values_mut() {
                resolve_env_vars(v)?;
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                resolve_env_vars(v)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn decrypt_or_read_file(fname: &str) -> Result<zeroize::Zeroizing<String>, ConfigError> {
    if let Ok(secret_key) = std::env::var("BITVMX_AGE_KEY") {
        let encrypted = fs::read(fname)?;
        decrypt_age_in_memory(&encrypted, &secret_key)
    } else {
        let content = fs::read_to_string(fname)?;
        Ok(zeroize::Zeroizing::new(content))
    }
}

fn decrypt_age_in_memory(
    ciphertext: &[u8],
    secret_key: &str,
) -> Result<zeroize::Zeroizing<String>, ConfigError> {
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

    let mut out = Zeroizing::new(String::new());
    reader
        .read_to_string(&mut out)
        .map_err(|e| ConfigError::BadConfig(format!("plaintext is not valid UTF-8: {e}")))?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_env_vars_replaces_env_pattern() {
        let var_name = "BITVMX_TEST_RESOLVE_VAR";
        let var_value = "my_secret_value";
        env::set_var(var_name, var_value);

        let mut value = json!({
            "plain": "no-replace",
            "secret": format!("(env:{var_name})"),
            "nested": {
                "inner": format!("(env:{var_name})"),
                "keep": 42
            },
            "list": ["a", format!("(env:{var_name})")]
        });

        resolve_env_vars(&mut value).expect("resolve_env_vars should succeed");

        assert_eq!(value["plain"], "no-replace");
        assert_eq!(value["secret"], var_value);
        assert_eq!(value["nested"]["inner"], var_value);
        assert_eq!(value["nested"]["keep"], 42);
        assert_eq!(value["list"][0], "a");
        assert_eq!(value["list"][1], var_value);

        env::remove_var(var_name);
    }

    #[test]
    fn test_resolve_env_vars_missing_var_returns_error() {
        let mut value = json!({ "key": "(env:BITVMX_NONEXISTENT_VAR_12345)" });
        let result = resolve_env_vars(&mut value);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_age_in_memory_roundtrip() {
        use age::secrecy::ExposeSecret;
        use age::x25519;
        use std::io::Write;

        let identity = x25519::Identity::generate();
        let recipient = identity.to_public();

        // Encrypt a small YAML payload in memory
        let plaintext = "database_url: postgres://localhost/mydb\nport: 5432\n";
        let recipients: Vec<Box<dyn age::Recipient + Send>> = vec![Box::new(recipient)];
        let encryptor = age::Encryptor::with_recipients(
            recipients.iter().map(|r| r.as_ref() as &dyn age::Recipient),
        )
        .expect("valid recipient");

        let mut encrypted = vec![];
        {
            let mut writer = encryptor
                .wrap_output(&mut encrypted)
                .expect("wrap_output should succeed");
            writer
                .write_all(plaintext.as_bytes())
                .expect("write should succeed");
            writer.finish().expect("finish should succeed");
        }

        let secret_key = identity.to_string();
        let decrypted = decrypt_age_in_memory(&encrypted, secret_key.expose_secret())
            .expect("decryption should succeed");

        assert_eq!(&*decrypted, plaintext);
    }
}
