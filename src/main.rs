use crate::config::Settings;
use crate::app::startup::run_app;

mod config;
mod app;

#[tokio::main]
async fn main() {
    let settings = Settings::new() // Load settings
        .unwrap_or_else(
            |err| {
                eprintln!("Error loading settings: {}", err);
                std::process::exit(1);
            },
        );
    
    run_app(settings).await;
}
