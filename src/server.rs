use directories::UserDirs;
use ldap3::result::Result;
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
    let config = current_config();
    let (conn, mut ldap) =
        LdapConnAsync::new(format!("ldaps://{}:{}", config.host, config.port).as_ref()).await?;
    ldap3::drive!(conn);
    ldap.simple_bind(config.user.as_ref(), config.password.as_ref())
        .await?;
    Ok((ldap, config.search_base))
}

fn config_path() -> PathBuf {
    let user_dirs = UserDirs::new().unwrap();
    user_dirs.home_dir().join(".ldap").join("config")
}

fn current_config() -> ServerConfig {
    let path = config_path();
    let contents = fs::read_to_string(path).unwrap();
    let full_config: Config = serde_json::from_str(contents.as_ref()).unwrap();
    full_config
        .servers
        .iter()
        .find(|item| item.name == full_config.current)
        .unwrap()
        .clone()
}
