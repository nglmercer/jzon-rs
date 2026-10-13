# Audited local safety patch

This is serde_json 1.0.151 registry source, retaining upstream MIT/Apache licenses
and package identity for type interoperability. `src/de.rs` MapKey bool parsing
now peeks at the first key byte, consuming it only in true/false literal paths.
The invalid-key path parses the full string rather than splitting a Unicode
scalar before StrRead's unchecked UTF-8 conversion. Callback dispatch is unchanged.

This is NOT a new fixed upstream release. Published facade packages would lose
path dependencies during Cargo normalization. Consumers must patch crates.io
serde_json to this exact audited source at their workspace root. A registry-only
0.4.0 release remains blocked until a verified safe upstream release or an
explicitly migrated maintained fork is available. No publication is authorized.
