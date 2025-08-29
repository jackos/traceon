use criterion::black_box;
use criterion::{Criterion, criterion_group, criterion_main};
use tracing_subscriber::EnvFilter;

fn bench_traceon(c: &mut Criterion) {
    let mut group = c.benchmark_group("traceon");

    group.bench_function("traceon", |b| {
        let _guard = traceon::builder()
            .json()
            .writer(std::io::sink())
            .on_thread();
        b.iter(|| {
            black_box(tracing::info!("testing out a resonably long string"));
        })
    });

    tracing_subscriber::fmt()
        .json()
        .with_file(false)
        .with_line_number(false)
        .with_target(false)
        .flatten_event(true)
        .with_span_list(true)
        .with_writer(std::io::sink)
        .with_env_filter(EnvFilter::new("info"))
        .init();
    group.bench_function("tracing_subscriber json", |b| {
        b.iter(|| {
            black_box(tracing::info!("testing out a resonably long string"));
        })
    });
}

criterion_group!(benches, bench_traceon);
criterion_main!(benches);
