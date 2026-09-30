use jzon::serde_impl::{self as native, Category, Deserializer};
use serde::Deserialize;
#[test]
fn categories_positions_and_public_constructor() {
    for (input, line, column) in [
        ("{:1}", 1, 2),
        ("{\n :1}", 2, 2),
        ("[1,]", 1, 4),
        ("[", 1, 1),
        ("true false", 1, 6),
    ] {
        let error = native::from_str::<serde_json::Value>(input).unwrap_err();
        let reference = serde_json::from_str::<serde_json::Value>(input).unwrap_err();
        assert_eq!(
            (error.line(), error.column()),
            (line, column),
            "{input}: {error}"
        );
        assert_eq!(
            format!("{:?}", error.classify()),
            format!("{:?}", reference.classify()),
            "{input}"
        );
    }
    let mut de = Deserializer::from_str("\n256");
    let error = u8::deserialize(&mut de).unwrap_err();
    assert_eq!(error.classify(), Category::Data);
    assert_eq!((error.line(), error.column()), (2, 3));
    let mut de = Deserializer::from_str("[1,]");
    let error = Vec::<u8>::deserialize(&mut de).unwrap_err();
    assert_eq!((error.line(), error.column()), (1, 4));
}

#[test]
fn valid_wrong_types_are_data_errors() {
    fn compare<T: serde::de::DeserializeOwned>(input: &str) {
        let ours = native::from_str::<T>(input).err().unwrap();
        let reference = serde_json::from_str::<T>(input).err().unwrap();
        assert_eq!(
            format!("{:?}", ours.classify()),
            format!("{:?}", reference.classify()),
            "{input}: {ours}"
        );
        assert_eq!(
            (ours.line(), ours.column()),
            (reference.line(), reference.column()),
            "{input}: {ours}"
        );
        let reader = native::from_reader::<_, T>(input.as_bytes()).err().unwrap();
        assert_eq!(
            reader.classify(),
            ours.classify(),
            "reader {input}: {reader}"
        );
        assert_eq!(
            (reader.line(), reader.column()),
            (ours.line(), ours.column()),
            "reader {input}: {reader}"
        );
    }
    for input in ["null", "1", "\"x\"", "[]", "{}"] {
        compare::<bool>(input);
    }
    for input in ["null", "true", "\"x\"", "[]", "{}"] {
        compare::<u8>(input);
    }
    for input in ["null", "1", "true", "[]", "{}"] {
        compare::<String>(input);
    }
    for input in ["null", "1", "true", "\"x\"", "{}"] {
        compare::<Vec<u8>>(input);
    }
}
