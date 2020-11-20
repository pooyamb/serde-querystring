use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize, PartialEq)]
struct SampleChild {
    x: i32,
    y: i32,
    z: i32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    x: SampleChild,
    y: SampleChild,
    z: SampleChild,
}

fn deserialize(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let ordered = "x[x]=11111&x[y]=222222&x[z]=33333&\
                   y[x]=11111&y[y]=222222&y[z]=33333&\
                   z[x]=11111&z[y]=222222&z[z]=33333";

    c.bench_function("one level ordered", |b| b.iter(|| deserialize(ordered)));
}

fn unordered(c: &mut Criterion) {
    let unordered = "z[z]=11111&z[y]=222222&z[x]=33333&\
                     y[z]=11111&y[y]=222222&y[x]=33333&\
                     x[z]=11111&x[y]=222222&x[x]=33333";

    c.bench_function("one level unordered", |b| b.iter(|| deserialize(unordered)));
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
