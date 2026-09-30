//! Isolated allocation counts, separate from Criterion timing and parser-token lengths.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
struct Counting;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ACTIVE.load(Ordering::Relaxed) {
            CALLS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if ACTIVE.load(Ordering::Relaxed) {
            CALLS.fetch_add(1, Ordering::Relaxed);
            BYTES.fetch_add(size, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;
fn measure<T>(mode: &str, n: usize, input: usize, f: impl FnOnce() -> T) {
    CALLS.store(0, Ordering::Relaxed);
    BYTES.store(0, Ordering::Relaxed);
    ACTIVE.store(true, Ordering::Relaxed);
    let result = f();
    ACTIVE.store(false, Ordering::Relaxed);
    let calls = CALLS.load(Ordering::Relaxed);
    let bytes = BYTES.load(Ordering::Relaxed);
    std::hint::black_box(&result);
    println!("{mode},{n},{input},{calls},{bytes}");
}
fn main() {
    println!("mode,elements,input_bytes,allocation_calls,total_requested_bytes");
    for n in [100, 1000, 10000] {
        let values: Vec<u32> = (0..n).collect();
        let input = serde_json::to_vec(&values).unwrap();
        measure("native_incremental", n as usize, input.len(), || {
            jzon::serde_impl::from_reader::<_, Vec<u32>>(input.as_slice()).unwrap()
        });
        measure("native_buffered", n as usize, input.len(), || {
            jzon::serde_impl::from_reader_buffered::<_, Vec<u32>>(input.as_slice()).unwrap()
        });
        measure("patched_reference", n as usize, input.len(), || {
            serde_json::from_reader::<_, Vec<u32>>(input.as_slice()).unwrap()
        });
        measure("native_ignored", n as usize, input.len(), || {
            jzon::serde_impl::from_reader::<_, serde::de::IgnoredAny>(input.as_slice()).unwrap()
        });
    }
}
