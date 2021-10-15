use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::{criterion_group, criterion_main};
use lib::p2p::crypto::Key;

fn bench_key_hash(c: &mut Criterion) {
    const PASSWORDS: [&str; 3] = [
        "the biggest oversight",
        "
The most intelligent E-Class family of all time welcomes a powerful new member to the dynasty. \
The E400 Sedan model arrives this year, boasting a 3.0L V6 biturbo engine producing 329 hp and \
354 lb-ft of torque — the same powertrain that currently drives its E400 Coupe, Cabriolet and \
4MATIC Wagon cousins.",
        "ehulrachaolercuhaoelrcuhaoerlc",
    ];

    let mut group = c.benchmark_group("key_hash");
    for (i, p) in PASSWORDS.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("password", i), &p, |b, p| {
            b.iter(|| Key::from(p).hash())
        });
    }
}

criterion_group!(benches, bench_key_hash);
criterion_main!(benches);
