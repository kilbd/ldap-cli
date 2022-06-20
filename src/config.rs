use color_eyre::{
    eyre::{eyre, Result, WrapErr},
    owo_colors::OwoColorize,
};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub current: String,
    pub servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub name: String,
    pub password: String,
    pub port: u32,
    pub search_base: String,
    pub user: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            current: String::from("none"),
            servers: vec![],
        }
    }

    pub fn exists() -> Result<bool> {
        Ok(config_path()?.join("config").exists())
    }

    pub fn load() -> Result<Config> {
        let path = config_path()?.join("config");
        let contents = fs::read_to_string(path).wrap_err(format!(
            "Unable to read config file. Have you added a server with `{}`?",
            "ldap server add".green().bold()
        ))?;
        serde_json::from_str(contents.as_ref()).wrap_err("Unable to parse JSON in config.")
    }

    pub fn save(&self, password: Option<String>) -> Result<()> {
        let new_contents = serde_json::to_string_pretty(&self)
            .wrap_err("Unable to save configuration. Please try again.")?;
        let path = config_path()?.join("config");
        fs::write(path, new_contents).wrap_err("Unable to save new server. Please try again.")
    }

    pub fn current() -> Result<ServerConfig> {
        let full_config = Self::load()?;
        Ok(full_config
            .servers
            .iter()
            .find(|item| item.name == full_config.current)
            .ok_or_else(|| eyre!(
                "Could not find the server configuration to use. You may need to add a config with `{}` or select a different one with `{}`",
                "ldap server add".green().bold(),
                "ldap server use".green().bold()))?
            .clone())
    }
}

fn config_path() -> Result<PathBuf> {
    let user_dirs =
        UserDirs::new().ok_or_else(|| eyre!("Could not find a home directory for you."))?;
    Ok(user_dirs.home_dir().join(".ldap"))
}
