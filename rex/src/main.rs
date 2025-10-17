use clap::{Parser, Subcommand};

mod version;

/// Rex - Container Registry Explorer
///
/// A CLI tool for exploring and interacting with OCI-compliant container registries.
#[derive(Parser, Debug)]
#[command(name = "rex")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Registry URL (overrides config)
    #[arg(short, long, global = true, env = "REX_REGISTRY")]
    registry: Option<String>,

    /// Output format: pretty, json, yaml
    #[arg(
        short,
        long,
        global = true,
        default_value = "pretty",
        env = "REX_FORMAT"
    )]
    format: String,

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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Version => {
            version::print_version();
        }
    }
}
