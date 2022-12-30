fn main() {
    let file_appender = tracing_appender::rolling::hourly("./", "test.log");
    traceon::json().writer(file_appender).on();
    tracing::info!("wow cool!");
}
