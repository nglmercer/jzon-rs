use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use jzon_rs_compat as facade;
use serde_json as reference;
fn bench(c: &mut Criterion) {
    let input = include_str!("../../jzon/data/twitter.json");
    let value: facade::Value = facade::from_str(input).unwrap();
    assert_eq!(
        value,
        reference::from_str::<reference::Value>(input).unwrap()
    );
    let mut parse = c.benchmark_group("mode_c/full_value/immutable_input");
    parse.throughput(Throughput::Bytes(input.len() as u64));
    parse.bench_function("facade", |b| {
        b.iter(|| facade::from_str::<facade::Value>(black_box(input)).unwrap())
    });
    parse.bench_function("reference", |b| {
        b.iter(|| reference::from_str::<reference::Value>(black_box(input)).unwrap())
    });
    parse.finish();
    let output = facade::to_vec(&value).unwrap();
    assert_eq!(output, reference::to_vec(&value).unwrap());
    eprintln!("mode_c output bytes: {}", output.len());
    let mut serialize = c.benchmark_group("mode_c/full_value/serialize");
    serialize.throughput(Throughput::Bytes(output.len() as u64));
    serialize.bench_function("facade/fresh", |b| {
        b.iter(|| facade::to_vec(black_box(&value)).unwrap())
    });
    serialize.bench_function("reference/fresh", |b| {
        b.iter(|| reference::to_vec(black_box(&value)).unwrap())
    });
    let mut buffer = Vec::with_capacity(output.len());
    serialize.bench_function("facade/reuse", |b| {
        b.iter(|| {
            buffer.clear();
            facade::to_writer(&mut buffer, black_box(&value)).unwrap();
            black_box(&buffer);
        })
    });
    serialize.bench_function("reference/reuse", |b| {
        b.iter(|| {
            buffer.clear();
            reference::to_writer(&mut buffer, black_box(&value)).unwrap();
            black_box(&buffer);
        })
    });
    serialize.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
