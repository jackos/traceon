use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_stdout as stdout;
use tracing::{info, span};
use tracing_subscriber::{Registry, layer::SubscriberExt};

fn main() {
    // Build provider with batch processing and resource configuration
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(stdout::SpanExporter::default())
        .build();

    // Create a tracing layer with the configured tracer
    let tracer = provider.tracer("traceon_example");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    // Compose traceon and opentelemetry together
    let subscriber = Registry::default().with(telemetry).with(traceon::builder());

    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
        let _enter = root.enter();

        info!(
            "This will generate an event for the entered span using opentelemetry \
            and also log flattened data via traceon."
        );

        // Nested span example
        {
            let child = span!(tracing::Level::INFO, "processing", items = 5);
            let _enter = child.enter();
            info!("Processing items in nested span");
        }
    });
}
