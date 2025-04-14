use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub app: App,
    pub backend: Backend,
    pub http_client: HttpClient,
    pub log: Log
}

#[derive(Deserialize, Clone)]
pub struct App {
    pub name: String,
    pub port : u16,
    pub secret_key: String,
}

#[derive(Deserialize, Clone)]
pub struct Backend {
    pub rpc_url: String,
    pub yellowstone_grpc_url: String,
    pub yellowstone_grpc_token: String,
}

#[derive(Deserialize, Clone)]
pub struct HttpClient {
    pub pool_max_idle_per_host: usize,
    pub timeout_secs: u64,
    pub connect_timeout_secs: u64,
    pub pool_idle_timeout_secs: u64,
}
#[derive(Deserialize, Clone)]
pub struct Log {
    pub file: String,
    pub level: String,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?.try_deserialize()
    }
}
