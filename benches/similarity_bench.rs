use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use elid::*;

fn bench_levenshtein(c: &mut Criterion) {
    let mut group = c.benchmark_group("levenshtein");

    let test_cases = vec![
        ("short", "shirt"),
        ("kitten", "sitting"),
        ("saturday", "sunday"),
        ("exponential", "polynomial"),
    ];

    for (a, b) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_vs_{}", a, b)),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| levenshtein(black_box(a), black_box(b)));
            },
        );
    }

    group.finish();
}

fn bench_jaro_winkler(c: &mut Criterion) {
    let mut group = c.benchmark_group("jaro_winkler");

    let test_cases = vec![
        ("martha", "marhta"),
        ("DIXON", "DICKSON"),
        ("John Smith", "Jon Smith"),
    ];

    for (a, b) in test_cases {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_vs_{}", a, b)),
            &(a, b),
            |bench, (a, b)| {
                bench.iter(|| jaro_winkler(black_box(a), black_box(b)));
            },
        );
    }

    group.finish();
}

fn bench_hamming(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamming");

    let a = "ACGTACGT";
    let b = "ACGTACCT";

    group.bench_function("dna_sequence", |bench| {
        bench.iter(|| hamming(black_box(a), black_box(b)));
    });

    group.finish();
}

fn bench_best_match(c: &mut Criterion) {
    let candidates = vec![
        "apple",
        "application",
        "apply",
        "apricot",
        "banana",
        "bandana",
        "cathedral",
    ];

    c.bench_function("find_best_match", |b| {
        b.iter(|| find_best_match(black_box("app"), black_box(&candidates)));
    });
}

criterion_group!(
    benches,
    bench_levenshtein,
    bench_jaro_winkler,
    bench_hamming,
    bench_best_match
);
criterion_main!(benches);
