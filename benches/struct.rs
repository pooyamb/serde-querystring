use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Sample {
    x: String,
    y: i64,
    z: u64,
}

fn serde_querystring(input: &str) {
    serde_querystring::from_str::<Sample>(input).unwrap();
}

fn serde_urlencoded(input: &str) {
    serde_urlencoded::from_str::<Sample>(input).unwrap();
}

fn serde_qs(input: &str) {
    serde_qs::from_str::<Sample>(input).unwrap();
}

fn ordered(c: &mut Criterion) {
    let ordered = "x=11111&y=222222&z=33333";
    c.bench_function("struct ordered querystring", |b| {
        b.iter(|| serde_querystring(ordered))
    });
    c.bench_function("struct ordered urlencoded", |b| {
        b.iter(|| serde_urlencoded(ordered))
    });
    c.bench_function("struct ordered qs", |b| b.iter(|| serde_qs(ordered)));
}

fn unordered(c: &mut Criterion) {
    let unordered = "z=11111&y=222222&x=33333";
    c.bench_function("struct unordered querystring", |b| {
        b.iter(|| serde_querystring(unordered))
    });
    c.bench_function("struct unordered urlencoded", |b| {
        b.iter(|| serde_urlencoded(unordered))
    });
    c.bench_function("struct unordered qs", |b| b.iter(|| serde_qs(unordered)));
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
