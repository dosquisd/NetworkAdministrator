use std::io;

use time::macros::format_description;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::LocalTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::cli::types::{LogFormat, LogLevel};

pub struct LogConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub file_path: Option<String>,
    pub max_log_files: Option<usize>,
}

pub fn configure_global_tracing(config: LogConfig) {
    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ));

    let level = config.level.as_tracing_level();
    let filter = EnvFilter::from_default_env()
        .add_directive(format!("network_administrator={}", level).parse().unwrap())
        .add_directive("trust_dns_proto=warn".parse().unwrap())
        .add_directive("trust_dns_resolver=warn".parse().unwrap())
        .add_directive("tokio=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("h2=warn".parse().unwrap());

    let registry = tracing_subscriber::registry().with(filter);
    let max_files = config.max_log_files.unwrap_or(7);

    match config.format {
        LogFormat::Pretty => {
            let console_layer = fmt::layer()
                .pretty()
                .with_thread_ids(true)
                .with_line_number(false)
                .with_file(true)
                .with_timer(timer.clone())
                .with_writer(io::stdout);

            if let Some(file_path) = config.file_path {
                let file_appender = RollingFileAppender::builder()
                    .rotation(Rotation::DAILY)
                    .filename_prefix(&file_path)
                    .filename_suffix("log")
                    .max_log_files(max_files)
                    .build("./logs")
                    .expect("Failed to created rolling file appender");
                let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

                let file_layer = fmt::layer()
                    .with_thread_ids(true)
                    .with_ansi(false)
                    .with_timer(timer.clone())
                    .with_writer(non_blocking_file);

                registry.with(console_layer).with(file_layer).init();

                std::mem::forget(_guard);
            } else {
                registry.with(console_layer).init();
            }
        }
        LogFormat::Json => {
            let console_layer = fmt::layer()
                .json()
                .with_timer(timer.clone())
                .with_writer(io::stdout);

            if let Some(file_path) = config.file_path {
                let file_appender = RollingFileAppender::builder()
                    .rotation(Rotation::DAILY)
                    .filename_prefix(&file_path)
                    .filename_suffix("log")
                    .max_log_files(max_files)
                    .build("./logs")
                    .expect("Failed to created rolling file appender");
                let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

                let file_layer = fmt::layer()
                    .json()
                    .with_timer(timer.clone())
                    .with_writer(non_blocking_file);

                registry.with(console_layer).with(file_layer).init();

                std::mem::forget(_guard);
            } else {
                registry.with(console_layer).init();
            }
        }
        LogFormat::Compact => {
            let console_layer = fmt::layer()
                .compact()
                .with_timer(timer.clone())
                .with_writer(io::stdout);

            if let Some(file_path) = config.file_path {
                let file_appender = RollingFileAppender::builder()
                    .rotation(Rotation::DAILY)
                    .filename_prefix(&file_path)
                    .filename_suffix("log")
                    .max_log_files(max_files)
                    .build("./logs")
                    .expect("Failed to created rolling file appender");
                let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);

                let file_layer = fmt::layer()
                    .compact()
                    .with_ansi(false)
                    .with_timer(timer.clone())
                    .with_writer(non_blocking_file);

                registry.with(console_layer).with(file_layer).init();

                std::mem::forget(_guard);
            } else {
                registry.with(console_layer).init();
            }
        }
    }
}
