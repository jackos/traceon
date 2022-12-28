#![doc = include_str!("../README.md")]
mod formatting;
pub use formatting::Traceon;

pub use tracing;
use tracing::subscriber::DefaultGuard;
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
    let subscriber = Registry::default().with(traceon).with(env_filter);

    tracing::subscriber::set_global_default(subscriber)
        .expect("more than one global default subscriber set");
}

/// Use the defaults and set the global default subscriber
pub fn on_thread() -> DefaultGuard {
    let traceon = Traceon::new(std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(traceon).with(env_filter);

    tracing::subscriber::set_default(subscriber)
}

/// Use the defaults and set the global default subscriber with a custom filter
pub fn on_with<
    W: for<'a> MakeWriter<'a> + 'static + std::marker::Sync + std::marker::Send + Clone + Copy,
>(
    traceon: Traceon<W>,
) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(traceon).with(env_filter);

    // Panic straight away if user is trying to set two global default subscribers
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
