use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    amoo: String,
    baba: String,
}

fn serde_querystring(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn serde_urlencoded(input: &str) -> Result<Sample, impl Error> {
    serde_urlencoded::from_str::<Sample>(input)
}

fn serde_qs(input: &str) -> Result<Sample, impl Error> {
    serde_qs::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let ordered = "baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
    amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2";

    // Check if everything is working as expected
    assert_eq!(
        serde_querystring(ordered).unwrap(),
        serde_urlencoded(ordered).unwrap()
    );
    assert_eq!(
        serde_querystring(ordered).unwrap(),
        serde_qs(ordered).unwrap()
    );

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

    // Check if everything is working as expected
    assert_eq!(
        serde_querystring(unordered).unwrap(),
        serde_urlencoded(unordered).unwrap()
    );
    assert_eq!(
        serde_querystring(unordered).unwrap(),
        serde_qs(unordered).unwrap()
    );

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
