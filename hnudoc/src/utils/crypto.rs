//  AES-256-CBC(盐 + MD5)

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::engine::Engine as _;
use base64::engine::general_purpose::STANDARD as base64;
use rand_core::{OsRng, RngCore};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const PASS_PHRASE: &str = "hnudoc-crypto-2026";

#[inline]
fn gen_salt() -> [u8; 8] {
    let mut bytes = [0u8; 8];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

#[inline]
fn passphrase_to_key_and_iv(
    salt: &[u8],
    pass_phrase: &str,
) -> ([u8; 32], [u8; 16]) {
    assert_eq!(salt.len(), 8);
    let h1 = md5::compute([pass_phrase.as_bytes(), salt].concat());
    let h2 = md5::compute(
        [h1.as_slice(), pass_phrase.as_bytes(), salt].concat(),
    );
    let h3 = md5::compute(
        [h2.as_slice(), pass_phrase.as_bytes(), salt].concat(),
    );
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    key[..16].copy_from_slice(h1.as_slice());
    key[16..].copy_from_slice(h2.as_slice());
    iv.copy_from_slice(h3.as_slice());
    (key, iv)
}


pub fn encrypt(data: &str) -> String {
    let salt = gen_salt();
    let (key, iv) = passphrase_to_key_and_iv(&salt, PASS_PHRASE);
    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(&iv);
    let body = Aes256CbcEnc::new(key, iv)
        .encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes());
    let mut out = Vec::with_capacity(16 + body.len());
    out.extend_from_slice(b"Salted__");
    out.extend_from_slice(&salt);
    out.extend_from_slice(&body);
    base64.encode(&out)
}

pub fn decrypt(data: &str) -> anyhow::Result<String> {
    let raw = base64.decode(data)?;
    if raw.len() < 16 || &raw[0..8] != b"Salted__" {
        anyhow::bail!("密文格式无效");
    }
    let salt = &raw[8..16];
    let (key, iv) = passphrase_to_key_and_iv(salt, PASS_PHRASE);
    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(&iv);
    let plain = Aes256CbcDec::new(key, iv)
        .decrypt_padded_vec_mut::<Pkcs7>(&raw[16..])
        .map_err(|_| anyhow::anyhow!("解密失败"))?;
    Ok(String::from_utf8(plain)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let data = "p@ssw0rd 你好";
        let enc = encrypt(data);
        let dec = decrypt(&enc).expect("decrypt failed");
        assert_eq!(dec, data);
    }
}
