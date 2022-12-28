use tracing::Instrument;

async fn add(a: i32, b: i32) {
    // Important! Don't put any `.await` calls in between `entered()` and `exit()`
    let _span = tracing::info_span!("add", a, b).entered();
    tracing::info!("result: {}", a + b);
}

#[tokio::main]
async fn main() {
	traceon::builder().file(true).on();

	tracing::info!("wow that's cool");
    let span = tracing::info_span!("math functions", package_name = env!("CARGO_PKG_NAME"));
    add(5, 10).instrument(span).await;
}
