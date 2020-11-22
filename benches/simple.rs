use std::collections::HashMap;
use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};

type Value<T> = HashMap<String, T>;

fn deserialize<T: serde::de::DeserializeOwned>() -> Result<Value<T>, impl Error> {
    serde_querystring::from_str::<Value<T>>("x=11111&y=222222&z=33333")
}

fn deserialize_to_string(c: &mut Criterion) {
    c.bench_function("simple string", |b| b.iter(deserialize::<String>));
}

fn deserialize_to_number(c: &mut Criterion) {
    c.bench_function("simple number", |b| b.iter(deserialize::<isize>));
}

criterion_group!(benches, deserialize_to_string, deserialize_to_number);
criterion_main!(benches);
