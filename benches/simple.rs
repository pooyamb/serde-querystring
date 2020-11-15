use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};

fn serde_querystring() {
    serde_querystring::from_str::<HashMap<String, String>>("x=11111&y=222222&z=33333").unwrap();
}

fn serde_urlencoded() {
    serde_urlencoded::from_str::<HashMap<String, String>>("x=11111&y=222222&z=33333").unwrap();
}

fn serde_qs() {
    serde_qs::from_str::<HashMap<String, String>>("x=11111&y=222222&z=33333").unwrap();
}

fn deserialize_to_string(c: &mut Criterion) {
    c.bench_function("simple string querystring", |b| {
        b.iter(|| serde_querystring())
    });
    c.bench_function("simple string urlencoded", |b| {
        b.iter(|| serde_urlencoded())
    });
    c.bench_function("simple string qs", |b| b.iter(|| serde_qs()));
}

fn serde_querystring_number() {
    serde_querystring::from_str::<HashMap<String, usize>>("x=11111&y=222222&z=33333").unwrap();
}

fn serde_urlencoded_number() {
    serde_urlencoded::from_str::<HashMap<String, usize>>("x=11111&y=222222&z=33333").unwrap();
}

fn serde_qs_number() {
    serde_qs::from_str::<HashMap<String, usize>>("x=11111&y=222222&z=33333").unwrap();
}

fn deserialize_to_number(c: &mut Criterion) {
    c.bench_function("simple number querystring", |b| {
        b.iter(|| serde_querystring_number())
    });
    c.bench_function("simple number urlencoded", |b| {
        b.iter(|| serde_urlencoded_number())
    });
    c.bench_function("simple number qs", |b| b.iter(|| serde_qs_number()));
}

criterion_group!(benches, deserialize_to_string, deserialize_to_number);
criterion_main!(benches);
