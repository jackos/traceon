use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{
    Resource,
    trace::{SdkTracerProvider, TraceError},
};
use opentelemetry_stdout as stdout;
use std::error::Error;
use tracing::{info, instrument, span};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Initialize the tracer provider
    let provider = init_tracer()?;

    // Get a tracer from the provider
    let tracer = provider.tracer("traceon_async");

    // Create OpenTelemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Compose layers
    let subscriber = Registry::default().with(telemetry).with(traceon::builder());

    // Set as the default subscriber
    tracing::subscriber::set_global_default(subscriber)?;

    // Run async application logic
    run_application().await;

    // Properly shutdown the provider
    // This ensures all pending spans are exported
    provider.shutdown()?;

    Ok(())
}

fn init_tracer() -> Result<SdkTracerProvider, TraceError> {
    let resource = Resource::builder()
        .with_service_name("traceon_async_example")
        .build();

    let exporter = stdout::SpanExporter::default();

    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build())
}

async fn run_application() {
    let root = span!(tracing::Level::INFO, "async_application");
    let _enter = root.enter();

    info!("Starting async application");

    // Run concurrent tasks
    let task1 = process_task("task1", 100);
    let task2 = process_task("task2", 150);
    let task3 = process_task("task3", 75);

    // Wait for all tasks to complete
    let (r1, r2, r3) = tokio::join!(task1, task2, task3);

    info!(
        task1_result = r1,
        task2_result = r2,
        task3_result = r3,
        "All tasks completed"
    );
}

#[instrument(fields(duration_ms))]
async fn process_task(name: &str, duration_ms: u64) -> u64 {
    info!("Task started");

    // Simulate async work
    tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;

    // Record the duration
    tracing::Span::current().record("duration_ms", duration_ms);

    info!("Task completed");
    duration_ms
}
