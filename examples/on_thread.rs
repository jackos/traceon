fn main() {
    // Turn on the subscriber with json formatting
    let _span = tracing::info_span!(
        "the storage layer in this subscriber will have a field",
        field = "temp"
    );
    let _guard = traceon::builder().json().on_thread();
    tracing::info!("first subscriber");

    // Drop the previous subscriber and storage for the fields, this new one has pretty formatting
    let _span = tracing::info_span!("the storage layer has been reset");
    let _guard = traceon::builder().on_thread();
    tracing::info!("second subscriber")
}
