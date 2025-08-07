use base64::Engine;
use htycommons::common::{BASE64_DECODER, HtyResponse};
use htycommons::logger::info;

#[test]
fn test_base64_decode() {
    let bytes = BASE64_DECODER.decode("aGVsbG8gd29ybGQ=").unwrap();
    let s = std::str::from_utf8(&bytes).unwrap();
    info(format!("{:?}", s).as_str());
    assert_eq!("hello world", s);
}
