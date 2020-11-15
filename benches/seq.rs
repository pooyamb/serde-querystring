use criterion::{criterion_group, criterion_main, Criterion};
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Sample {
    x: Vec<i32>,
}

fn serde_querystring(input: &str) {
    serde_querystring::from_str::<Sample>(input).unwrap();
}

fn serde_qs(input: &str) {
    serde_qs::from_str::<Sample>(input).unwrap();
}

fn ordered(c: &mut Criterion) {
    let mut ordered = String::new();

    for i in 0..1000 {
        ordered.push_str(&format!("x[{}]={}&", i, 1024 * i));
    }

    ordered.remove(ordered.len() - 1);

    c.bench_function("sequence ordered querystring", |b| {
        b.iter(|| serde_querystring(&ordered))
    });
    c.bench_function("sequence ordered qs", |b| b.iter(|| serde_qs(&ordered)));
}

fn reverse_ordered(c: &mut Criterion) {
    let mut reverse = String::new();

    for i in 0..1000 {
        reverse.push_str(&format!("x[{}]={}&", 1000 - i, 1024 * (1000 - i)));
    }

    reverse.remove(reverse.len() - 1);

    c.bench_function("sequence reverse querystring", |b| {
        b.iter(|| serde_querystring(&reverse))
    });
    c.bench_function("sequence reverse qs", |b| b.iter(|| serde_qs(&reverse)));
}

criterion_group!(benches, ordered, reverse_ordered);
criterion_main!(benches);
