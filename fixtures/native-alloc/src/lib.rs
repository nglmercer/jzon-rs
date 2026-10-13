#![no_std]
extern crate alloc;
use alloc::{string::String, vec::Vec};
#[derive(jzon::FromJson, jzon::ToJson)]
pub struct Core {
    name: String,
    #[serde(default)]
    count: u32,
}
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Native {
    name: String,
    values: Vec<u32>,
}
pub fn parse(input: &str) -> Result<Vec<u8>, jzon::serde_impl::Error> {
    let value: Native = jzon::from_str(input)?;
    jzon::to_bytes(&value)
}
pub fn derive(input: &str) -> Result<String, jzon::Error> {
    use jzon::{FromJson, ToJson};
    Ok(Core::from_json_str(input)?.to_json_string())
}

pub fn stream(input: &str) -> usize {
    jzon::serde_impl::Deserializer::from_str(input)
        .into_iter::<Native>()
        .filter_map(Result::ok)
        .count()
}
