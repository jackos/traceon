use traceon::SpanFormat;

fn main() {
    traceon::builder().span(SpanFormat::Join(">")).on();

    let _span = tracing::info_span!("level_1").entered();
    tracing::info!("span level 1");

    let _span = tracing::info_span!("level_2").entered();
    tracing::info!("span level 2");
}
