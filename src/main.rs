mod app;
mod mcp;
mod wheel;

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

    /// Run in auto-spin mode with transparent background
    #[arg(long)]
    auto_run: bool,

    /// Path to a .powertoysroulette file to open
    #[arg(value_name = "FILE")]
    file: Option<String>,
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
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .init();

    let cli = Cli::parse();

    // Handle file opening
    if let Some(file_path) = &cli.file {
        info!("Opening file: {}", file_path);
        // TODO: Load wheel configuration from the file
        // For now, just launch the GUI and log that a file was provided
        return app::launch();
    }

    match &cli.command {
        Some(Commands::Mcp) => {
            // Run in MCP mode
            mcp::run();
            Ok(())
        }
        None => {
            // Default: Launch the GUI application
            if cli.auto_run {
                info!("Launching GUI mode with auto-run");
                app::launch_auto_run()
            } else {
                info!("Launching GUI mode");
                app::launch()
            }
        }
    }
}
