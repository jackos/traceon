#[tracing::instrument]
async fn add(a: u32, b: u32) {
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .flatten_event(true)
        .init();
    tracing::info!("wow cool one");
    add(5, 10).await;
}
