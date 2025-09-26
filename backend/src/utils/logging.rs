use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dwrcasts=info,tower_http=debug,auction_watcher=debug,metadata_updater=info,traits_processor=info,server=debug,sync_whitelist=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
