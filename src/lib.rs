#![doc = include_str!("../README.md")]
mod traceon;
pub use crate::traceon::{Case, LevelFormat, Traceon};

pub use tracing;
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

/// Returns a builder that can be configured before being turned on, or used as a layer for a subscriber.
/// All the options are shown in the example below.
/// ```
/// use traceon::{LevelFormat, Case};
/// traceon::builder()
///     // Turn off the default fields
///     .file(false)
///     .span(false)
///     .module(false)
///     .timestamp(false)
///     // Set the log level to text instead of numbers
///     .level(LevelFormat::Lowercase)
///     // Rename the json keys to match a case
///     .case(Case::Snake)
///     // Concatentate fields that are repeated in nested spans, or turn off with ""
///     .concat(Some("::"))
///     // Send output to anything that implements `std::io::Write`
///     .writer(std::io::stderr());
/// ```
#[must_use]
pub fn json() -> Traceon {
    Traceon::default()
}

#[must_use]
pub fn pretty() -> Traceon {
    Traceon::pretty()
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
