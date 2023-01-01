fn main() {
    let file_appender = tracing_appender::rolling::hourly("./", "test.log");
    traceon::builder().writer(file_appender).on();
    tracing::info!("wow cool!");
}
