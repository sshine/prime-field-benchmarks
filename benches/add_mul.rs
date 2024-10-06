use criterion::*;
use itertools::Itertools;
use prime_field;

fn add_benchmark(c: &mut Criterion) {
    let n_samples = 1_000;
    let n_operations = 1_000;

    let mut add_group = c.benchmark_group("add");
    add_group.sample_size(n_samples);

    let operands = prime_field::random_elements(n_operations + 1);

    let id = BenchmarkId::new("baseline", n_operations);
    add_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                let _sum = x + y;
            }
        });
    });

    let id = BenchmarkId::new("mod", n_operations);
    add_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                prime_field::add_modulo(x, y);
            }
        });
    });

    let id = BenchmarkId::new("fast", n_operations);
    add_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                prime_field::add_with_sub_u128(x, y);
            }
        });
    });

    let id = BenchmarkId::new("winterfell", n_operations);
    add_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                prime_field::add_winterfell(x, y);
            }
        });
    });
}

fn mul_benchmark(c: &mut Criterion) {
    let n_samples = 1_000;
    let n_operations = 1_000;

    let mut mul_group = c.benchmark_group("mul");
    mul_group.sample_size(n_samples);

    let operands = prime_field::random_elements(n_operations + 1);

    let id = BenchmarkId::new("baseline", n_operations);
    mul_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                let _sum = x * y;
            }
        });
    });

    let id = BenchmarkId::new("mod", n_operations);
    mul_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                prime_field::mul_modulo(x, y);
            }
        });
    });

    let id = BenchmarkId::new("reduce159", n_operations);
    mul_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                prime_field::mul_reduce159(x, y);
            }
        });
    });

    let id = BenchmarkId::new("reduce_montgomery", n_operations);
    mul_group.bench_function(id, |bencher| {
        bencher.iter(|| {
            for (&x, &y) in operands.iter().tuple_windows() {
                prime_field::mul_reduce_montgomery(x, y);
            }
        });
    });
}

criterion_group!(add_mul, add_benchmark, mul_benchmark);
criterion_main!(add_mul);
