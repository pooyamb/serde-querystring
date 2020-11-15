use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Sample {
    amoo: String,
    baba: String,
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
    let ordered = "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
    amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2";
    c.bench_function("decode ordered querystring", |b| {
        b.iter(|| serde_querystring(ordered))
    });
    c.bench_function("decode ordered urlencoded", |b| {
        b.iter(|| serde_urlencoded(ordered))
    });
    c.bench_function("decode ordered qs", |b| b.iter(|| serde_qs(ordered)));
}

fn unordered(c: &mut Criterion) {
    let unordered = "amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2&\
    baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF";
    c.bench_function("decode unordered querystring", |b| {
        b.iter(|| serde_querystring(unordered))
    });
    c.bench_function("decode unordered urlencoded", |b| {
        b.iter(|| serde_urlencoded(unordered))
    });
    c.bench_function("decode unordered qs", |b| b.iter(|| serde_qs(unordered)));
}

criterion_group!(benches, ordered, unordered);
criterion_main!(benches);
