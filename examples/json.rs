fn main() {
    traceon::builder().json().on();
    tracing::info!("a simple message");
}
