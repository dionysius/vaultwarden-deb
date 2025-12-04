#[cfg(feature = "serde")]
#[test]
fn str_is_accepted() {
    let input = r#""42 KiB""#;

    let actual = serde_json::from_str::<ubyte::ByteUnit>(&input).unwrap();
    assert_eq!(actual.as_u64(), 42 * 1024);
}

#[cfg(feature = "serde")]
#[test]
fn u64_bytes_is_accepted() {
    let input = r#"42"#;

    let actual = serde_json::from_str::<ubyte::ByteUnit>(&input).unwrap();
    assert_eq!(actual, 42);
}
