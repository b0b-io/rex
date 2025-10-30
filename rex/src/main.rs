use clap::{CommandFactory, Parser, Subcommand};

mod commands;
mod config;
mod context;
mod format;

/// Rex - Container Registry Explorer
///
/// A CLI tool for exploring and interacting with OCI-compliant container registries.
#[derive(Parser, Debug)]
#[command(name = "rex")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Verbose output (can be repeated: -v, -vv, -vvv)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Control colored output: auto, always, never
    #[arg(long, global = true, default_value = "auto")]
    color: String,

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
    /// Explore images and their details
    Image {
        #[command(subcommand)]
        command: ImageCommands,
    },
    /// Search for images and tags
    Search {
        /// Search query
        query: String,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
        /// Limit number of results per category
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Generate shell completion scripts
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand, Debug)]
enum ImageCommands {
    /// List all images in the registry
    #[command(visible_alias = "ls")]
    List {
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
        /// Show only image names
        #[arg(short, long)]
        quiet: bool,
        /// Filter by pattern (supports fuzzy matching)
        #[arg(long)]
        filter: Option<String>,
        /// Limit number of results
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List tags for a specific image
    Tags {
        /// Image name (repository)
        name: String,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
        /// Show only tag names
        #[arg(short, long)]
        quiet: bool,
        /// Filter by pattern (supports fuzzy matching)
        #[arg(long)]
        filter: Option<String>,
        /// Limit number of results
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Show brief details about an image
    Show {
        /// Image reference (name:tag or name@digest)
        reference: String,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Show complete detailed inspection of an image
    Inspect {
        /// Image reference (name:tag or name@digest)
        reference: String,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
        /// Inspect specific platform (for multi-arch images)
        #[arg(long)]
        platform: Option<String>,
        /// Show raw manifest JSON
        #[arg(long)]
        raw_manifest: bool,
        /// Show raw config JSON
        #[arg(long)]
        raw_config: bool,
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
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
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
    /// Manage registry cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand, Debug)]
enum CacheCommands {
    /// Show cache statistics
    Stats {
        /// Registry name (optional, uses default if omitted)
        name: Option<String>,
        /// Output format: pretty, json, yaml
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
    /// Clear cache entries
    Clear {
        /// Registry name (optional, uses default if omitted)
        name: Option<String>,
        /// Clear cache for all registries
        #[arg(long)]
        all: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Remove expired cache entries
    Prune {
        /// Registry name (optional, uses default if omitted)
        name: Option<String>,
        /// Prune cache for all registries
        #[arg(long)]
        all: bool,
        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,
    },
    /// Pre-populate cache by syncing registry metadata
    Sync {
        /// Registry name (optional, uses default if omitted)
        name: Option<String>,
        /// Also fetch and cache image manifests and config blobs
        #[arg(long)]
        manifests: bool,
        /// Sync cache for all registries
        #[arg(long)]
        all: bool,
        /// Re-fetch even if entries exist in cache
        #[arg(short, long)]
        force: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Build context with precedence: defaults > config file > env vars > CLI flags
    let ctx = context::AppContext::build(
        format::ColorChoice::from(cli.color.as_str()),
        context::VerbosityLevel::from_count(cli.verbose),
    );

    match cli.command {
        Commands::Version => {
            commands::version::print_version();
        }
        Commands::Config { command } => match command {
            ConfigCommands::Init => commands::config::handle_init(&ctx),
            ConfigCommands::Get { key, format } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::config::handle_get(&ctx, key.as_deref(), fmt);
            }
            ConfigCommands::Set { key, value } => {
                commands::config::handle_set(&ctx, key.as_deref(), value.as_deref());
            }
            ConfigCommands::Edit => {
                commands::config::handle_set(&ctx, None, None);
            }
        },
        Commands::Registry { command } => match command {
            RegistryCommands::Init { name, url } => {
                commands::registry::handlers::handle_registry_init(&ctx, &name, &url);
            }
            RegistryCommands::List { format } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::registry::handlers::handle_registry_list(&ctx, fmt);
            }
            RegistryCommands::Remove { name, force } => {
                commands::registry::handlers::handle_registry_remove(&ctx, &name, force);
            }
            RegistryCommands::Use { name } => {
                commands::registry::handlers::handle_registry_use(&ctx, &name);
            }
            RegistryCommands::Show { name, format } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::registry::handlers::handle_registry_show(&ctx, &name, fmt);
            }
            RegistryCommands::Check { name, format } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::registry::handlers::handle_registry_check(&ctx, &name, fmt);
            }
            RegistryCommands::Login {
                name,
                username,
                password,
            } => {
                commands::registry::handlers::handle_registry_login(
                    &ctx,
                    &name,
                    username.as_deref(),
                    password.as_deref(),
                );
            }
            RegistryCommands::Logout { name } => {
                commands::registry::handlers::handle_registry_logout(&ctx, &name);
            }
            RegistryCommands::Cache { command } => match command {
                CacheCommands::Stats { name, format } => {
                    let fmt = format::OutputFormat::from(format.as_str());
                    commands::registry::handlers::handle_cache_stats(&ctx, name.as_deref(), fmt);
                }
                CacheCommands::Clear { name, all, force } => {
                    commands::registry::handlers::handle_cache_clear(
                        &ctx,
                        name.as_deref(),
                        all,
                        force,
                    );
                }
                CacheCommands::Prune { name, all, dry_run } => {
                    commands::registry::handlers::handle_cache_prune(
                        &ctx,
                        name.as_deref(),
                        all,
                        dry_run,
                    );
                }
                CacheCommands::Sync {
                    name,
                    manifests,
                    all,
                    force,
                } => {
                    commands::registry::handlers::handle_cache_sync(
                        &ctx,
                        name.as_deref(),
                        manifests,
                        all,
                        force,
                    );
                }
            },
        },
        Commands::Image { command } => match command {
            ImageCommands::List {
                format,
                quiet,
                filter,
                limit,
            } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::image::handlers::handle_image_list(
                    &ctx,
                    fmt,
                    quiet,
                    filter.as_deref(),
                    limit,
                );
            }
            ImageCommands::Tags {
                name,
                format,
                quiet,
                filter,
                limit,
            } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::image::handlers::handle_image_tags(
                    &ctx,
                    name.as_str(),
                    fmt,
                    quiet,
                    filter.as_deref(),
                    limit,
                );
            }
            ImageCommands::Show { reference, format } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::image::handlers::handle_image_details(&ctx, reference.as_str(), fmt);
            }
            ImageCommands::Inspect {
                reference,
                format,
                platform,
                raw_manifest,
                raw_config,
            } => {
                let fmt = format::OutputFormat::from(format.as_str());
                commands::image::handlers::handle_image_inspect(
                    &ctx,
                    reference.as_str(),
                    fmt,
                    platform.as_deref(),
                    raw_manifest,
                    raw_config,
                );
            }
        },
        Commands::Search {
            query,
            format,
            limit,
        } => {
            let fmt = format::OutputFormat::from(format.as_str());
            commands::search::handlers::handle_search(&ctx, query.as_str(), fmt, limit);
        }
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }
}
