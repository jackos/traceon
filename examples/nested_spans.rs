use traceon::SpanFormat;

#[tracing::instrument]
fn add(a: i32, b: i32) {
    tracing::info!("result: {}", a + b);
}

fn main() {
    // traceon::builder().on();
    traceon::builder().span(SpanFormat::Overwrite).on();
    let _guard = tracing::info_span!("math", package_name = env!("CARGO_PKG_NAME")).entered();
    add(5, 10);
}
