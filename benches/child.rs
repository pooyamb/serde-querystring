use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct SampleChild {
    x: i32,
    y: i32,
    z: i32,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Sample {
    x: SampleChild,
    y: SampleChild,
    z: SampleChild,
}

fn serde_querystring(input: &str) {
    serde_querystring::from_str::<Sample>(input).unwrap();
}

fn serde_qs(input: &str) {
    serde_qs::from_str::<Sample>(input).unwrap();
}

fn ordered(c: &mut Criterion) {
    let ordered = "x[x]=11111&x[y]=222222&x[z]=33333&\
                   y[x]=11111&y[y]=222222&y[z]=33333&\
                   z[x]=11111&z[y]=222222&z[z]=33333";
    c.bench_function("one level child ordered querystring", |b| {
        b.iter(|| serde_querystring(ordered))
    });
    c.bench_function("one level child ordered qs", |b| {
        b.iter(|| serde_qs(ordered))
    });
}

fn unordered(c: &mut Criterion) {
    let unordered = "z[z]=11111&z[y]=222222&z[x]=33333&\
                     y[z]=11111&y[y]=222222&y[x]=33333&\
                     x[z]=11111&x[y]=222222&x[x]=33333";
    c.bench_function("one level child unordered querystring", |b| {
        b.iter(|| serde_querystring(unordered))
    });
    c.bench_function("one level child unordered qs", |b| {
        b.iter(|| serde_qs(unordered))
    });
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
