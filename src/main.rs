use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use ldap_commands::search;

/// A CLI for interacting with an LDAP server.
#[derive(Debug, Parser)]
#[clap(name = "ldap")]
struct Cli {
    #[clap(subcommand)]
    cmd: Option<Command>,
    /// Attributes to output in response (outputs all attributes if not specified)
    #[clap(short, long)]
    attrs: Option<String>,
    /// QUOTED string to use for LDAP filter in a search
    filter: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Manage LDAP servers
    Server {
        #[clap(subcommand)]
        cmd: ServerCommand,
    },
    /// Modify a single attribute for an account
    Modify {
        dn: String,
        /// Attribute to modify
        #[clap(long, short)]
        attr: String,
        /// New value for attribute (can be used multiple times for multi-value attributes)
        #[clap(long, short)]
        value: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ServerCommand {
    /// Add new credentials for authenticating to a server
    Add,
    /// Modify an existing server configuration
    Edit,
    /// List configured servers
    List,
    /// Switch to using specified server for commands
    Use { name: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    match cli.cmd {
        Some(cmd) => match cmd {
            Command::Server { cmd: subcmd } => {
                println!("Run session to {subcmd:?} a server");
                Ok(())
            }
            Command::Modify { attr, value, dn: _ } => {
                println!("Modify attribute '{attr}' to be {value:?}");
                Ok(())
            }
        },
        None => search(cli.filter, cli.attrs).await,
    }
}
