use serde::{Deserialize, Serialize};

use std::{env, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sender: String,
    pub smtp_pass: String,
    pub smtp_relay: String,
    pub smtp_username: String,
    pub port: i32,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Retrieve env variable
    let config_path =
        env::var("EMAIL_SERVICE_CONFIG").unwrap_or_else(|_| "config.yaml".to_string());

    // Try env path
    if Path::new(&config_path).exists() {
        let contents = fs::read_to_string(&config_path)?;
        return serde_yaml::from_str(&contents).map_err(Into::into);
    }

    // Fallback to config.yaml
    if Path::new("config.yaml").exists() {
        tracing::warn!(
            "Config file '{}' not found, falling back to 'config.yaml'",
            config_path
        );
        let contents = fs::read_to_string("config.yaml")?;
        return serde_yaml::from_str(&contents).map_err(Into::into);
    }

    // Fallback to config.example.yaml
    if Path::new("config.example.yaml").exists() {
        tracing::warn!(
            "Config file '{}' and 'config.yaml' not found, falling back to 'config.example.yaml'\
             \n This file should not be used and should be replaced with actual data",
            config_path
        );
        let contents = fs::read_to_string("config.example.yaml")?;
        return serde_yaml::from_str(&contents).map_err(Into::into);
    }

    Err(format!(
        "Config file not found. Tried: '{}', 'config.yaml', 'config.example.yaml'",
        config_path
    )
    .into())
}
