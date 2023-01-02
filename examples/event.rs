use tracing::Level;

fn main() {
    traceon::builder().on();

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
