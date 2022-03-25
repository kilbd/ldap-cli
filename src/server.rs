use color_eyre::{
    eyre::{eyre, Result, WrapErr},
    owo_colors::OwoColorize,
};
use directories::UserDirs;
use ldap3::{Ldap, LdapConnAsync};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    current: String,
    servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

pub fn list() -> Result<()> {
    let config = load_config()?;
    for server in config.servers.iter() {
        if server.name == config.current {
            println!("{}", format!("* {}", server.name).green().bold());
        } else {
            println!("  {}", server.name);
        }
    }
    Ok(())
}

pub fn add(name: String) -> Result<()> {
    let mut config = load_config()?;
    if config.servers.iter().any(|item| item.name == name) {
        return Err(eyre!(
            "You already saved a server named '{}'. Please choose a different name.",
            name.green()
        ));
    }
    let server_config = prompt_for_details(&name)?;
    config.servers.push(server_config);
    config.servers.sort_by(|a, b| a.name.cmp(&b.name));
    config.current = name.clone();
    let new_contents = serde_json::to_string_pretty(&config)
        .wrap_err("Unable to save configuration. Please try again.")?;
    let path = config_path()?;
    fs::write(path, new_contents).wrap_err("Unable to save new server. Please try again.")?;
    println!("Switched to new server '{}'", name.green().bold());
    Ok(())
}

fn config_path() -> Result<PathBuf> {
    let user_dirs =
        UserDirs::new().ok_or_else(|| eyre!("Could not find a home directory for you."))?;
    Ok(user_dirs.home_dir().join(".ldap").join("config"))
}

fn load_config() -> Result<Config> {
    let path = config_path()?;
    let contents = fs::read_to_string(path).wrap_err(format!(
        "Unable to read config file. Have you added a server with `{}`?",
        "ldap server add".green().bold()
    ))?;
    serde_json::from_str(contents.as_ref()).wrap_err("Unable to parse JSON in config.")
}

fn current_config() -> Result<ServerConfig> {
    let full_config = load_config()?;
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

fn prompt_for_details(name: &String) -> Result<ServerConfig> {
    let host = rprompt::prompt_reply_stderr(&format!(
        "\
Please specify the following details for server connections:
{} (example: ldap.example.com)
>",
        "Host".blue().bold()
    ))
    .wrap_err("Failed to get host location")?;
    let port = rprompt::prompt_reply_stderr(&format!(
        "\
{} (example: 636)
>",
        "Port".blue().bold()
    ))
    .wrap_err("Failed to get port")?
    .parse::<u32>()
    .wrap_err("Please enter a number for the port")?;
    let user = rprompt::prompt_reply_stderr(&format!(
        "\
{} (example: cn=Me,ou=Admin,o=MyOrg,c=US)
>",
        "Bind DN".blue().bold()
    ))
    .wrap_err("Failed to get bind DN")?;
    let pw = rpassword::prompt_password(format!(
        "\
{}
>",
        "Password".blue().bold()
    ))
    .wrap_err("Failed to get password")?;
    let base = rprompt::prompt_reply_stderr(&format!(
        "\
{} (example: o=MyOrg,c=US)
>",
        "Search Base".blue().bold()
    ))
    .wrap_err("Failed to get search base")?;
    Ok(ServerConfig {
        host,
        name: name.clone(),
        password: pw,
        port,
        search_base: base,
        user,
    })
}
