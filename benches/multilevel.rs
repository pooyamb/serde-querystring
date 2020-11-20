use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Level4 {
    x4: String,
    y4: String,
    z4: String,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Level3 {
    x3: Level4,
    y3: Level4,
    z3: Level4,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Level2 {
    x2: Level3,
    y2: Level3,
    z2: Level3,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Level1 {
    x1: Level2,
    y1: Level2,
    z1: Level2,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    x0: Level1,
    y0: Level1,
    z0: Level1,
}

fn deserialize(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let ordered = include_str!("../tests/multilevel_ordered.txt");

    c.bench_function("multilevel ordered", |b| b.iter(|| deserialize(ordered)));
}

fn unordered(c: &mut Criterion) {
    let unordered = include_str!("../tests/multilevel_unordered.txt");

    c.bench_function("multilevel unordered", |b| {
        b.iter(|| deserialize(unordered))
    });
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
