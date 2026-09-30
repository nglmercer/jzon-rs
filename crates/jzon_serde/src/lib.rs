//! `jzon_serde` — serde-compatible JSON serializer/deserializer backed by jzon's
//! SIMD string escaping and zero-copy scanner.
//!
//! Uses existing Serde traits. This native engine has its own errors and
//! a buffered reader and streaming writer; assess its documented contract separately
//! from the delegated compatibility facade.
//! This crate re-exports the engine from
//! [`jzon`](https://crates.io/crates/jzon-rs) (feature `serde`); it exists so
//! `Mode B` users only depend on one small crate.
//!
//! # Usage
//!
//! ```rust
//! use jzon_serde::{to_string, from_str};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Point { x: f64, y: f64 }
//!
//! let p = Point { x: 1.0, y: 2.0 };
//! let json = to_string(&p).unwrap();
//! let p2: Point = from_str(&json).unwrap();
//! assert_eq!(p, p2);
//! ```

pub use jzon::serde_impl::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[test]
    fn shim_roundtrip() {
        let p = Point { x: 1.5, y: -2.0 };
        let json = to_string(&p).unwrap();
        let p2: Point = from_str(&json).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn shim_slice_bytes_forms() {
        let v = vec![1u64, 2, 3];
        let bytes = to_bytes(&v).unwrap();
        let back: Vec<u64> = from_slice(&bytes).unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn shim_matches_serde_json() {
        let v = vec![1u64, 2, 3];
        assert_eq!(to_string(&v).unwrap(), serde_json::to_string(&v).unwrap());
    }
}
