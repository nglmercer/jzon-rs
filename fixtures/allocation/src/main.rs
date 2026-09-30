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
struct Length(usize);
impl<'de> serde::Deserialize<'de> for Length {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Length;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Length, E> {
                Ok(Length(value.len()))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
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
    {
        use serde::Deserialize;
        let mut parser = jzon_serde::Deserializer::from_str(r#""\n" "\t" "\u00e9""#);
        assert_eq!(Length::deserialize(&mut parser).unwrap().0, 1);
        let region = Region::new(GLOBAL);
        assert_eq!(Length::deserialize(&mut parser).unwrap().0, 1);
        assert_eq!(Length::deserialize(&mut parser).unwrap().0, 2);
        let stats = region.change();
        assert_eq!(
            stats.allocations, 0,
            "warmed transient scratch should allocate nothing"
        );
        println!(
            "B,warmed_transient_scratch,12,{},0,{},0,0",
            stats.allocations, stats.bytes_allocated
        );
        let mut serializer = jzon_serde::Serializer::with_capacity(1024);
        serializer.serialize(&[1, 2, 3]).unwrap();
        let region = Region::new(GLOBAL);
        serializer.clear();
        serializer.serialize(&[4, 5, 6]).unwrap();
        let stats = region.change();
        assert_eq!(
            stats.allocations, 0,
            "warmed reusable serializer should allocate nothing"
        );
        println!(
            "B,warmed_reusable_serializer,7,{},0,{},0,0",
            stats.allocations, stats.bytes_allocated
        );
    }
    {
        let region = Region::new(GLOBAL);
        let empty: Vec<u8> = jzon_serde::from_str("[]").unwrap();
        let stats = region.change();
        assert!(empty.is_empty());
        assert_eq!(
            stats.allocations, 0,
            "bounded native empty containers need no counter allocation"
        );
        println!(
            "B,empty_container,2,{},0,{},0,0",
            stats.allocations, stats.bytes_allocated
        );
    }
    {
        let region = Region::new(GLOBAL);
        jzon_serde::to_writer(std::io::sink(), &[1, 2, 3]).unwrap();
        let stats = region.change();
        assert_eq!(
            stats.allocations, 0,
            "direct native streaming needs no intermediate output buffer"
        );
        println!(
            "B,direct_streaming_writer,7,{},0,{},0,0",
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
