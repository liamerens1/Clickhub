use tracing_subscriber::EnvFilter;

/// Installs the global tracing subscriber. Honours `RUST_LOG`, defaults to `info`.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // try_init: a second call is a no-op instead of a panic.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
