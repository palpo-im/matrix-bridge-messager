use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing() {
    let format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}
