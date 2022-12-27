#![doc = include_str!("../README.md")]
mod formatting;
mod storage;

pub use formatting::Traceon;
pub use storage::{JsonStorage, StorageLayer};

pub use tracing;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Copy, Clone)]
pub enum Level {
    Off,
    Text,
    Number,
}

/// Use the defaults and set the global default subscriber
pub fn on() {
    let traceon = Traceon::new(std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default()
        .with(StorageLayer)
        .with(traceon)
        .with(env_filter);

    // Panic straight away if user is trying to set two global default subscribers
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

/// Use the defaults and set the global default subscriber with a custom filter
pub fn on_with<
    W: for<'a> MakeWriter<'a> + 'static + std::marker::Sync + std::marker::Send + Clone + Copy,
>(
    traceon: Traceon<W>,
) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default()
        .with(StorageLayer)
        .with(traceon)
        .with(env_filter);

    // Panic straight away if user is trying to set two global default subscribers
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
