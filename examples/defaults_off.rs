pub fn main() {
    traceon::json()
        .with_default_fields(false)
        .with_span_name(true)
        .with_concat(None)
        .on();
    let _span = tracing::info_span!("level_1").entered();
    tracing::info!("span field is on");

    let _span = tracing::info_span!("level_2").entered();
    tracing::info!("span field is on");
}
