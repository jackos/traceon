use tracing::Level;

mod math {
    #[tracing::instrument]
    pub fn add(a: u32) {
        tracing::info!("inside a module");
    }
}

fn main() {
    // traceon::builder()
    //     .level(Uppercase)
    //     .pretty()
    //     .timestamp()
    //     .concat(Some("::"))
    //     .on();
    // traceon::builder().time(TimeFormat::);
    tracing::info!("a simple message");

    let five = 5;
    let _span = tracing::info_span!("cool", five).entered();
    tracing::info!("only this message and level as text");

    tracing::event!(
        Level::INFO,
        event_example = "add field and log it without a span"
    );

    let vector = vec!["one", "two", "three"];
    tracing::event!(
        Level::WARN,
        message = "add message field, and debug a vector",
        vector = format!("{:#?}", vector),
    );

    math::add(15);
}
