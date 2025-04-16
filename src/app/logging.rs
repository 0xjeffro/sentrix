use crate::config::Settings;
use std::fs::OpenOptions;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_logger(settings: &Settings) {
    let log_file = OpenOptions::new() // Prepare the log file
        .create(true) // Create the file if it doesn't exist
        .append(true) // Append to the file if it exists
        .open(&settings.log.file)
        .unwrap_or_else(|err| {
            eprintln!("Error opening log file: {}", err);
            std::process::exit(1);
        });

    let subscriber = tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(settings.log.level.parse().unwrap_or(tracing::Level::INFO));
    subscriber.json().init();
}
