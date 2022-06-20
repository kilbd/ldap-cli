use crate::config::{Config, ServerConfig};
use color_eyre::{
    eyre::{eyre, Result, WrapErr},
    owo_colors::OwoColorize,
};
use ldap3::{Ldap, LdapConnAsync};

pub async fn server_connection() -> Result<(Ldap, String)> {
    let config = Config::current()?;
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
    let config = Config::load()?;
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
    let mut password: Option<String> = None;
    let mut config = if Config::exists()? {
        Config::load()?
    } else {
        password = Some(prompt_for_password()?);
        Config::new()
    };
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
    config.save(password)?;
    println!("Switched to new server '{}'.", name.green().bold());
    Ok(())
}

pub fn rm(name: String) -> Result<()> {
    let mut config = Config::load()?;
    let index = config
        .servers
        .iter()
        .position(|item| item.name == name)
        .ok_or_else(|| {
            eyre!(
                "Unable to find the requested server configuration. Check the name in `{}`.",
                "ldap server list".green().bold(),
            )
        })?;
    let confirm = rprompt::prompt_reply_stderr(&format!(
        "\
{}{}{} (y/n)
>",
        "Are you sure you wish to remove the ".red(),
        name.green().bold(),
        " server configuration?".red(),
    ))
    .wrap_err("Failed to get confirmation for removal")?;
    match confirm.to_lowercase().as_ref() {
        "y" => {
            config.servers.remove(index);
            println!("Removed server configuration '{}'.", name.green().bold());
            if config.current == name {
                if !config.servers.is_empty() {
                    config.current = config.servers[0].name.clone();
                    println!(
                        "Switched to server '{}'.",
                        config.servers[0].name.green().bold()
                    );
                } else {
                    config.current = String::from("");
                    println!("There are no more server configurations. Please add one.");
                }
            }
            config.save(None)?;
        }
        "n" => (),
        _ => {
            println!("Did not understand your response. Please try again.");
        }
    }
    Ok(())
}

pub fn switch_to(name: String) -> Result<()> {
    let mut config = Config::load()?;
    if !config.servers.iter().any(|item| item.name == name) {
        return Err(
            eyre!("Unable to find the requested server configuration. Check the name in `{}` or add with `{}`.",
            "ldap server list".green().bold(),
            "ldap server add".green().bold())
        );
    }
    config.current = name.clone();
    config.save(None)?;
    println!("Switched to server '{}'.", name.green().bold());
    Ok(())
}

fn prompt_for_details(name: &str) -> Result<ServerConfig> {
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
        name: name.to_string(),
        password: pw,
        port,
        search_base: base,
        user,
    })
}

fn prompt_for_password() -> Result<String> {
    rpassword::prompt_password(format!(
        "\
To begin setup, enter a password to use for unlocking saved servers.
You will need to reconfigure servers if you lose this password.
{}
>",
        "Password:".blue().bold()
    ))
    .wrap_err("Failed to get password")
}
