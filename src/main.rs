use clap::Parser;

use baza::Result;
use baza::cli::{Cli, Command};
use baza::commands;
use baza::config;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("baza: {e:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::load()?;
    match cli.command {
        Command::Run(args) => commands::run::execute(args, cfg).await,
        Command::Check(args) => commands::check::execute(args, cfg).await,
        Command::Dump(args) => commands::dump::execute(args, cfg).await,
        Command::Clean(args) => commands::clean::execute(args, cfg).await,
        Command::Config(args) => commands::config::execute(args, cfg).await,
    }
}
