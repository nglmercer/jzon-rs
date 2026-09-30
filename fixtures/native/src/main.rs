fn generic<T: serde::Serialize + ?Sized>(v: &T) {
    jzon_serde::to_string(v).unwrap();
    jzon_serde::to_bytes(v).unwrap();
    jzon_serde::to_writer(Vec::new(), v).unwrap();
}
fn main() {
    generic("str");
    generic(&[1, 2][..]);
    generic(&42);
}
