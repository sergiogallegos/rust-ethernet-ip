use rust_ethernet_ip::tag_path::TagPath;

#[test]
fn parse_rejects_negative_indices() {
    assert!(TagPath::parse("MyArray[-1]").is_err());
    assert!(TagPath::parse("MyString.DATA[-1]").is_err());
}

#[test]
fn parse_rejects_oversized_indices() {
    assert!(TagPath::parse("MyArray[4294967296]").is_err());
    assert!(TagPath::parse("MyString.DATA[4294967296]").is_err());
}

#[test]
fn parse_rejects_empty_segments() {
    assert!(TagPath::parse("Program:.Tag").is_err());
    assert!(TagPath::parse("Program:Main..Tag").is_err());
}
