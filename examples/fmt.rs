//! This example is not related to traceon, it's just an example using tracing_subscribers
//! default builder that is an alternate to traceon
use tracing::Level;

fn main() {
    tracing_subscriber::fmt().pretty().init();

    let five = 5;
    let _span = tracing::info_span!("cool", five).entered();
    tracing::info!("only this message and level as text");

    tracing::event!(
        Level::INFO,
        event_example = "add field and log it without a span"
    );

    let vector = vec!["cool", "one", "cuz"];
    tracing::event!(
        Level::WARN,
        message = "add message field, and debug a vector",
        ?vector,
    );
}
