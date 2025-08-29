use tracing_subscriber::{EnvFilter, prelude::*};

fn main() {
    tracing_subscriber::registry()
        .with(traceon::builder())
        .with(EnvFilter::new("error"))
        .init();

    tracing::info!("info log message won't write to stdout");
    tracing::error!("only error messages will write to stdout");
}
