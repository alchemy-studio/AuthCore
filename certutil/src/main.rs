use std::env;
use htycommons::cert::{encrypt_text_with_private_key};

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    let private_key = args[1].clone();
    let text = args[2].clone();

    // Debug
    // println!("certutil -> private_key : {:?}" , private_key.clone());
    // println!("certutil -> app_id : {:?}" , app_id.clone());

    let enc_text = encrypt_text_with_private_key(private_key, text.clone())?;
    println!("{}", enc_text);

    // println!("{}", verify(text.clone(), enc_text.clone(), text.clone()).unwrap());

    Ok(())
}