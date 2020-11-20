use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    x: String,
    y: i64,
    z: u64,
}

fn deserialize(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let ordered = "x=11111&y=222222&z=33333";

    c.bench_function("struct ordered", |b| b.iter(|| deserialize(ordered)));
}

fn unordered(c: &mut Criterion) {
    let unordered = "z=11111&y=222222&x=33333";

    c.bench_function("struct unordered", |b| b.iter(|| deserialize(unordered)));
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
