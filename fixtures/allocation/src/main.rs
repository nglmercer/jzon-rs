use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;
#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;
fn measure(label: &str, input: &str, native: bool) {
    let region = Region::new(GLOBAL);
    let start = std::time::Instant::now();
    let values: Vec<String> = if native {
        jzon_serde::from_str(input).unwrap()
    } else {
        jzon::FromJson::from_json_str(input).unwrap()
    };
    let nanos = start.elapsed().as_nanos();
    let stats = region.change();
    let retained: usize = values.iter().map(String::capacity).sum();
    println!(
        "{},{label},{},{},{},{},{},{nanos}",
        if native { "B" } else { "A" },
        input.len(),
        stats.allocations,
        stats.reallocations,
        stats.bytes_allocated,
        retained
    );
    if label.starts_with("short/") {
        assert!(retained <= values.len() * 32);
    }
    std::hint::black_box(values);
}
fn main() {
    println!("mode,case,input_bytes,allocations,reallocations,allocated_bytes,retained_string_capacity,time_ns_single_sample");
    {
        let region = Region::new(GLOBAL);
        let number: u64 = jzon_serde::from_str("42").unwrap();
        let borrowed: &str = jzon_serde::from_str(r#""borrowed""#).unwrap();
        let stats = region.change();
        assert_eq!(
            stats.allocations, 0,
            "scalar and borrowed parsing should allocate nothing"
        );
        assert_eq!(number, 42);
        assert_eq!(borrowed, "borrowed");
        println!(
            "B,scalar_and_borrowed,12,{},0,{},0,0",
            stats.allocations, stats.bytes_allocated
        );
    }
    for count in [100, 1000, 10000] {
        let input = format!("[{}]", vec![r#""\n""#; count].join(","));
        measure(&format!("short/{count}"), &input, true);
        measure(&format!("short/{count}"), &input, false);
    }
    for (label, string) in [
        ("long/plain", "a".repeat(100000)),
        ("long/late_escape", format!("{}\\n", "a".repeat(100000))),
        ("long/dense", r#"\n"#.repeat(50000)),
        ("long/unicode", r#"\u00e9"#.repeat(16666)),
    ] {
        measure(label, &format!("[\"{string}\"]"), true);
        measure(label, &format!("[\"{string}\"]"), false);
    }
    let input = format!("[\"\\n\",\"{}\"]", "a".repeat(100000));
    measure("tiny_then_large", &input, true);
    measure("tiny_then_large", &input, false);
}
