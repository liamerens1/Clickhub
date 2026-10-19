use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

const KEEP_LOG_FILES: usize = 7;

/// Installs the global tracing subscriber: stderr, plus daily-rotated files in `log_dir`
/// (`clickhub.YYYY-MM-DD.log`, last 7 kept). Honours `RUST_LOG`, defaults to `info`.
///
/// Returns the file writer's guard; keep it alive until exit so buffered lines are flushed.
pub fn init(log_dir: Option<&Path>) -> Option<WorkerGuard> {
    let filter = || EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stderr = fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(filter());

    let mut file_error = None;
    let (file, guard) = match log_dir.map(open_appender) {
        Some(Ok(appender)) => {
            let (writer, guard) = tracing_appender::non_blocking(appender);
            let layer = fmt::layer()
                .with_ansi(false)
                .with_writer(writer)
                .with_filter(filter());
            (Some(layer), Some(guard))
        }
        Some(Err(error)) => {
            file_error = Some(error);
            (None, None)
        }
        None => (None, None),
    };

    // try_init: a second call is a no-op instead of a panic.
    let _ = tracing_subscriber::registry()
        .with(stderr)
        .with(file)
        .try_init();

    match (log_dir, file_error) {
        (Some(dir), None) => tracing::info!(dir = %dir.display(), "logging to file"),
        (Some(dir), Some(error)) => {
            tracing::warn!(dir = %dir.display(), %error, "file logging unavailable");
        }
        (None, _) => tracing::warn!("no log directory; logging to stderr only"),
    }
    guard
}

fn open_appender(
    dir: &Path,
) -> Result<tracing_appender::rolling::RollingFileAppender, tracing_appender::rolling::InitError> {
    Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("clickhub")
        .filename_suffix("log")
        .max_log_files(KEEP_LOG_FILES)
        .build(dir)
}
