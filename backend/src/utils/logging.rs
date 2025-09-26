use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "together=info,tower_http=debug,attestation_watcher=debug,server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
