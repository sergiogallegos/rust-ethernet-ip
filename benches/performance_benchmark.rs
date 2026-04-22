use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_ethernet_ip::PlcValue;

fn bench_single_read(c: &mut Criterion) {
    c.bench_function("single_tag_read", |b| {
        b.iter(|| {
            // Mock implementation for benchmarking
            // In real tests, connect to actual PLC
            black_box(PlcValue::Dint(42))
        })
    });
}

fn bench_batch_read(c: &mut Criterion) {
    let tag_counts = vec![5, 10, 25, 50, 100];

    let mut group = c.benchmark_group("batch_read");
    for count in tag_counts {
        group.bench_with_input(BenchmarkId::new("tags", count), &count, |b, &count| {
            b.iter(|| {
                // Mock batch read implementation
                let mut results = Vec::new();
                for i in 0..count {
                    results.push(PlcValue::Dint(i));
                }
                black_box(results)
            })
        });
    }
    group.finish();
}

fn bench_connection_pool(c: &mut Criterion) {
    c.bench_function("connection_pool", |b| {
        b.iter(|| {
            // Mock connection pool operations
            black_box(())
        })
    });
}

criterion_group!(
    benches,
    bench_single_read,
    bench_batch_read,
    bench_connection_pool
);
criterion_main!(benches);
