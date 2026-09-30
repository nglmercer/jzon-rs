//! **jzon** — purpose-built, zero-copy JSON serialization for specific structs.
//!
//! # Design
//!
//! Use `#[derive(ToJson, FromJson)]` to generate a **monomorphised** JSON
//! (de)serializer for each of your types at compile time.  No generic visitor
//! indirection, no intermediate `Value` allocation, no format-string overhead.
//!
//! ## Cargo features
//!
//! | Feature | Default | Effect |
//! |---------|---------|--------|
//! | `derive` | ✓ | `#[derive(ToJson, FromJson)]` proc-macros |
//! | `serde` | | `jzon::from_str` / `to_string` for any serde type (module `serde_impl`) |
//! | `compat` | | `jzon::compat` — `serde_json`-compatible API (module `compat`) |
//! | `simd` | | u128 SWAR scanning (16 B/iter) |
//! | `simd-intrinsics` | | hand-written aarch64 NEON / x86_64 SSE2+AVX2 kernels |
//! | `simd + unstable` | | `std::simd` portable SIMD (32–64 B/iter, nightly) |
//! | `fast-float` | | no-op (exact `ryu` / `fast-float2` backends are always on) |
//! | `zmij-float-ser` | | `zmij` float serialization instead of `ryu` |
//! | `stats` | | `ScannerStats` decoded-string/numeric-byte/cache-hit events |
//!
//! For serde integration see [`jzon-rs-serde`](https://crates.io/crates/jzon-rs-serde).
//! For a `serde_json` drop-in see [`jzon-rs-compat`](https://crates.io/crates/jzon-rs-compat).
//!
//! ## Zero-copy deserialization
//!
//! Fields typed `&'de str` borrow **directly** from the input — no `String` is
//! allocated unless the JSON string contains escape sequences.
//!
//! ## Field-hint cache
//!
//! The generated `FromJson` impl maintains a one-word *field-hint* variable
//! that predicts which field key to expect next.  For JSON whose field order
//! matches the struct definition — the common case — almost every key dispatch
//! is O(1) without hashing.
//!
//! ## Unsafe scope
//!
//! Most of the crate is safe Rust.  The small unsafe surface is limited to
//! architecture-specific SIMD kernels. UTF-8 conversion and depth management
//! use safe Rust. See the readiness report for the scope of executed checks.
//!
//! # Quick start
//!
//! ```rust
//! use jzon::{ToJson, FromJson};
//!
//! #[derive(ToJson, FromJson, Debug, PartialEq)]
//! #[serde(rename_all = "camelCase")]
//! struct User<'a> {
//!     user_id:  u64,
//!     name:     &'a str,
//!     #[serde(skip_serializing_if = "Option::is_none")]
//!     email:    Option<String>,
//!     #[serde(default)]
//!     score:    f64,
//! }
//!
//! let input = r#"{"userId":1,"name":"alice","score":9.5}"#;
//! let user: User = User::from_json_str(input).unwrap();
//! let out = user.to_json_string();
//! ```

#![cfg_attr(all(feature = "simd", feature = "unstable"), feature(portable_simd))]
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
/// Allocation types used by generated code without requiring consumer std.
#[doc(hidden)]
pub mod __private {
    pub use alloc::{
        borrow::ToOwned,
        boxed::Box,
        format,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
}

// Enable `std::simd` portable SIMD on nightly when both features are set.

#[cfg(feature = "compat")]
pub mod compat;
pub mod de;
pub mod error;
pub mod fixed;
#[cfg(all(feature = "serde", not(feature = "float_roundtrip")))]
mod native_number;
pub mod scanner;
pub mod ser;
#[cfg(feature = "serde")]
pub mod serde_impl;
pub mod simd;
#[cfg(feature = "simd-intrinsics")]
pub mod simd_arch;
#[cfg(feature = "stats")]
pub mod stats;

pub use de::FromJson;
pub use error::Error;
pub use fixed::{json_str_len, FixedBuf, ToJsonExt};
pub use scanner::{DepthGuard, JsonStr, Scanner};
#[cfg(feature = "std")]
pub use ser::IoSink;
pub use ser::{JsonSink, LengthCounter, ToJson, VecSink};

#[cfg(feature = "derive")]
pub use jzon_derive::{FromJson, ToJson};

// Serde engine entry points live in [`serde_impl`]; the most-used functions
// are re-exported at the crate root. (`serde_impl::Error` keeps its module
// path to avoid clashing with [`Error`].)
#[cfg(all(feature = "serde", feature = "std"))]
pub use serde_impl::{
    from_reader, from_reader_buffered, to_writer, to_writer_buffered, ReaderDeserializer,
    ReaderStream,
};
#[cfg(all(feature = "serde", feature = "std", feature = "stats"))]
pub use serde_impl::{from_reader_buffered_with_stats, from_reader_with_stats};
#[cfg(feature = "serde")]
pub use serde_impl::{
    from_slice, from_str, to_bytes, to_bytes_in, to_string, Deserializer, Serializer,
    StreamDeserializer,
};
#[cfg(all(feature = "serde", feature = "stats"))]
pub use serde_impl::{from_slice_with_stats, from_str_with_stats};
