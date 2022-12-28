use traceon::LevelFormat;

mod helpers {
    pub fn trace() {
        tracing::info!("in helpers module");
    }
}

fn main() {
    traceon::builder()
        .module(true)
        .span(false)
        .file(false)
        .time(false)
        .level(LevelFormat::Off)
        .on();

    tracing::info!("only the module and message");
    helpers::trace();
}
