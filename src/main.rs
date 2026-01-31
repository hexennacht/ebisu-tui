#![allow(dead_code)]

mod action;
mod app;
mod database;
mod error;
mod models;
mod state;
mod tui;

use app::App;

#[tokio::main]
async fn main() -> error::Result<()> {
    // Initialize and run the TUI application
    let mut app = App::new().await?;
    app.run().await?;
    Ok(())
}
