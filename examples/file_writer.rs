fn main() {
    // let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    let file_appender = tracing_appender::rolling::hourly("./", "test.log");
    traceon::json().with_writer(file_appender).on();
    tracing::info!("wow cool!");
}
