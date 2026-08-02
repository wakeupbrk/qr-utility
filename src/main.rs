use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod generator;
mod models;
mod services;
mod storage;
mod ui;
mod utils;

use cli::Cli;
use ui::App;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        // Execute headless CLI subcommand
        Cli::execute_command(command).await?;
    } else {
        // Launch interactive Ratatui TUI application
        let terminal = ratatui::init();
        let mut app = App::new();
        let result = app.run(terminal).await;
        ratatui::restore();

        if let Err(err) = result {
            eprintln!("Application Error: {:?}", err);
        }
    }

    Ok(())
}
