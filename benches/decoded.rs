use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    amoo: String,
    baba: String,
}

fn deserialize(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let ordered = "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
    amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2";

    c.bench_function("decode ordered", |b| b.iter(|| deserialize(ordered)));
}

fn unordered(c: &mut Criterion) {
    let unordered = "amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2&\
    baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF";

    c.bench_function("decode unordered", |b| b.iter(|| deserialize(unordered)));
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
