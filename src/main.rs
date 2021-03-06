use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use ldap_commands::{modify, search, search::Output, server};

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
    /// Format for search output
    #[clap(long, short, arg_enum)]
    format: Option<Output>,
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
        /// Remove an attribute value, or entirely if no value specified
        #[clap(long)]
        rm: bool,
        /// Replace existing values with the specified value(s)
        #[clap(long)]
        replace: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ServerCommand {
    /// Add new credentials for authenticating to a server
    Add {
        /// A name to use when referring to the new configuration
        name: String,
    },
    /// List configured servers
    List,
    /// Remove an existing server configuration
    Rm {
        /// The name of a saved configuration, as seen in `ldap server list`
        name: String,
    },
    /// Switch to using specified server for commands
    Use {
        /// The name of a saved configuration, as seen in `ldap server list`
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    match cli.cmd {
        Some(cmd) => match cmd {
            Command::Server { cmd: subcmd } => match subcmd {
                ServerCommand::Add { name } => server::add(name),
                ServerCommand::List => server::list(),
                ServerCommand::Rm { name } => server::rm(name),
                ServerCommand::Use { name } => server::switch_to(name),
            },
            Command::Modify {
                attr,
                value,
                dn,
                rm,
                replace,
            } => modify(dn, attr, value, rm, replace).await,
        },
        None => search(cli.filter, cli.attrs, cli.format).await,
    }
}
