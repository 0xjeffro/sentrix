use std::time::Duration;
use crate::config::Settings;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub http_client: reqwest::Client,
}


impl AppState {
    pub fn new(settings: &Settings) -> Self {
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(settings.http_client.pool_max_idle_per_host)
            .timeout(Duration::from_secs(settings.http_client.timeout_secs))
            .connect_timeout(Duration::from_secs(settings.http_client.connect_timeout_secs))
            .pool_idle_timeout(Duration::from_secs(settings.http_client.pool_idle_timeout_secs))
            .build()
            .unwrap_or_else(
                |err| {
                    eprintln!("Error creating HTTP client: {}", err);
                    std::process::exit(1);
                },
            );
        
        AppState { settings: settings.clone(), http_client }
    }
}