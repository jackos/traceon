use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_traceon(c: &mut Criterion) {
    let mut group = c.benchmark_group("traceon");

    group.bench_function("traceon", |b| {
        let _guard = traceon::Traceon::new()
            .file(false)
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
        .with_span_list(false)
        .init();
    group.bench_function("tracing_subscriber json", |b| {
        b.iter(|| {
            black_box(tracing::info!("testing out a resonably long string"));
        })
    });
}

criterion_group!(benches, bench_traceon);
criterion_main!(benches);
