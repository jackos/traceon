use traceon::{Instrument, info_span, instrument};

#[instrument]
async fn span_one(a: &str, b: &str) {
    tracing::info!("hello!");
    span_two("two_a", "two_b").await;
}

#[instrument]
async fn span_two(a: &str, b: &str) {
    tracing::info!("hello again!");
}

#[tokio::main]
async fn main() {
    traceon::builder().on();
    let span = info_span!("base_span", wow = 50, cool = 200);
    tracing::info!("first");
    span_one("one_a", "one_b").instrument(span).await;
}
