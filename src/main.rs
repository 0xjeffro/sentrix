use crate::app::startup::run_app;
use crate::config::Settings;

mod app;
mod auth;
mod config;

#[tokio::main]
async fn main() {
    let settings = Settings::new() // Load settings
        .unwrap_or_else(|err| {
            eprintln!("Error loading settings: {}", err);
            std::process::exit(1);
        });

    run_app(settings).await;
}
