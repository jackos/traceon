use tracing::{info, instrument};

#[instrument]
fn add(a: i32, b: i32) {
    info!("result: {}", a + b);
}

fn main() {
    traceon::builder().on();
    add(5, 10);
}
