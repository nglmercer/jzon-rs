fn generic<T: serde::Serialize + ?Sized>(v: &T) {
    jzon_serde::to_string(v).unwrap();
    jzon_serde::to_bytes(v).unwrap();
    jzon_serde::to_writer(Vec::new(), v).unwrap();
    jzon_serde::to_writer_buffered(Vec::new(), v).unwrap();
    let mut reusable = jzon_serde::Serializer::with_capacity(1024);
    reusable.serialize(v).unwrap();
    reusable.clear();
    let mut streaming: jzon_serde::Serializer<jzon_serde::WriterSink<Vec<u8>>> =
        jzon_serde::Serializer::from_writer(Vec::new());
    streaming.serialize(v).unwrap();
    assert!(!streaming.into_inner().into_inner().is_empty());
}
fn main() {
    generic("str");
    generic(&[1, 2][..]);
    generic(&42);
}
