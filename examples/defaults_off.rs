pub fn main() {
    traceon::builder().span().concat(None).on();
    let _span = tracing::info_span!("level_1").entered();
    tracing::info!("span field is on");

    let _span = tracing::info_span!("level_2").entered();
    tracing::info!("span field is on");
}
