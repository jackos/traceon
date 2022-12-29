use tracing::Instrument;

async fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
    traceon::on();
    let span = tracing::info_span!("math_functions", package_name = env!("CARGO_PKG_NAME"));
    add(5, 10).instrument(span).await;
}
