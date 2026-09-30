#![no_std]
extern crate alloc;
pub fn parse(input: &str) -> serde_json::Result<serde_json::Value> {
    serde_json::from_str(input)
}
pub fn serialize(value: &serde_json::Value) -> serde_json::Result<alloc::string::String> {
    serde_json::to_string(value)
}
