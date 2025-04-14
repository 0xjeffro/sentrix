use std::sync::Arc;
use crate::app::logging::init_logger;
use crate::app::router::build_router;
use crate::app::state::AppState;
use crate::config::Settings;

pub async fn run_app(settings: Settings) {
    init_logger(&settings);
    let app_state = Arc::new(AppState::new(&settings));
    let app = build_router(app_state.clone());
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", settings.app.port))
        .await
        .unwrap_or_else(
            |err| {
                eprintln!("Error binding to address: {}", err);
                std::process::exit(1);
            },
        );

    println!("🚀 App '{}' is up and listening at 0.0.0.0:{}", settings.app.name, settings.app.port);

    axum::serve(listener, app)
        .await
        .unwrap_or_else(
            |err| {
                eprintln!("Error starting app: {}", err);
                std::process::exit(1);
            },
        );
}