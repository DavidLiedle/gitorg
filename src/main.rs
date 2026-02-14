mod commands;
mod config;
mod display;
mod error;
mod github;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "gitorg",
    version,
    about = "Manage and monitor multiple GitHub organizations"
)]
pub struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Show verbose output (rate limits, debug info)
    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with a GitHub personal access token
    Auth {
        /// Token to use (if omitted, prompts interactively)
        #[arg(long)]
        token: Option<String>,
    },
    /// List your GitHub organizations
    Orgs,
    /// List repositories across organizations
    Repos {
        /// Filter to a specific organization
        #[arg(long)]
        org: Option<String>,
        /// Sort by: activity, stars, staleness, name
        #[arg(long, default_value = "activity")]
        sort: String,
    },
    /// Find stale repositories with no recent pushes
    Stale {
        /// Filter to a specific organization
        #[arg(long)]
        org: Option<String>,
        /// Number of days without a push to consider stale
        #[arg(long, default_value = "90")]
        days: u64,
    },
    /// List open issues across organizations
    Issues {
        /// Filter to a specific organization
        #[arg(long)]
        org: Option<String>,
    },
    /// Show aggregate statistics across organizations
    Stats {
        /// Filter to a specific organization
        #[arg(long)]
        org: Option<String>,
    },
    /// Show a full dashboard overview
    Overview {
        /// Filter to a specific organization
        #[arg(long)]
        org: Option<String>,
        /// Days threshold for stale repos in overview
        #[arg(long, default_value = "90")]
        days: u64,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Auth { token } => commands::auth::run(token).await,
        Commands::Orgs => commands::orgs::run(cli.json, cli.verbose).await,
        Commands::Repos { org, sort } => {
            commands::repos::run(org, sort, cli.json, cli.verbose).await
        }
        Commands::Stale { org, days } => {
            commands::stale::run(org, *days, cli.json, cli.verbose).await
        }
        Commands::Issues { org } => commands::issues::run(org, cli.json, cli.verbose).await,
        Commands::Stats { org } => commands::stats::run(org, cli.json, cli.verbose).await,
        Commands::Overview { org, days } => {
            commands::overview::run(org, *days, cli.json, cli.verbose).await
        }
    };

    if let Err(e) = result {
        display::error(&e.to_string());
        std::process::exit(1);
    }
}
