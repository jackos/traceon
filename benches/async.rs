use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use tracing_subscriber::{prelude::*, EnvFilter};

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
use tokio::runtime::Runtime;

use tracing::{info_span, Instrument};

#[tracing::instrument]
async fn level_1(param: &str) {
    tracing::info!("level_1");
    level_2("level_1", param).await;
}
#[tracing::instrument]
async fn level_2(param: &str, param_2: &str) {
    tracing::info!("level_2");
    level_3("level_2", param, param).await;
}
#[tracing::instrument]
async fn level_3(param: &str, param_2: &str, param_3: &str) {
    tracing::info!("level_3");
}

// Here we have an async function to benchmark
async fn traceon() {
    let _guard = traceon::json()
        .with_filepath(false)
        .with_writer(std::io::sink())
        .with_concat("::")
        .on_thread();

    let span = info_span!("base");
    level_1("base").instrument(span).await;
}

async fn tracing_sub() {
    let _sub = tracing_subscriber::fmt()
        .json()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .flatten_event(true)
        .with_span_list(true)
        .with_writer(std::io::sink)
        .with_env_filter(EnvFilter::new("info"))
        .set_default();
    let span = info_span!("base");
    level_1("base").instrument(span).await;
}

fn async_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("traceon async");

    group.bench_function("traceon", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| traceon());
    });
    group.bench_function("tracing_sub", |b| {
        b.to_async(Runtime::new().unwrap()).iter(|| tracing_sub());
    });
}

criterion_group!(benches, async_bench);
criterion_main!(benches);
