use log::debug;
use anyhow::anyhow;
use ring::{rand, signature::{self, KeyPair}};
use crate::common::{HtyErr, HtyErrCode};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HtyKeyPair {
    pub pubkey: Option<String>,
    pub privkey: Option<String>,
}

pub fn generate_cert_key_pair() -> anyhow::Result<HtyKeyPair> {
    const SEED_LEN: usize = 32;

    let secure_random = rand::SystemRandom::new();
    let seed: [u8; SEED_LEN];
    // let seed: [u8; SEED_LEN] = rand::generate(&secure_random)?.expose();

    match rand::generate(&secure_random) {
        Ok(rand_seed) => {
            debug!("Generate random seed ok");
            seed = rand_seed.expose();
        }
        Err(_error) => {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("Generate random seed fail".to_string()),
            }));
        }
    }

    let priv_key = hex::encode(seed.as_ref().to_vec());
    debug!("Generate private key is {:?}", priv_key.clone());

    let key_pair;

    match signature::Ed25519KeyPair::from_seed_unchecked(&seed) {
        Ok(ed25519_key_pair) => {
            debug!("Ed25519KeyPair generate ok");
            key_pair = ed25519_key_pair;
        }

        Err(_error) => {
            return Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("Ed25519KeyPair generate fail".to_string()),
            }));
        }
    }

    let pub_key = hex::encode(key_pair.public_key().as_ref().to_vec());
    debug!("Generate public key is {:?}", pub_key.clone());

    Ok(HtyKeyPair{ pubkey: Some(pub_key), privkey: Some(priv_key) })
}

pub fn encrypt_text_with_private_key(private_key: String, message: String) -> anyhow::Result<String> {
    let private_key_seed = hex::decode(private_key).expect("Decoding failed");
    match signature::Ed25519KeyPair::from_seed_unchecked(private_key_seed.as_ref()) {
        Ok(key_pair) => {
            let encrypt_message = key_pair.sign(message.as_ref());
            let encrypt_message_string = hex::encode(encrypt_message.as_ref().to_vec());
            debug!(" Encrypt message {:?}", encrypt_message_string);
            Ok(encrypt_message_string)
        }
        Err(error) => {
            Err(anyhow!(HtyErr {
                code: HtyErrCode::NullErr,
                reason: Some("Encrypt cert from private seed fail".to_string() + &error.to_string()),
            }))
        }
    }
}

pub fn verify(public_key_string: String, encrypted_text: String, verify_text: String) -> anyhow::Result<bool> {
    debug!("::verify_cert:: public_key_string -> {:?} / encrypted_text -> {:?} / verify_text -> {:?}", public_key_string, encrypted_text, verify_text);

    let public_key =
        signature::UnparsedPublicKey::new(&signature::ED25519,
                                          hex::decode(public_key_string).expect("Decoding failed"));
    debug!("::verify_cert:: verifying...");
    public_key.verify(verify_text.as_bytes(), hex::decode(encrypted_text)?.as_ref())
        .map_err(|e| anyhow!(HtyErr {
            code: HtyErrCode::NullErr,
            reason: Some(format!("Verification failed: {:?}", e)),
        }))?;
    debug!("::verify_cert:: Ok.");
    Ok(true)
}

