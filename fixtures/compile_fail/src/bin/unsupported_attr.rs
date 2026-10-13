#[derive(jzon::FromJson)]
#[serde(from = "u8")]
struct Unsupported {
    value: u8,
}
fn main() {}
