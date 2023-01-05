use traceon::{info, instrument};

#[instrument]
fn add(a: i32, b: i32) {
    info!("result: {}", a + b);
}

fn main() {
    traceon::builder().on();
    let _guard = tracing::info_span!("math", package_name = env!("CARGO_PKG_NAME")).entered();
    add(5, 10);
}
