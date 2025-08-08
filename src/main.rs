use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod config;
mod parser;
mod theme;
mod tui;

use config::Config;
use parser::HyprlandParser;
use tui::App;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load configuration
    let config = Config::load(cli.config)?;
    
    // Parse Hyprland configuration files
    let parser = HyprlandParser::new(&config);
    let keybindings = parser.parse()?;
    
    // Start TUI
    let mut app = App::new(keybindings, &config);
    app.run()?;
    
    Ok(())
}