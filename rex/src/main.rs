use clap::{Parser, Subcommand};

mod config;
mod output;
mod version;

/// Rex - Container Registry Explorer
///
/// A CLI tool for exploring and interacting with OCI-compliant container registries.
#[derive(Parser, Debug)]
#[command(name = "rex")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Display version information
    Version,
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Manage registries
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Initialize configuration with default values
    Init,
    /// Get a configuration value (or display all if no key provided)
    Get {
        /// Configuration key to get (e.g., style.format)
        key: Option<String>,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Set a configuration value (or open editor if no arguments)
    Set {
        /// Configuration key to set (e.g., style.format)
        key: Option<String>,
        /// Value to set
        value: Option<String>,
    },
    /// Edit configuration file in $EDITOR (alias for 'set' with no arguments)
    Edit,
}

#[derive(Subcommand, Debug)]
enum RegistryCommands {
    /// Initialize a new registry
    Init {
        /// Registry name
        name: String,
        /// Registry URL
        url: String,
    },
    /// List all registries
    #[command(visible_alias = "ls")]
    List {
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Remove a registry
    #[command(visible_alias = "rm")]
    Remove {
        /// Registry name
        name: String,
    },
    /// Set the default registry
    Use {
        /// Registry name
        name: String,
    },
    /// Show registry details
    Show {
        /// Registry name
        name: String,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Check registry connectivity and status
    Check {
        /// Registry name
        name: String,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Login to a registry
    Login {
        /// Registry name
        name: String,
        /// Username (will prompt if not provided)
        #[arg(short, long)]
        username: Option<String>,
        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Logout from a registry
    Logout {
        /// Registry name
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            version::print_version();
        }
        Commands::Config { command } => match command {
            ConfigCommands::Init => config::handle_init(),
            ConfigCommands::Get { key, format } => {
                let fmt = output::OutputFormat::from(format.as_str());
                config::handle_get(key.as_deref(), fmt);
            }
            ConfigCommands::Set { key, value } => {
                config::handle_set(key.as_deref(), value.as_deref());
            }
            ConfigCommands::Edit => {
                config::handle_set(None, None);
            }
        },
        Commands::Registry { command } => match command {
            RegistryCommands::Init { name, url } => {
                config::handle_registry_init(&name, &url);
            }
            RegistryCommands::List { format } => {
                let fmt = output::OutputFormat::from(format.as_str());
                config::handle_registry_list(fmt);
            }
            RegistryCommands::Remove { name } => {
                config::handle_registry_remove(&name);
            }
            RegistryCommands::Use { name } => {
                config::handle_registry_use(&name);
            }
            RegistryCommands::Show { name, format } => {
                let fmt = output::OutputFormat::from(format.as_str());
                config::handle_registry_show(&name, fmt);
            }
            RegistryCommands::Check { name, format } => {
                let fmt = output::OutputFormat::from(format.as_str());
                config::handle_registry_check(&name, fmt).await;
            }
            RegistryCommands::Login {
                name,
                username,
                password,
            } => {
                config::handle_registry_login(&name, username.as_deref(), password.as_deref())
                    .await;
            }
            RegistryCommands::Logout { name } => {
                config::handle_registry_logout(&name);
            }
        },
    }
}
