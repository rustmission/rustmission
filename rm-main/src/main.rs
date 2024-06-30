pub mod app;
mod cli;
pub mod transmission;
pub mod tui;
mod ui;
mod utils;

use app::App;
use rm_config::Config;

use anyhow::Result;
use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let config = Config::init()?;

    if let Some(command) = args.command {
        cli::handle_command(&config, command).await?;
    } else {
        run_tui(config).await?;
    }

    Ok(())
}

async fn run_tui(config: Config) -> Result<()> {
    let mut app = App::new(config).await?;
    app.run().await?;
    Ok(())
}
