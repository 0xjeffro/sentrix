use crate::config::Settings;
use dashmap::DashMap;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub http_client: reqwest::Client,
    pub user_rate_limit_state: DashMap<String, RateLimitState>,
    pub user_rpc_method_state: DashMap<String, UserRpcMethodState>, // user_id -> RpcMethodState
}

#[derive(Clone)]
pub struct RateLimitState {
    pub request_count: u64,
    pub last_request_time: Instant,
}
#[derive(Clone)]
pub struct UserRpcMethodState {
    pub user_id: String,
    pub last_log_time: Instant, // Time when this user state was last written to log
    pub rpc_method_state: DashMap<String, RpcMethodState>, // rpc_method -> RpcMethodState
}

impl UserRpcMethodState {
    pub fn new(user_id: &str) -> Self {
        UserRpcMethodState {
            user_id: user_id.to_string(),
            last_log_time: Instant::now(),
            rpc_method_state: DashMap::new(),
        }
    }
    
    pub fn log_if_needed(&mut self, user_rpc_log_interval: u64) {
        // Log the state of this user
        let now = Instant::now();
        if now.duration_since(self.last_log_time).as_secs() > user_rpc_log_interval {
            // Log the state
            for pair in self.rpc_method_state.iter() {
                let method = pair.key();
                let state = pair.value();
                info!(
                    event = "user_rpc_analysis",
                    user = self.user_id,
                    method = method,
                    count = state.request_count,
                    mean = state.mean_response_time,
                    max = state.max_response_time,
                    min = state.min_response_time,
                    std = state.std_response_time,
                );
            }
            self.last_log_time = now;
            self.rpc_method_state = DashMap::new();
        }
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct RpcMethodState {
    pub method: String,
    pub request_count: u64,
    // All times are in milliseconds
    pub mean_response_time: f64, 
    pub max_response_time: f64,
    pub min_response_time: f64,
    pub std_response_time: f64,
    pub m2: f64, // For variance calculation
}

impl RpcMethodState {
    pub fn new(method: &str) -> Self {
        RpcMethodState {
            method: method.to_string(),
            request_count: 0,
            mean_response_time: 0.0,
            max_response_time: 0.0,
            min_response_time: f64::MAX,
            std_response_time: 0.0,
            m2: 0.0,
        }
    }
    
    pub fn update(&mut self, response_time: f64) {
        self.request_count += 1;
        if response_time > self.max_response_time {
            self.max_response_time = response_time;
        }
        if response_time < self.min_response_time {
            self.min_response_time = response_time;
        }
        
        // Using Welford's algorithm for online mean and variance calculation
        let delta = response_time - self.mean_response_time;
        self.mean_response_time += delta / self.request_count as f64;
        
        let delta2 = response_time - self.mean_response_time;
        self.m2 += delta * delta2;
        if self.request_count <= 1 {
            self.std_response_time = 0.0;
        } else {
            self.std_response_time = (self.m2 / (self.request_count - 1) as f64).sqrt();
        }
    }
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
            user_rate_limit_state: DashMap::new(),
            user_rpc_method_state: DashMap::new(),
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
    
    pub fn update_and_log_rpc_method_state(
        &self,
        user_id: &str,
        rpc_method: &str,
        response_time: f64,
    ) {
        let mut user_rpc_method_state = self
            .user_rpc_method_state
            .entry(user_id.to_string())
            .or_insert(UserRpcMethodState::new(user_id));

        {
            let mut rpc_method_state = user_rpc_method_state
                .rpc_method_state
                .entry(rpc_method.to_string())
                .or_insert(RpcMethodState::new(rpc_method));

            rpc_method_state.update(response_time);
        }
        user_rpc_method_state.log_if_needed(self.settings.log.user_rpc_log_interval);
    }
}
