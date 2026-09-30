#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if data.len() > 4096 {
        return;
    }
    for start in [0, data.len() / 2, data.len(), usize::MAX] {
        assert_eq!(
            jzon::simd::find_escape(data, start),
            jzon::simd::find_escape_scalar(data, start)
        );
        assert_eq!(
            jzon::simd::find(data, start),
            jzon::simd::find_quote_or_backslash(data, start)
        );
        assert_eq!(
            jzon::simd::scan_string_run(data, start),
            jzon::simd::scan_string_run_scalar(data, start)
        );
    }
});
