use crate::config::Settings;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub http_client: reqwest::Client,
    pub user_rate_limit_state: Arc<DashMap<String, RateLimitState>>,
}

#[derive(Clone)]
pub struct RateLimitState {
    pub request_count: u64,
    pub last_request_time: Instant,
}

impl AppState {
    pub fn new(settings: &Settings) -> Self {
        let http_client = reqwest::Client::builder()
            .pool_max_idle_per_host(settings.http_client.pool_max_idle_per_host)
            .timeout(Duration::from_secs(settings.http_client.timeout_secs))
            .connect_timeout(Duration::from_secs(
                settings.http_client.connect_timeout_secs,
            ))
            .pool_idle_timeout(Duration::from_secs(
                settings.http_client.pool_idle_timeout_secs,
            ))
            .build()
            .unwrap_or_else(|err| {
                eprintln!("Error creating HTTP client: {}", err);
                std::process::exit(1);
            });

        AppState {
            settings: settings.clone(),
            http_client,
            user_rate_limit_state: Arc::new(DashMap::new()),
        }
    }

    pub fn update_and_check_rate_limit(&self, user_id: &str, user_qps: u32) -> bool {
        let now = Instant::now();
        if let Some(rate_limit_state) = self.user_rate_limit_state.get(user_id) {
            let elapsed = now.duration_since(rate_limit_state.last_request_time);
            if elapsed <= Duration::from_secs(1)
                && rate_limit_state.request_count + 1 > user_qps as u64
            {
                return false; // Rate limit exceeded
            }
        }

        let mut rate_limit_state = self
            .user_rate_limit_state
            .entry(user_id.to_string())
            .or_insert(RateLimitState {
                request_count: 0,
                last_request_time: now,
            });
        let elapsed = now.duration_since(rate_limit_state.last_request_time);
        if elapsed >= Duration::from_secs(1) {
            rate_limit_state.request_count = 1;
            rate_limit_state.last_request_time = now;
            true
        } else {
            rate_limit_state.request_count += 1;
            if rate_limit_state.request_count > user_qps as u64 {
                false
            } else {
                true
            }
        }
    }
}
