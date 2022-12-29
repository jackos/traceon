use traceon::LevelFormat;
use tracing::Level;

fn main() {
    traceon::builder()
        .timestamp(false)
        .module(false)
        .span(false)
        .file(false)
        .level(LevelFormat::Lowercase)
        .on();

    tracing::info!("only this message and level as text");

    tracing::event!(
        Level::INFO,
        event_example = "add field and log it without a span"
    );

    let vector = vec![10, 15, 20];
    tracing::event!(
        Level::WARN,
        message = "add message field, and debug a vector",
        ?vector,
    );
}
