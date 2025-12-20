use serde::{Deserialize, Serialize};

use std::{env, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub upstream: Upstream,
    pub rest_port: u32,
    pub grpc_port: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upstream {
    pub base_url: String,
    pub rest_port: u16,
    pub grpc_port: u16,
}

fn load_from_env() -> Result<Config, Box<dyn std::error::Error>> {
    use std::env;

    let upstream = Upstream {
        base_url: env::var("UPSTREAM_BASE_URL")
            .map_err(|_| "UPSTREAM_BASE_URL environment variable is required")?,
        rest_port: env::var("UPSTREAM_REST_PORT")
            .map_err(|_| "UPSTREAM_REST_PORT environment variable is required")?
            .parse::<u16>()
            .map_err(|e| format!("Failed to parse UPSTREAM_REST_PORT: {}", e))?,
        grpc_port: env::var("UPSTREAM_GRPC_PORT")
            .map_err(|_| "UPSTREAM_GRPC_PORT environment variable is required")?
            .parse::<u16>()
            .map_err(|e| format!("Failed to parse UPSTREAM_GRPC_PORT: {}", e))?,
    };

    let rest_port = env::var("REST_PORT")
        .map_err(|_| "REST_PORT environment variable is required")?
        .parse::<u32>()
        .map_err(|e| format!("Failed to parse REST_PORT: {}", e))?;

    let grpc_port = env::var("GRPC_PORT")
        .map_err(|_| "GRPC_PORT environment variable is required")?
        .parse::<u32>()
        .map_err(|e| format!("Failed to parse GRPC_PORT: {}", e))?;

    Ok(Config {
        upstream,
        rest_port,
        grpc_port,
    })
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Retrieve env variable
    let config_path = env::var("SIDE_CAR_CONFIG").unwrap_or_else(|_| "config.yaml".to_string());

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

    // Fallback to environment variables
    tracing::info!(
        "No config file found, attempting to load configuration from environment variables"
    );
    match load_from_env() {
        Ok(config) => {
            tracing::info!("Successfully loaded configuration from environment variables");
            Ok(config)
        }
        Err(e) => Err(format!(
            "Config file not found and environment variables are incomplete. \
             Tried: '{}', 'config.yaml', 'config.example.yaml', and environment variables. \
             Error: {}",
            config_path, e
        )
        .into()),
    }
}
