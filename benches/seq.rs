use std::error::Error;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize, PartialEq)]
struct Sample {
    value: Vec<i32>,
}

fn deserialize(input: &str) -> Result<Sample, impl Error> {
    serde_querystring::from_str::<Sample>(input)
}

fn ordered(c: &mut Criterion) {
    let mut ordered = String::new();
    for i in 0..1000 {
        ordered.push_str(&format!("value[{}]={}&", i, 1024 * i));
    }
    ordered.remove(ordered.len() - 1);

    c.bench_function("sequence ordered", |b| b.iter(|| deserialize(&ordered)));
}

fn reverse_ordered(c: &mut Criterion) {
    let mut reverse = String::new();
    for i in 1..=1000 {
        reverse.push_str(&format!("value[{}]={}&", 1000 - i, 1024 * (1000 - i)));
    }
    reverse.remove(reverse.len() - 1);

    c.bench_function("sequence reverse", |b| b.iter(|| deserialize(&reverse)));
}

criterion_group!(benches, ordered, reverse_ordered);
criterion_main!(benches);
