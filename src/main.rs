use std::fs::OpenOptions;
use axum::routing::post;
use crate::config::Settings;
use crate::handler::proxy_handler;

mod config;
mod handler;

#[tokio::main]
async fn main() {
    let settings = Settings::new() // Load settings
        .unwrap_or_else(
            |err| {
                eprintln!("Error loading settings: {}", err);
                std::process::exit(1);
            },
        );
    
    let log_file = OpenOptions::new() // Prepare the log file
        .create(true) // Create the file if it doesn't exist
        .append(true) // Append to the file if it exists
        .open(&settings.log.file)
        .unwrap_or_else(
            |err| {
                eprintln!("Error opening log file: {}", err);
                std::process::exit(1);
            },
        );
    
    let subscriber = tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_max_level(settings.log.level.parse().unwrap_or(tracing::Level::INFO));
    subscriber.json().init();
    
    let app = axum::Router::new()
        .route("/", post(proxy_handler))
        .with_state(settings.clone());
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", settings.port))
        .await
        .unwrap_or_else(
            |err| {
                eprintln!("Error binding to address: {}", err);
                std::process::exit(1);
            },
        );
    
    println!("🚀 App '{}' is up and listening at 0.0.0.0:{}", settings.app_name, settings.port);
    
    axum::serve(listener, app)
        .await
        .unwrap_or_else(
            |err| {
                eprintln!("Error starting server: {}", err);
                std::process::exit(1);
            },
        );
}
