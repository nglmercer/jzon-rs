use jzon::simd;
#[test]
fn kernels_match_at_alignment_boundaries_and_invalid_offsets() {
    for length in 0..130 {
        let mut bytes = vec![b'a'; length];
        for marker in [b'"', b'\\', 0x1f, 0x80, 0xff] {
            for position in 0..length {
                bytes[position] = marker;
                for start in [0, position, position + 1, length, usize::MAX] {
                    assert_eq!(
                        simd::find_escape(&bytes, start),
                        simd::find_escape_scalar(&bytes, start)
                    );
                    assert_eq!(
                        simd::find(&bytes, start),
                        simd::find_quote_or_backslash(&bytes, start)
                    );
                    assert_eq!(
                        simd::scan_string_run(&bytes, start),
                        simd::scan_string_run_scalar(&bytes, start)
                    );
                    #[cfg(feature = "simd")]
                    {
                        assert_eq!(
                            simd::find_quote_or_backslash_simd16(&bytes, start),
                            simd::find_quote_or_backslash(&bytes, start)
                        );
                        assert_eq!(
                            simd::find_escape_simd16(&bytes, start),
                            simd::find_escape_scalar(&bytes, start)
                        );
                    }
                    #[cfg(all(feature = "simd-intrinsics", target_arch = "x86_64"))]
                    {
                        use jzon::simd_arch::x86;
                        assert_eq!(
                            x86::find_quote_or_backslash_16(&bytes, start),
                            simd::find_quote_or_backslash(&bytes, start)
                        );
                        assert_eq!(
                            x86::find_escape_16(&bytes, start),
                            simd::find_escape_scalar(&bytes, start)
                        );
                    }
                    #[cfg(all(feature = "simd", feature = "unstable"))]
                    {
                        assert_eq!(
                            simd::find_quote_or_backslash_portable32(&bytes, start),
                            simd::find_quote_or_backslash(&bytes, start)
                        );
                        assert_eq!(
                            simd::find_quote_or_backslash_portable64(&bytes, start),
                            simd::find_quote_or_backslash(&bytes, start)
                        );
                    }
                }
                bytes[position] = b'a';
            }
        }
    }
}
