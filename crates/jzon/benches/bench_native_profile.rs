//! Component timings for corrected Mode B, with explicit allocation contracts.
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use jzon_serde as native;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Wide {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
    g: u64,
    h: u64,
    i: u64,
    j: u64,
    k: u64,
    l: u64,
    m: u64,
    n: u64,
    o: u64,
    p: u64,
}
fn profile(c: &mut Criterion) {
    let mut parse = c.benchmark_group("mode_b/component_parse");
    for (name, input) in [
        ("integer", "18446744073709551615"),
        ("float", "1.2345678901234567e-12"),
    ] {
        parse.throughput(Throughput::Bytes(input.len() as u64));
        assert_eq!(
            native::from_str::<f64>(input).unwrap().to_bits(),
            serde_json::from_str::<f64>(input).unwrap().to_bits()
        );
        parse.bench_function(name, |b| {
            b.iter(|| native::from_str::<f64>(black_box(input)).unwrap())
        });
    }
    for (name, input) in [
        ("string/plain", r#""abcdefghijklmnop""#),
        ("string/late_escape", r#""abcdefghijklmno\n""#),
        ("string/dense", r#""\n\t\r\n\t\r\n\t\r""#),
        ("string/unicode", r#""\u00e9\u00e9\uD83D\uDE00""#),
    ] {
        parse.throughput(Throughput::Bytes(input.len() as u64));
        assert_eq!(
            native::from_str::<String>(input).unwrap(),
            serde_json::from_str::<String>(input).unwrap()
        );
        parse.bench_function(name, |b| {
            b.iter(|| native::from_str::<String>(black_box(input)).unwrap())
        });
    }
    let plain = r#""abcdefghijklmnop""#;
    parse.throughput(Throughput::Bytes(plain.len() as u64));
    parse.bench_function("string/borrowed", |b| {
        b.iter(|| native::from_str::<&str>(black_box(plain)).unwrap())
    });
    let wide = Wide {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: 5,
        f: 6,
        g: 7,
        h: 8,
        i: 9,
        j: 10,
        k: 11,
        l: 12,
        m: 13,
        n: 14,
        o: 15,
        p: 16,
    };
    let input = serde_json::to_string(&wide).unwrap();
    assert_eq!(native::from_str::<Wide>(&input).unwrap(), wide);
    parse.throughput(Throughput::Bytes(input.len() as u64));
    parse.bench_function("wide/field_dispatch", |b| {
        b.iter(|| native::from_str::<Wide>(black_box(&input)).unwrap())
    });
    parse.finish();
    let mut serialize = c.benchmark_group("mode_b/component_serialize");
    let integers: Vec<u64> = (0..128).map(|n| n * 1000000007).collect();
    let floats: Vec<f64> = (1..128).map(|n| n as f64 / 7.0).collect();
    let integer_bytes = native::to_bytes(&integers).unwrap();
    let float_bytes = native::to_bytes(&floats).unwrap();
    assert_eq!(integer_bytes, serde_json::to_vec(&integers).unwrap());
    assert_eq!(float_bytes, serde_json::to_vec(&floats).unwrap());
    serialize.throughput(Throughput::Bytes(integer_bytes.len() as u64));
    serialize.bench_function("integers/fresh", |b| {
        b.iter(|| native::to_bytes(black_box(&integers)).unwrap())
    });
    serialize.throughput(Throughput::Bytes(float_bytes.len() as u64));
    serialize.bench_function("floats/fresh", |b| {
        b.iter(|| native::to_bytes(black_box(&floats)).unwrap())
    });
    for (name, value) in [
        ("string/plain", "abcdefghijklmnop"),
        ("string/escaped", "line\n\t\"end"),
    ] {
        let bytes = native::to_bytes(value).unwrap();
        assert_eq!(bytes, serde_json::to_vec(value).unwrap());
        serialize.throughput(Throughput::Bytes(bytes.len() as u64));
        serialize.bench_function(name, |b| {
            b.iter(|| native::to_bytes(black_box(value)).unwrap())
        });
    }
    let bytes = native::to_bytes(&wide).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&wide).unwrap());
    serialize.throughput(Throughput::Bytes(bytes.len() as u64));
    serialize.bench_function("wide/fresh", |b| {
        b.iter(|| native::to_bytes(black_box(&wide)).unwrap())
    });
    let mut buffer = Vec::with_capacity(bytes.len());
    serialize.bench_function("wide/buffered_writer", |b| {
        b.iter(|| {
            buffer.clear();
            native::to_writer(&mut buffer, black_box(&wide)).unwrap();
            black_box(&buffer);
        })
    });
    serialize.bench_function("wide/direct_reuse", |b| {
        b.iter(|| {
            buffer.clear();
            native::to_bytes_in(black_box(&wide), &mut buffer).unwrap();
            black_box(&buffer);
        })
    });
    serialize.finish();
}
criterion_group!(benches, profile);
criterion_main!(benches);
