use tracing::info;

fn main() {
    traceon::builder().on();
    let vector = vec![10, 15, 20];
    info!(?vector, "Wow this works ok: {vector:#?}");
    // tracing::event!(
    //     Level::WARN,
    //     message = "add message field, and debug a vector",
    //     ?vector,
    // );
}
