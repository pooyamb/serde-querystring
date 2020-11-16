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

fn serde_querystring(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn serde_qs(input: &str) -> Result<Sample, impl Error> {
    serde_qs::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let ordered = "x0[x1][x2][x3][x4]=1&x0[x1][x2][x3][y4]=2&x0[x1][x2][x3][z4]=3\
    &x0[x1][x2][y3][x4]=1&x0[x1][x2][y3][y4]=2&x0[x1][x2][y3][z4]=3&x0[x1][x2][z3][x4]=1\
    &x0[x1][x2][z3][y4]=2&x0[x1][x2][z3][z4]=3&x0[x1][y2][x3][x4]=1&x0[x1][y2][x3][y4]=2\
    &x0[x1][y2][x3][z4]=3&x0[x1][y2][y3][x4]=1&x0[x1][y2][y3][y4]=2&x0[x1][y2][y3][z4]=3\
    &x0[x1][y2][z3][x4]=1&x0[x1][y2][z3][y4]=2&x0[x1][y2][z3][z4]=3&x0[x1][z2][x3][x4]=1\
    &x0[x1][z2][x3][y4]=2&x0[x1][z2][x3][z4]=3&x0[x1][z2][y3][x4]=1&x0[x1][z2][y3][y4]=2\
    &x0[x1][z2][y3][z4]=3&x0[x1][z2][z3][x4]=1&x0[x1][z2][z3][y4]=2&x0[x1][z2][z3][z4]=3\
    &x0[y1][x2][x3][x4]=1&x0[y1][x2][x3][y4]=2&x0[y1][x2][x3][z4]=3&x0[y1][x2][y3][x4]=1\
    &x0[y1][x2][y3][y4]=2&x0[y1][x2][y3][z4]=3&x0[y1][x2][z3][x4]=1&x0[y1][x2][z3][y4]=2\
    &x0[y1][x2][z3][z4]=3&x0[y1][y2][x3][x4]=1&x0[y1][y2][x3][y4]=2&x0[y1][y2][x3][z4]=3\
    &x0[y1][y2][y3][x4]=1&x0[y1][y2][y3][y4]=2&x0[y1][y2][y3][z4]=3&x0[y1][y2][z3][x4]=1\
    &x0[y1][y2][z3][y4]=2&x0[y1][y2][z3][z4]=3&x0[y1][z2][x3][x4]=1&x0[y1][z2][x3][y4]=2\
    &x0[y1][z2][x3][z4]=3&x0[y1][z2][y3][x4]=1&x0[y1][z2][y3][y4]=2&x0[y1][z2][y3][z4]=3\
    &x0[y1][z2][z3][x4]=1&x0[y1][z2][z3][y4]=2&x0[y1][z2][z3][z4]=3&x0[z1][x2][x3][x4]=1\
    &x0[z1][x2][x3][y4]=2&x0[z1][x2][x3][z4]=3&x0[z1][x2][y3][x4]=1&x0[z1][x2][y3][y4]=2\
    &x0[z1][x2][y3][z4]=3&x0[z1][x2][z3][x4]=1&x0[z1][x2][z3][y4]=2&x0[z1][x2][z3][z4]=3\
    &x0[z1][y2][x3][x4]=1&x0[z1][y2][x3][y4]=2&x0[z1][y2][x3][z4]=3&x0[z1][y2][y3][x4]=1\
    &x0[z1][y2][y3][y4]=2&x0[z1][y2][y3][z4]=3&x0[z1][y2][z3][x4]=1&x0[z1][y2][z3][y4]=2\
    &x0[z1][y2][z3][z4]=3&x0[z1][z2][x3][x4]=1&x0[z1][z2][x3][y4]=2&x0[z1][z2][x3][z4]=3\
    &x0[z1][z2][y3][x4]=1&x0[z1][z2][y3][y4]=2&x0[z1][z2][y3][z4]=3&x0[z1][z2][z3][x4]=1\
    &x0[z1][z2][z3][y4]=2&x0[z1][z2][z3][z4]=3&y0[x1][x2][x3][x4]=1&y0[x1][x2][x3][y4]=2\
    &y0[x1][x2][x3][z4]=3&y0[x1][x2][y3][x4]=1&y0[x1][x2][y3][y4]=2&y0[x1][x2][y3][z4]=3\
    &y0[x1][x2][z3][x4]=1&y0[x1][x2][z3][y4]=2&y0[x1][x2][z3][z4]=3&y0[x1][y2][x3][x4]=1\
    &y0[x1][y2][x3][y4]=2&y0[x1][y2][x3][z4]=3&y0[x1][y2][y3][x4]=1&y0[x1][y2][y3][y4]=2\
    &y0[x1][y2][y3][z4]=3&y0[x1][y2][z3][x4]=1&y0[x1][y2][z3][y4]=2&y0[x1][y2][z3][z4]=3\
    &y0[x1][z2][x3][x4]=1&y0[x1][z2][x3][y4]=2&y0[x1][z2][x3][z4]=3&y0[x1][z2][y3][x4]=1\
    &y0[x1][z2][y3][y4]=2&y0[x1][z2][y3][z4]=3&y0[x1][z2][z3][x4]=1&y0[x1][z2][z3][y4]=2\
    &y0[x1][z2][z3][z4]=3&y0[y1][x2][x3][x4]=1&y0[y1][x2][x3][y4]=2&y0[y1][x2][x3][z4]=3\
    &y0[y1][x2][y3][x4]=1&y0[y1][x2][y3][y4]=2&y0[y1][x2][y3][z4]=3&y0[y1][x2][z3][x4]=1\
    &y0[y1][x2][z3][y4]=2&y0[y1][x2][z3][z4]=3&y0[y1][y2][x3][x4]=1&y0[y1][y2][x3][y4]=2\
    &y0[y1][y2][x3][z4]=3&y0[y1][y2][y3][x4]=1&y0[y1][y2][y3][y4]=2&y0[y1][y2][y3][z4]=3\
    &y0[y1][y2][z3][x4]=1&y0[y1][y2][z3][y4]=2&y0[y1][y2][z3][z4]=3&y0[y1][z2][x3][x4]=1\
    &y0[y1][z2][x3][y4]=2&y0[y1][z2][x3][z4]=3&y0[y1][z2][y3][x4]=1&y0[y1][z2][y3][y4]=2\
    &y0[y1][z2][y3][z4]=3&y0[y1][z2][z3][x4]=1&y0[y1][z2][z3][y4]=2&y0[y1][z2][z3][z4]=3\
    &y0[z1][x2][x3][x4]=1&y0[z1][x2][x3][y4]=2&y0[z1][x2][x3][z4]=3&y0[z1][x2][y3][x4]=1\
    &y0[z1][x2][y3][y4]=2&y0[z1][x2][y3][z4]=3&y0[z1][x2][z3][x4]=1&y0[z1][x2][z3][y4]=2\
    &y0[z1][x2][z3][z4]=3&y0[z1][y2][x3][x4]=1&y0[z1][y2][x3][y4]=2&y0[z1][y2][x3][z4]=3\
    &y0[z1][y2][y3][x4]=1&y0[z1][y2][y3][y4]=2&y0[z1][y2][y3][z4]=3&y0[z1][y2][z3][x4]=1\
    &y0[z1][y2][z3][y4]=2&y0[z1][y2][z3][z4]=3&y0[z1][z2][x3][x4]=1&y0[z1][z2][x3][y4]=2\
    &y0[z1][z2][x3][z4]=3&y0[z1][z2][y3][x4]=1&y0[z1][z2][y3][y4]=2&y0[z1][z2][y3][z4]=3\
    &y0[z1][z2][z3][x4]=1&y0[z1][z2][z3][y4]=2&y0[z1][z2][z3][z4]=3&z0[x1][x2][x3][x4]=1\
    &z0[x1][x2][x3][y4]=2&z0[x1][x2][x3][z4]=3&z0[x1][x2][y3][x4]=1&z0[x1][x2][y3][y4]=2\
    &z0[x1][x2][y3][z4]=3&z0[x1][x2][z3][x4]=1&z0[x1][x2][z3][y4]=2&z0[x1][x2][z3][z4]=3\
    &z0[x1][y2][x3][x4]=1&z0[x1][y2][x3][y4]=2&z0[x1][y2][x3][z4]=3&z0[x1][y2][y3][x4]=1\
    &z0[x1][y2][y3][y4]=2&z0[x1][y2][y3][z4]=3&z0[x1][y2][z3][x4]=1&z0[x1][y2][z3][y4]=2\
    &z0[x1][y2][z3][z4]=3&z0[x1][z2][x3][x4]=1&z0[x1][z2][x3][y4]=2&z0[x1][z2][x3][z4]=3\
    &z0[x1][z2][y3][x4]=1&z0[x1][z2][y3][y4]=2&z0[x1][z2][y3][z4]=3&z0[x1][z2][z3][x4]=1\
    &z0[x1][z2][z3][y4]=2&z0[x1][z2][z3][z4]=3&z0[y1][x2][x3][x4]=1&z0[y1][x2][x3][y4]=2\
    &z0[y1][x2][x3][z4]=3&z0[y1][x2][y3][x4]=1&z0[y1][x2][y3][y4]=2&z0[y1][x2][y3][z4]=3\
    &z0[y1][x2][z3][x4]=1&z0[y1][x2][z3][y4]=2&z0[y1][x2][z3][z4]=3&z0[y1][y2][x3][x4]=1\
    &z0[y1][y2][x3][y4]=2&z0[y1][y2][x3][z4]=3&z0[y1][y2][y3][x4]=1&z0[y1][y2][y3][y4]=2\
    &z0[y1][y2][y3][z4]=3&z0[y1][y2][z3][x4]=1&z0[y1][y2][z3][y4]=2&z0[y1][y2][z3][z4]=3\
    &z0[y1][z2][x3][x4]=1&z0[y1][z2][x3][y4]=2&z0[y1][z2][x3][z4]=3&z0[y1][z2][y3][x4]=1\
    &z0[y1][z2][y3][y4]=2&z0[y1][z2][y3][z4]=3&z0[y1][z2][z3][x4]=1&z0[y1][z2][z3][y4]=2\
    &z0[y1][z2][z3][z4]=3&z0[z1][x2][x3][x4]=1&z0[z1][x2][x3][y4]=2&z0[z1][x2][x3][z4]=3\
    &z0[z1][x2][y3][x4]=1&z0[z1][x2][y3][y4]=2&z0[z1][x2][y3][z4]=3&z0[z1][x2][z3][x4]=1\
    &z0[z1][x2][z3][y4]=2&z0[z1][x2][z3][z4]=3&z0[z1][y2][x3][x4]=1&z0[z1][y2][x3][y4]=2\
    &z0[z1][y2][x3][z4]=3&z0[z1][y2][y3][x4]=1&z0[z1][y2][y3][y4]=2&z0[z1][y2][y3][z4]=3\
    &z0[z1][y2][z3][x4]=1&z0[z1][y2][z3][y4]=2&z0[z1][y2][z3][z4]=3&z0[z1][z2][x3][x4]=1\
    &z0[z1][z2][x3][y4]=2&z0[z1][z2][x3][z4]=3&z0[z1][z2][y3][x4]=1&z0[z1][z2][y3][y4]=2\
    &z0[z1][z2][y3][z4]=3&z0[z1][z2][z3][x4]=1&z0[z1][z2][z3][y4]=2&z0[z1][z2][z3][z4]=3";

    // Check if everything is working as expected
    assert_eq!(
        serde_querystring(ordered).unwrap(),
        serde_qs(ordered).unwrap()
    );

    c.bench_function("many level child ordered querystring", |b| {
        b.iter(|| serde_querystring(ordered))
    });
    c.bench_function("many level child ordered qs", |b| {
        b.iter(|| serde_qs(ordered))
    });
}

fn unordered(c: &mut Criterion) {
    let unordered = "z0[z1][z2][z3][z4]=3&z0[z1][z2][z3][y4]=2&z0[z1][z2][z3][x4]=1\
    &z0[z1][z2][y3][z4]=3&z0[z1][z2][y3][y4]=2&z0[z1][z2][y3][x4]=1&z0[z1][z2][x3][z4]=3\
    &z0[z1][z2][x3][y4]=2&z0[z1][z2][x3][x4]=1&z0[z1][y2][z3][z4]=3&z0[z1][y2][z3][y4]=2\
    &z0[z1][y2][z3][x4]=1&z0[z1][y2][y3][z4]=3&z0[z1][y2][y3][y4]=2&z0[z1][y2][y3][x4]=1\
    &z0[z1][y2][x3][z4]=3&z0[z1][y2][x3][y4]=2&z0[z1][y2][x3][x4]=1&z0[z1][x2][z3][z4]=3\
    &z0[z1][x2][z3][y4]=2&z0[z1][x2][z3][x4]=1&z0[z1][x2][y3][z4]=3&z0[z1][x2][y3][y4]=2\
    &z0[z1][x2][y3][x4]=1&z0[z1][x2][x3][z4]=3&z0[z1][x2][x3][y4]=2&z0[z1][x2][x3][x4]=1\
    &z0[y1][z2][z3][z4]=3&z0[y1][z2][z3][y4]=2&z0[y1][z2][z3][x4]=1&z0[y1][z2][y3][z4]=3\
    &z0[y1][z2][y3][y4]=2&z0[y1][z2][y3][x4]=1&z0[y1][z2][x3][z4]=3&z0[y1][z2][x3][y4]=2\
    &z0[y1][z2][x3][x4]=1&z0[y1][y2][z3][z4]=3&z0[y1][y2][z3][y4]=2&z0[y1][y2][z3][x4]=1\
    &z0[y1][y2][y3][z4]=3&z0[y1][y2][y3][y4]=2&z0[y1][y2][y3][x4]=1&z0[y1][y2][x3][z4]=3\
    &z0[y1][y2][x3][y4]=2&z0[y1][y2][x3][x4]=1&z0[y1][x2][z3][z4]=3&z0[y1][x2][z3][y4]=2\
    &z0[y1][x2][z3][x4]=1&z0[y1][x2][y3][z4]=3&z0[y1][x2][y3][y4]=2&z0[y1][x2][y3][x4]=1\
    &z0[y1][x2][x3][z4]=3&z0[y1][x2][x3][y4]=2&z0[y1][x2][x3][x4]=1&z0[x1][z2][z3][z4]=3\
    &z0[x1][z2][z3][y4]=2&z0[x1][z2][z3][x4]=1&z0[x1][z2][y3][z4]=3&z0[x1][z2][y3][y4]=2\
    &z0[x1][z2][y3][x4]=1&z0[x1][z2][x3][z4]=3&z0[x1][z2][x3][y4]=2&z0[x1][z2][x3][x4]=1\
    &z0[x1][y2][z3][z4]=3&z0[x1][y2][z3][y4]=2&z0[x1][y2][z3][x4]=1&z0[x1][y2][y3][z4]=3\
    &z0[x1][y2][y3][y4]=2&z0[x1][y2][y3][x4]=1&z0[x1][y2][x3][z4]=3&z0[x1][y2][x3][y4]=2\
    &z0[x1][y2][x3][x4]=1&z0[x1][x2][z3][z4]=3&z0[x1][x2][z3][y4]=2&z0[x1][x2][z3][x4]=1\
    &z0[x1][x2][y3][z4]=3&z0[x1][x2][y3][y4]=2&z0[x1][x2][y3][x4]=1&z0[x1][x2][x3][z4]=3\
    &z0[x1][x2][x3][y4]=2&z0[x1][x2][x3][x4]=1&y0[z1][z2][z3][z4]=3&y0[z1][z2][z3][y4]=2\
    &y0[z1][z2][z3][x4]=1&y0[z1][z2][y3][z4]=3&y0[z1][z2][y3][y4]=2&y0[z1][z2][y3][x4]=1\
    &y0[z1][z2][x3][z4]=3&y0[z1][z2][x3][y4]=2&y0[z1][z2][x3][x4]=1&y0[z1][y2][z3][z4]=3\
    &y0[z1][y2][z3][y4]=2&y0[z1][y2][z3][x4]=1&y0[z1][y2][y3][z4]=3&y0[z1][y2][y3][y4]=2\
    &y0[z1][y2][y3][x4]=1&y0[z1][y2][x3][z4]=3&y0[z1][y2][x3][y4]=2&y0[z1][y2][x3][x4]=1\
    &y0[z1][x2][z3][z4]=3&y0[z1][x2][z3][y4]=2&y0[z1][x2][z3][x4]=1&y0[z1][x2][y3][z4]=3\
    &y0[z1][x2][y3][y4]=2&y0[z1][x2][y3][x4]=1&y0[z1][x2][x3][z4]=3&y0[z1][x2][x3][y4]=2\
    &y0[z1][x2][x3][x4]=1&y0[y1][z2][z3][z4]=3&y0[y1][z2][z3][y4]=2&y0[y1][z2][z3][x4]=1\
    &y0[y1][z2][y3][z4]=3&y0[y1][z2][y3][y4]=2&y0[y1][z2][y3][x4]=1&y0[y1][z2][x3][z4]=3\
    &y0[y1][z2][x3][y4]=2&y0[y1][z2][x3][x4]=1&y0[y1][y2][z3][z4]=3&y0[y1][y2][z3][y4]=2\
    &y0[y1][y2][z3][x4]=1&y0[y1][y2][y3][z4]=3&y0[y1][y2][y3][y4]=2&y0[y1][y2][y3][x4]=1\
    &y0[y1][y2][x3][z4]=3&y0[y1][y2][x3][y4]=2&y0[y1][y2][x3][x4]=1&y0[y1][x2][z3][z4]=3\
    &y0[y1][x2][z3][y4]=2&y0[y1][x2][z3][x4]=1&y0[y1][x2][y3][z4]=3&y0[y1][x2][y3][y4]=2\
    &y0[y1][x2][y3][x4]=1&y0[y1][x2][x3][z4]=3&y0[y1][x2][x3][y4]=2&y0[y1][x2][x3][x4]=1\
    &y0[x1][z2][z3][z4]=3&y0[x1][z2][z3][y4]=2&y0[x1][z2][z3][x4]=1&y0[x1][z2][y3][z4]=3\
    &y0[x1][z2][y3][y4]=2&y0[x1][z2][y3][x4]=1&y0[x1][z2][x3][z4]=3&y0[x1][z2][x3][y4]=2\
    &y0[x1][z2][x3][x4]=1&y0[x1][y2][z3][z4]=3&y0[x1][y2][z3][y4]=2&y0[x1][y2][z3][x4]=1\
    &y0[x1][y2][y3][z4]=3&y0[x1][y2][y3][y4]=2&y0[x1][y2][y3][x4]=1&y0[x1][y2][x3][z4]=3\
    &y0[x1][y2][x3][y4]=2&y0[x1][y2][x3][x4]=1&y0[x1][x2][z3][z4]=3&y0[x1][x2][z3][y4]=2\
    &y0[x1][x2][z3][x4]=1&y0[x1][x2][y3][z4]=3&y0[x1][x2][y3][y4]=2&y0[x1][x2][y3][x4]=1\
    &y0[x1][x2][x3][z4]=3&y0[x1][x2][x3][y4]=2&y0[x1][x2][x3][x4]=1&x0[z1][z2][z3][z4]=3\
    &x0[z1][z2][z3][y4]=2&x0[z1][z2][z3][x4]=1&x0[z1][z2][y3][z4]=3&x0[z1][z2][y3][y4]=2\
    &x0[z1][z2][y3][x4]=1&x0[z1][z2][x3][z4]=3&x0[z1][z2][x3][y4]=2&x0[z1][z2][x3][x4]=1\
    &x0[z1][y2][z3][z4]=3&x0[z1][y2][z3][y4]=2&x0[z1][y2][z3][x4]=1&x0[z1][y2][y3][z4]=3\
    &x0[z1][y2][y3][y4]=2&x0[z1][y2][y3][x4]=1&x0[z1][y2][x3][z4]=3&x0[z1][y2][x3][y4]=2\
    &x0[z1][y2][x3][x4]=1&x0[z1][x2][z3][z4]=3&x0[z1][x2][z3][y4]=2&x0[z1][x2][z3][x4]=1\
    &x0[z1][x2][y3][z4]=3&x0[z1][x2][y3][y4]=2&x0[z1][x2][y3][x4]=1&x0[z1][x2][x3][z4]=3\
    &x0[z1][x2][x3][y4]=2&x0[z1][x2][x3][x4]=1&x0[y1][z2][z3][z4]=3&x0[y1][z2][z3][y4]=2\
    &x0[y1][z2][z3][x4]=1&x0[y1][z2][y3][z4]=3&x0[y1][z2][y3][y4]=2&x0[y1][z2][y3][x4]=1\
    &x0[y1][z2][x3][z4]=3&x0[y1][z2][x3][y4]=2&x0[y1][z2][x3][x4]=1&x0[y1][y2][z3][z4]=3\
    &x0[y1][y2][z3][y4]=2&x0[y1][y2][z3][x4]=1&x0[y1][y2][y3][z4]=3&x0[y1][y2][y3][y4]=2\
    &x0[y1][y2][y3][x4]=1&x0[y1][y2][x3][z4]=3&x0[y1][y2][x3][y4]=2&x0[y1][y2][x3][x4]=1\
    &x0[y1][x2][z3][z4]=3&x0[y1][x2][z3][y4]=2&x0[y1][x2][z3][x4]=1&x0[y1][x2][y3][z4]=3\
    &x0[y1][x2][y3][y4]=2&x0[y1][x2][y3][x4]=1&x0[y1][x2][x3][z4]=3&x0[y1][x2][x3][y4]=2\
    &x0[y1][x2][x3][x4]=1&x0[x1][z2][z3][z4]=3&x0[x1][z2][z3][y4]=2&x0[x1][z2][z3][x4]=1\
    &x0[x1][z2][y3][z4]=3&x0[x1][z2][y3][y4]=2&x0[x1][z2][y3][x4]=1&x0[x1][z2][x3][z4]=3\
    &x0[x1][z2][x3][y4]=2&x0[x1][z2][x3][x4]=1&x0[x1][y2][z3][z4]=3&x0[x1][y2][z3][y4]=2\
    &x0[x1][y2][z3][x4]=1&x0[x1][y2][y3][z4]=3&x0[x1][y2][y3][y4]=2&x0[x1][y2][y3][x4]=1\
    &x0[x1][y2][x3][z4]=3&x0[x1][y2][x3][y4]=2&x0[x1][y2][x3][x4]=1&x0[x1][x2][z3][z4]=3\
    &x0[x1][x2][z3][y4]=2&x0[x1][x2][z3][x4]=1&x0[x1][x2][y3][z4]=3&x0[x1][x2][y3][y4]=2\
    &x0[x1][x2][y3][x4]=1&x0[x1][x2][x3][z4]=3&x0[x1][x2][x3][y4]=2&x0[x1][x2][x3][x4]=1";

    // Check if everything is working as expected
    assert_eq!(
        serde_querystring(unordered).unwrap(),
        serde_qs(unordered).unwrap()
    );

    c.bench_function("many level child unordered querystring", |b| {
        b.iter(|| serde_querystring(unordered))
    });
    c.bench_function("many level child unordered qs", |b| {
        b.iter(|| serde_qs(unordered))
    });
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
