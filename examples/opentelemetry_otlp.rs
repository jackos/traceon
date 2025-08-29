use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use std::time::Duration;
use tracing::{error, info, span, warn};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;

fn init_tracer() -> SdkTracerProvider {
    // Configure resource with service metadata
    let resource = Resource::builder()
        .with_service_name("traceon_otlp_example")
        .build();

    // Configure OTLP exporter
    // Default endpoint is http://localhost:4318/v1/traces for HTTP
    // or http://localhost:4317 for gRPC
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint("http://localhost:4318")
        .with_timeout(Duration::from_secs(3))
        .build()
        .expect("Failed to create OTLP trace exporter");

    // Build the tracer provider with batch exporter
    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build()
}

fn main() {
    // Initialize the tracer provider
    let provider = init_tracer();

    // Get a tracer from the provider
    let tracer = provider.tracer("traceon_otlp");

    // Create OpenTelemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Compose layers: OpenTelemetry + traceon for local logging
    let subscriber = Registry::default().with(telemetry).with(traceon::builder());

    // Set as the default subscriber
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");

    // Application logic with tracing
    {
        let root = span!(
            tracing::Level::INFO,
            "application",
            version = env!("CARGO_PKG_VERSION")
        );
        let _enter = root.enter();

        info!("Application started");

        // Simulate some work
        process_request();

        // Simulate an error condition
        simulate_error();

        info!("Application completed");
    }

    // Note: In production, you would keep a reference to the provider
    // and call provider.force_flush() before shutdown
    println!("Example complete");
}

fn process_request() {
    let span = span!(tracing::Level::INFO, "process_request", request_id = 42);
    let _enter = span.enter();

    info!("Processing request");

    // Nested operation
    {
        let db_span = span!(tracing::Level::DEBUG, "database_query", table = "users");
        let _enter = db_span.enter();

        info!("Executing database query");
        std::thread::sleep(Duration::from_millis(50));
        info!(rows_returned = 5, "Query completed");
    }

    warn!("Deprecated API endpoint used");
    info!("Request processed successfully");
}

fn simulate_error() {
    let span = span!(tracing::Level::ERROR, "error_handler");
    let _enter = span.enter();

    error!(
        error_code = "DB_CONNECTION_FAILED",
        details = "Connection timeout after 30s",
        "Database connection error occurred"
    );
}
