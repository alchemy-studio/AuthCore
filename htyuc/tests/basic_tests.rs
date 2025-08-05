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
//
// #[test]
// fn test_decode_decrypt() {
//     use aes::Aes128;
//     use block_modes::block_padding::Pkcs7;
//     use block_modes::{BlockMode, Cbc};
//     use hex_literal::hex;
//
//     // create an alias for convenience
//     type Aes128Cbc = Cbc<Aes128, Pkcs7>;
//
//     let key = hex!("000102030405060708090a0b0c0d0e0f");
//     let iv = hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
//     let plaintext = b"Hello world!";
//     let cipher = Aes128Cbc::new_from_slices(&key, &iv).unwrap();
//
//     // buffer must have enough space for message+padding
//     let mut buffer = [0u8; 32];
//     // copy message to the buffer
//     let pos = plaintext.len();
//     buffer[..pos].copy_from_slice(plaintext);
//     let ciphertext = cipher.encrypt(&mut buffer, pos).unwrap();
//
//     assert_eq!(ciphertext, hex!("1b7a4c403124ae2fb52bedc534d82fa8"));
//
//     // re-create cipher mode instance and decrypt the message
//     let cipher = Aes128Cbc::new_from_slices(&key, &iv).unwrap();
//     let mut buf = ciphertext.to_vec();
//     let decrypted_ciphertext = cipher.decrypt(&mut buf).unwrap();
//
//     assert_eq!(decrypted_ciphertext, plaintext);
// }

// #[test]
// fn test_chrono() {
//     use chrono::{NaiveDate, NaiveDateTime};
//     let dt: NaiveDateTime = NaiveDate::from_ymd_opt(2016, 7, 8).and_hms(9, 10, 11);
//     info(format!("{:?}", dt).as_str());
// }

#[ignore]
#[test]
fn test_web_response() {
    use htyuc::models::HtyUser;
    let u = HtyUser {
        hty_id: "".into(),
        union_id: None,
        enabled: false,
        created_at: None,
        real_name: None,
        sex: None,
        mobile: None
    };

    let resp = HtyResponse {
        r: true,
        d: Some(u),
        e: Some("<TEST MESSAGE>".into()),
    };

    assert_eq!("{\"r\":true,\"d\":{\"hty_id\":\"\",\"union_id\":null,\"enabled\":false,\"created_at\":null,\"real_name\":null},\"e\":\"<TEST MESSAGE>\"}",
               serde_json::to_string(&resp).unwrap());
}
