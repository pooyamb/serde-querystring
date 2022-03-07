use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

use serde_querystring::Error;

#[derive(Debug, Deserialize, PartialEq)]
struct Value<T> {
    value: T,
}

#[derive(Debug, Deserialize, PartialEq)]
struct PercentDecoded {
    #[serde(rename = "amoo amoo")]
    amoo: String,
    #[serde(rename = "baba baba")]
    baba: String,
}

fn deserialize<'de, T>(input: &'de str) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    serde_querystring::from_str::<T>(input)
}

fn sequence(c: &mut Criterion) {
    let mut query_string = String::new();
    for _ in 0..32 {
        query_string.push_str(&format!("value=1&"));
    }

    c.bench_function("sequence", |b| {
        b.iter(|| deserialize::<Value<[bool; 32]>>(&query_string))
    });
}

fn pairs(c: &mut Criterion) {
    let mut query_string = String::new();

    for i in 1..256 {
        query_string = format!("{}&foo{}=bar", query_string, i)
    }

    c.bench_function("pairs", |b| {
        b.iter(|| deserialize::<HashMap<&str, &str>>(&query_string))
    });
}

fn decoded(c: &mut Criterion) {
    let query_string = "baba+baba=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF&\
    amoo+amoo=%D8%B9%D9%85%D9%88%20%D9%86%D9%88%D8%B1%D9%88%D8%B2";

    c.bench_function("percent_decoded", |b| {
        b.iter(|| deserialize::<PercentDecoded>(query_string))
    });
}

fn integers(c: &mut Criterion) {
    // Comparing to sequence bench, we can measure how much overhead the integer parsing adds on top
    let mut query_string = String::new();
    for i in 0..32 {
        query_string.push_str(&format!("value={}&", 1024 * i));
    }

    c.bench_function("integers", |b| {
        b.iter(|| deserialize::<Value<[isize; 32]>>(&query_string))
    });
}

fn floats(c: &mut Criterion) {
    // Comparing to sequence bench, we can measure how much overhead the float parsing adds on top
    let mut query_string = String::new();
    for i in 0..32 {
        query_string.push_str(&format!("value={}&", 1024.5 * i as f64));
    }

    c.bench_function("floats", |b| {
        b.iter(|| deserialize::<Value<[f64; 32]>>(&query_string))
    });
}

criterion_group!(benches, pairs, sequence, decoded, integers, floats);
criterion_main!(benches);
