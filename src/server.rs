use color_eyre::{
    eyre::{eyre, Result, WrapErr},
    owo_colors::OwoColorize,
};
use directories::UserDirs;
use ldap3::{Ldap, LdapConnAsync};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Config {
    current: String,
    servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Deserialize)]
struct ServerConfig {
    host: String,
    name: String,
    password: String,
    port: u32,
    search_base: String,
    user: String,
}

pub async fn server_connection() -> Result<(Ldap, String)> {
    let config = current_config()?;
    let (conn, mut ldap) =
        LdapConnAsync::new(format!("ldaps://{}:{}", config.host, config.port).as_ref())
            .await
            .wrap_err(
                "Unable to connect to LDAP server. Please check your host and port configuration.",
            )?;
    ldap3::drive!(conn);
    ldap.simple_bind(config.user.as_ref(), config.password.as_ref())
        .await
        .wrap_err(
            "Unable to bind to LDAP server. Please check your user DN and password configuration.",
        )?;
    Ok((ldap, config.search_base))
}

fn config_path() -> Result<PathBuf> {
    let user_dirs =
        UserDirs::new().ok_or_else(|| eyre!("Could not find a home directory for you."))?;
    Ok(user_dirs.home_dir().join(".ldap").join("config"))
}

fn current_config() -> Result<ServerConfig> {
    let path = config_path()?;
    let contents = fs::read_to_string(path).wrap_err(format!(
        "Unable to read config file. Have you added a server with `{}`?",
        "ldap server add".green().bold()
    ))?;
    let full_config: Config =
        serde_json::from_str(contents.as_ref()).wrap_err("Unable to parse JSON in config.")?;
    Ok(full_config
        .servers
        .iter()
        .find(|item| item.name == full_config.current)
        .ok_or_else(|| eyre!("Could not find the server configuration to use. You may need to add a config with `{}` or select a different one with `{}`",
            "ldap server add".green().bold(), "ldap server use".green().bold()))?
        .clone())
}
