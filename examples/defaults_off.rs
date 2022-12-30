pub fn main() {
    traceon::json()
        .default_fields(false)
        .span(true)
        .concat(None)
        .on();
    let _span = tracing::info_span!("level_1").entered();
    tracing::info!("span field is on");

    let _span = tracing::info_span!("level_2").entered();
    tracing::info!("span field is on");
}
