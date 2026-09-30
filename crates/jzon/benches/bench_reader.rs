use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
fn readers(c: &mut Criterion) {
    let values: Vec<u32> = (0..10_000).collect();
    let input = serde_json::to_vec(&values).unwrap();
    assert_eq!(
        jzon::serde_impl::from_reader::<_, Vec<u32>>(input.as_slice()).unwrap(),
        values
    );
    assert_eq!(
        serde_json::from_reader::<_, Vec<u32>>(input.as_slice()).unwrap(),
        values
    );
    let mut group = c.benchmark_group("reader_full_u32_array");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function("native_incremental", |b| {
        b.iter(|| {
            jzon::serde_impl::from_reader::<_, Vec<u32>>(black_box(input.as_slice())).unwrap()
        })
    });
    group.bench_function("native_buffered", |b| {
        b.iter(|| {
            jzon::serde_impl::from_reader_buffered::<_, Vec<u32>>(black_box(input.as_slice()))
                .unwrap()
        })
    });
    group.bench_function("patched_reference", |b| {
        b.iter(|| serde_json::from_reader::<_, Vec<u32>>(black_box(input.as_slice())).unwrap())
    });
    group.finish();
}
criterion_group!(benches, readers);
criterion_main!(benches);
