/// Accumulated statistics from a single `FromJson` parse run.
#[derive(Debug, Clone, Default)]
pub struct ScannerStats {
    /// String values returned as zero-copy `&'de str` borrows.
    pub zero_copy_borrows: u64,
    /// Escaped string values decoded by read_str; not total allocator calls.
    pub decoded_strings: u64,
    /// Numeric token bytes consumed by read_number_bytes; excludes other paths.
    pub number_bytes_scanned: u64,
    /// Field-dispatch hint cache correct predictions.
    pub hint_hits: u64,
    /// Field-dispatch hint cache misses requiring full dispatch.
    pub hint_misses: u64,
}

impl ScannerStats {
    pub fn hint_hit_rate(&self) -> Option<f64> {
        let total = self.hint_hits + self.hint_misses;
        if total == 0 {
            None
        } else {
            Some(self.hint_hits as f64 / total as f64)
        }
    }

    pub fn zero_copy_rate(&self) -> Option<f64> {
        let total = self.zero_copy_borrows + self.decoded_strings;
        if total == 0 {
            None
        } else {
            Some(self.zero_copy_borrows as f64 / total as f64)
        }
    }
}
