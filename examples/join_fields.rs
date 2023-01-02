use traceon::JoinFields;
fn main() {
    traceon::builder()
        .join_fields(JoinFields::Some("||", &["field_b"]))
        .on();

    let _span_1 =
        tracing::info_span!("span_1", field_a = "original", field_b = "original").entered();
    let _span_2 = tracing::info_span!("span_2", field_a = "changed", field_b = "changed").entered();

    tracing::info!("testing field join");
}
