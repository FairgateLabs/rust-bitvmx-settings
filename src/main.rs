use bitvmx_settings::settings;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    test: String,
}

fn main() -> Result<(), bitvmx_settings::errors::ConfigError> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    let config = settings::load::<Config>()?;
    println!("Config: {:?}", config.test);
    Ok(())
}
