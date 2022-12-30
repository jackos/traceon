use tracing::Level;

mod math {
    pub fn add() {
        tracing::info!("inside a module");
    }
}

fn main() {
    traceon::pretty().on();

    tracing::info!("single message nothing here");
    let five = 5;
    let _span = tracing::info_span!("cool", five).entered();
    tracing::info!("only this message and level as text");

    tracing::event!(
        Level::INFO,
        event_example = "add field and log it without a span"
    );

    let vector = vec!["cool", "one", "cuz"];
    tracing::event!(
        Level::WARN,
        message = "add message field, and debug a vector",
        vector = format!("{:#?}", vector),
    );

    math::add();
}
