#![doc = include_str!("../README.md")]
mod traceon;
pub use crate::traceon::Traceon;

pub use tracing;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Copy, Clone)]
pub enum LevelFormat {
    Off,
    Text,
    Number,
}

/// Use the defaults and set the global default subscriber
#[must_use]
pub fn builder() -> Traceon {
    Traceon::default()
}

/// Use the defaults and set the global default subscriber
pub fn on() {
    let traceon = Traceon::default();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(traceon).with(env_filter);

    tracing::subscriber::set_global_default(subscriber)
        .expect("more than one global default subscriber set");
}

/// Use the defaults and set the global default subscriber
pub fn try_on() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let traceon = Traceon::default();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(traceon).with(env_filter);

    tracing::subscriber::set_global_default(subscriber)
}

/// Use the defaults and set the global default subscriber
pub fn on_thread() -> DefaultGuard {
    let traceon = Traceon::default();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(traceon).with(env_filter);

    tracing::subscriber::set_default(subscriber)
}
