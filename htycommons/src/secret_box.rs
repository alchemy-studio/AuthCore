//! Symmetric seal/open for short secrets (e.g. per-teacher LLM API keys) using JWT_KEY-derived AES-128-CBC.

use aes::cipher::{
    block_padding::Pkcs7, BlockModeDecrypt, BlockModeEncrypt, KeyIvInit,
};
use anyhow::Context;
use base64::Engine;
use cbc::{Decryptor, Encryptor};
use ring::rand::{self, SecureRandom};

type Aes128CbcEnc = Encryptor<aes::Aes128>;
type Aes128CbcDec = Decryptor<aes::Aes128>;

fn derive_key_16() -> [u8; 16] {
    let jwt = crate::jwt::jwt_key();
    let mut key = [0u8; 16];
    for (i, b) in jwt.as_bytes().iter().take(16).enumerate() {
        key[i] = *b;
    }
    key
}

/// Encrypt `plain` → base64(iv || ciphertext).
pub fn seal_secret(plain: &str) -> anyhow::Result<String> {
    let key = derive_key_16();
    let mut iv = [0u8; 16];
    rand::SystemRandom::new()
        .fill(&mut iv)
        .map_err(|e| anyhow::anyhow!("random iv: {e}"))?;

    let plain_bytes = plain.as_bytes();
    let mut buf = vec![0u8; plain_bytes.len() + 16];
    buf[..plain_bytes.len()].copy_from_slice(plain_bytes);
    let ciphertext = Aes128CbcEnc::new_from_slices(&key, &iv)
        .context("seal_secret: invalid key/iv")?
        .encrypt_padded::<Pkcs7>(&mut buf, plain_bytes.len())
        .context("seal_secret: encrypt failed")?;

    let mut out = iv.to_vec();
    out.extend_from_slice(ciphertext);
    Ok(base64::engine::general_purpose::STANDARD.encode(out))
}

/// Decrypt base64 blob from [`seal_secret`].
pub fn open_secret(sealed: &str) -> anyhow::Result<String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(sealed.trim())
        .context("open_secret: base64 decode")?;
    if raw.len() <= 16 {
        anyhow::bail!("open_secret: ciphertext too short");
    }
    let (iv, ct) = raw.split_at(16);
    let key = derive_key_16();
    let mut buf = ct.to_vec();
    let plain = Aes128CbcDec::new_from_slices(&key, iv)
        .context("open_secret: invalid key/iv")?
        .decrypt_padded::<Pkcs7>(&mut buf)
        .context("open_secret: decrypt failed")?;
    Ok(String::from_utf8(plain.to_vec()).context("open_secret: utf8")?)
}

pub fn mask_secret(secret: &str) -> String {
    let s = secret.trim();
    if s.len() <= 8 {
        return "****".to_string();
    }
    format!(
        "{}****{}",
        &s[..4.min(s.len())],
        &s[s.len().saturating_sub(4)..]
    )
}
