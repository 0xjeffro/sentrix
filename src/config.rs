use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub app_name: String,
    pub port: u16,
    pub backend: Backend,
    pub log: Log
}


#[derive(Deserialize, Clone)]
pub struct Backend {
    pub rpc_url: String,
    pub yellowstone_grpc_url: String,
    pub yellowstone_grpc_token: String,
}
#[derive(Deserialize, Clone)]
pub struct Log {
    pub file: String,
    pub level: String,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .build()?.try_deserialize()
    }
}
