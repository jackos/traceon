fn main() {
    use traceon::{info, info_span};

    let _guard = traceon::on_thread();
    let _span = info_span!("span_with_field", field = "temp", "cool").entered();
    info!("first subscriber");

    let _guard = traceon::json_thread();
    let _span = info_span!("span_with_no_field").entered();
    info!("second subscriber")
}
