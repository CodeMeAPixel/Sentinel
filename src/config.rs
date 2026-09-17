use once_cell::sync::Lazy;
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Write, path::Path};

use crate::Error;

/// Global config object
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().expect("Failed to load config"));

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub token: String,
    pub proxy_url: String,
    pub frontend_url: String,
    pub owners: Vec<UserId>,
    pub client_id: String,
    pub client_secret: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            token: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            proxy_url: String::new(),
            frontend_url: String::from("https://skynet.infinitybots.gg"),
            owners: vec![
                UserId::new(728871946456137770),
                UserId::new(510065483693817867),
            ],
        }
    }
}

impl Config {
    fn load_dotenv() {
        let dotenv_path = Path::new(".env");
        if !dotenv_path.exists() {
            return;
        }

        let Ok(contents) = std::fs::read_to_string(dotenv_path) else {
            return;
        };

        for line in contents.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };

            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');

            if key.is_empty() || env::var(key).is_ok() {
                continue;
            }

            env::set_var(key, value);
        }
    }

    pub fn load_from_env() -> Result<Self, Error> {
        let database_url = env::var("DATABASE_URL").unwrap_or_default();
        let token = env::var("DISCORD_TOKEN")
            .or_else(|_| env::var("TOKEN"))
            .unwrap_or_default();
        let proxy_url = env::var("PROXY_URL").unwrap_or_default();
        let frontend_url = env::var("FRONTEND_URL")
            .unwrap_or_else(|_| String::from("https://skynet.infinitybots.gg"));
        let client_id = env::var("DISCORD_CLIENT_ID")
            .or_else(|_| env::var("CLIENT_ID"))
            .unwrap_or_default();
        let client_secret = env::var("DISCORD_CLIENT_SECRET")
            .or_else(|_| env::var("CLIENT_SECRET"))
            .unwrap_or_default();

        if database_url.trim().is_empty()
            || token.trim().is_empty()
            || client_id.trim().is_empty()
            || client_secret.trim().is_empty()
        {
            return Err("Missing required Discord bot config values in environment variables. Required: DATABASE_URL, DISCORD_TOKEN, DISCORD_CLIENT_ID, DISCORD_CLIENT_SECRET".into());
        }

        Ok(Self {
            database_url,
            token,
            proxy_url,
            frontend_url,
            owners: vec![
                UserId::new(728871946456137770),
                UserId::new(510065483693817867),
            ],
            client_id,
            client_secret,
        })
    }

    pub fn load() -> Result<Self, Error> {
        Self::load_dotenv();

        if std::path::Path::new("config.yaml").exists() {
            let file = File::open("config.yaml")?;
            let cfg: Config = serde_yaml::from_reader(file)?;
            return Ok(cfg);
        }

        if env::var("DATABASE_URL").is_ok()
            || env::var("DISCORD_TOKEN").is_ok()
            || env::var("TOKEN").is_ok()
            || env::var("DISCORD_CLIENT_ID").is_ok()
            || env::var("CLIENT_ID").is_ok()
            || env::var("DISCORD_CLIENT_SECRET").is_ok()
            || env::var("CLIENT_SECRET").is_ok()
            || env::var("FRONTEND_URL").is_ok()
        {
            return Self::load_from_env();
        }

        // Delete config.yaml.sample if it exists
        if std::path::Path::new("config.yaml.sample").exists() {
            std::fs::remove_file("config.yaml.sample")?;
        }

        // Create config.yaml.sample
        let mut sample = File::create("config.yaml.sample")?;

        // Write default config to config.yaml.sample
        sample.write_all(serde_yaml::to_string(&Config::default())?.as_bytes())?;

        println!("No config.yaml or supported environment variables were found. A sample config has been created at config.yaml.sample");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use std::env;

    #[test]
    fn config_loads_from_env_when_yaml_missing() {
        env::set_var("DATABASE_URL", "postgres://example/db");
        env::set_var("DISCORD_TOKEN", "token-from-env");
        env::set_var("FRONTEND_URL", "https://example.com");
        env::set_var("DISCORD_CLIENT_ID", "client-from-env");
        env::set_var("DISCORD_CLIENT_SECRET", "secret-from-env");
        env::set_var("PROXY_URL", "");

        let cfg = Config::load_from_env().unwrap();

        assert_eq!(cfg.database_url, "postgres://example/db");
        assert_eq!(cfg.token, "token-from-env");
        assert_eq!(cfg.frontend_url, "https://example.com");
        assert_eq!(cfg.client_id, "client-from-env");
        assert_eq!(cfg.client_secret, "secret-from-env");
        assert_eq!(cfg.proxy_url, "");
    }
}
