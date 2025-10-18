mod app;
mod mcp;

use clap::{Parser, Subcommand};
use log::info;

/// PowerToys Roulette - A prize wheel application launcher
#[derive(Parser)]
#[command(name = "PowerToys Roulette")]
#[command(version = "0.1.0")]
#[command(about = "Launch applications using a fun prize wheel interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in MCP (Model Context Protocol) mode
    Mcp,
}

fn main() -> Result<(), eframe::Error> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Mcp) => {
            // Run in MCP mode
            mcp::run();
            Ok(())
        }
        None => {
            // Default: Launch the GUI application
            info!("Launching GUI mode");
            app::launch()
        }
    }
}
