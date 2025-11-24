use tracing::{Level};
use tracing_subscriber::FmtSubscriber;

pub fn configure_global_tracing(log_level: Option<Level>) {
    let log_level = log_level.unwrap_or(Level::INFO);

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_thread_ids(true)
        .with_line_number(false)
        .with_max_level(log_level)
        .with_file(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
