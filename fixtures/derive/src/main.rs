use jzon::{FromJson, ToJson};
#[derive(ToJson, FromJson)]
struct Point {
    x: u32,
}
fn main() {
    let p = Point::from_json_str("{\"x\":1}").unwrap();
    assert_eq!(p.to_json_string(), "{\"x\":1}");
}
