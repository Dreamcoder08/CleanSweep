mod args;
mod commands;
mod output;
mod interactive;

use args::{Cli, Commands};
use clap::Parser;
use tracing::debug;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    setup_logging(cli.verbose);
    
    debug!("Dreamcoder CLI started");
    debug!("Command: {:?}", cli.command);
    
    // Ejecutar comando - clone cli to avoid move issues
    let cli_for_cmd = cli.clone();
    match cli.command {
        Commands::Apply(args) => commands::apply(args, cli_for_cmd).await?,
        Commands::List(args) => commands::list(args, cli_for_cmd).await?,
        Commands::Status(args) => commands::status(args, cli_for_cmd).await?,
        Commands::Backup(subcmd) => commands::backup(subcmd, cli_for_cmd).await?,
        Commands::Secret(subcmd) => commands::secret(subcmd, cli_for_cmd).await?,
        Commands::InstallDeps(args) => commands::install_deps(args, cli_for_cmd).await?,
        Commands::Update(args) => commands::update(args, cli_for_cmd).await?,
        Commands::Init(args) => commands::init(args, cli_for_cmd).await?,
        Commands::Interactive => interactive::run().await?,
    }
    
    Ok(())
}

fn setup_logging(verbosity: u8) {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("dreamcoder={}", level))
        .init();
}
