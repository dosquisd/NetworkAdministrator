use std::io;
use tracing::Level;
use tracing_appender::rolling::daily;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn configure_global_tracing(log_level: Option<Level>) {
    let log_level = log_level.unwrap_or(Level::INFO);

    let file_appender = daily("./logs", "network_administrator.log");
    let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::from_default_env().add_directive(log_level.into());

    let file_layer = fmt::layer()
        .with_thread_ids(true)
        .with_line_number(false)
        .with_file(true)
        .with_ansi(false)
        .with_writer(non_blocking_file);

    let console_layer = fmt::layer()
        .pretty()
        .with_thread_ids(true)
        .with_line_number(false)
        .with_file(true)
        .with_writer(io::stdout);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    std::mem::forget(_guard);
}
