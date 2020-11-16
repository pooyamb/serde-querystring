use std::collections::HashMap;
use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};

type Value<T> = HashMap<String, T>;

fn serde_querystring<T: serde::de::DeserializeOwned>() -> Result<Value<T>, impl Error> {
    serde_querystring::from_str::<Value<T>>("x=11111&y=222222&z=33333")
}

fn serde_urlencoded<T: serde::de::DeserializeOwned>() -> Result<Value<T>, impl Error> {
    serde_urlencoded::from_str::<Value<T>>("x=11111&y=222222&z=33333")
}

fn serde_qs<T: serde::de::DeserializeOwned>() -> Result<Value<T>, impl Error> {
    serde_qs::from_str::<Value<T>>("x=11111&y=222222&z=33333")
}

fn deserialize_to_string(c: &mut Criterion) {
    // Check if everything is working as expected
    assert_eq!(
        serde_querystring::<String>().unwrap(),
        serde_urlencoded::<String>().unwrap()
    );
    assert_eq!(
        serde_querystring::<String>().unwrap(),
        serde_qs::<String>().unwrap()
    );

    c.bench_function("simple string querystring", |b| {
        b.iter(|| serde_querystring::<String>())
    });
    c.bench_function("simple string urlencoded", |b| {
        b.iter(|| serde_urlencoded::<String>())
    });
    c.bench_function("simple string qs", |b| b.iter(|| serde_qs::<String>()));
}

fn deserialize_to_number(c: &mut Criterion) {
    // Check if everything is working as expected
    assert_eq!(
        serde_querystring::<isize>().unwrap(),
        serde_urlencoded::<isize>().unwrap()
    );
    assert_eq!(
        serde_querystring::<isize>().unwrap(),
        serde_qs::<isize>().unwrap()
    );

    c.bench_function("simple number querystring", |b| {
        b.iter(|| serde_querystring::<isize>())
    });
    c.bench_function("simple number urlencoded", |b| {
        b.iter(|| serde_urlencoded::<isize>())
    });
    c.bench_function("simple number qs", |b| b.iter(|| serde_qs::<isize>()));
}

criterion_group!(benches, deserialize_to_string, deserialize_to_number);
criterion_main!(benches);
