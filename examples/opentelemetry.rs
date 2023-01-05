//! Shows how to compose traceon with an opentelemetry layer, so you still get pretty print in the console
//! but also record span information
use opentelemetry::global;
use traceon::{error, info};
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("traceon_jaeger")
        .install_simple()
        .unwrap();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(EnvFilter::new("error"))
        .with(traceon::builder())
        .with(opentelemetry)
        .init();

    info!("info messages will be filtered out");
    error!("only error messages will write to stdout");
    global::shutdown_tracer_provider();
}
