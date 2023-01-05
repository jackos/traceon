use traceon::{info, instrument};

#[instrument]
async fn add(a: i32, b: i32) {
    info!("result = {}", a + b);
}

#[tokio::main]
async fn main() {
    traceon::builder().on();
    add(5, 10).await;
}
