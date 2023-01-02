use tracing::{info_span, instrument};

#[instrument]
fn span_one(a: &str, b: &str) {
    tracing::info!("hello!");
    span_two("two_a", "two_b")
}

#[instrument]
fn span_two(a: &str, b: &str) {
    tracing::info!("hello again!");
}

#[tokio::main]
async fn main() {
    traceon::builder().on();
    let _span = info_span!("how about this message?", wow = 50, cool = 200).entered();
    tracing::info!("first");
    span_one("one_a", "one_b");
}
