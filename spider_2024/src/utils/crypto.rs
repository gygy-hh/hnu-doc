#![allow(unused)]
//! crypto_2024项目的库导出，方便直接调用，不再通过构建网络请求的方式
use aes::cipher::{
    self, BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit,
    block_padding::Pkcs7, generic_array::GenericArray,
};
use anyhow::{Result, anyhow};
use base64::engine::{
    Engine as _, general_purpose::STANDARD as base64,
};
use num_bigint::BigUint;
use rand_core::{OsRng, RngCore};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;
type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;

const PASS_PHRASE: &str = "qnxg-crypto-2023";
const GRADUATE_KEY: &str = "southsoft12345!#";
const LAB_KEY: &str = "1234567891234567";

/// 生成一个长度为8的随机salt
#[inline]
fn gen_salt() -> [u8; 8] {
    let mut rng = OsRng;
    let mut bytes = [0u8; 8];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// openssl的加密方式，使用md5生成Aes Cbc 256的key和iv，以下是伪代码
/// ```text
/// hash1_128 = MD5(Passphrase + Salt)
/// hash2_128 = MD5(hash1_128 + Passphrase + Salt)
/// hash3_128 = MD5(hash2_128 + Passphrase + Salt)
/// Key = hash1_128 + hash2_128
/// IV  = hash3_128;
/// ```
#[inline]
fn passphrase_to_key_and_iv(
    salt: &[u8],
    pass_phrase: &str,
) -> ([u8; 32], [u8; 16]) {
    assert_eq!(salt.len(), 8);
    let hash1 = md5::compute([pass_phrase.as_bytes(), salt].concat());
    let hash2 = md5::compute(
        [hash1.as_slice(), pass_phrase.as_bytes(), salt].concat(),
    );
    let hash3 = md5::compute(
        [hash2.as_slice(), pass_phrase.as_bytes(), salt].concat(),
    );
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    let temp = [hash1.as_slice(), hash2.as_slice()].concat();
    key.copy_from_slice(&temp);
    iv.copy_from_slice(hash3.as_slice());
    (key, iv)
}

/// 加密函数，采用Aes256Cbc加密和Pkcs7填充
pub fn encrypt(data: &str) -> String {
    // 生成一个长度为8的随机字符串
    let salt = gen_salt();

    let (key, iv) = passphrase_to_key_and_iv(&salt, PASS_PHRASE);
    let key = GenericArray::from_slice(key.as_slice());
    let iv = GenericArray::from_slice(iv.as_slice());
    let res = Aes256CbcEnc::new(key, iv)
        .encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes());
    // 添加 Salted__ 和 salt 前缀
    let prefix = b"Salted__";
    let res = [prefix, &salt, res.as_slice()].concat();
    base64.encode(&res)
}

/// 解密函数，可能会返回错误
pub fn decrypt(data: &str) -> Result<String> {
    let decode = base64.decode(data)?;
    if decode.len() < 16 {
        return Err(anyhow!("Invalid data length"));
    }
    let salt = &decode[8..16];
    let (key, iv) = passphrase_to_key_and_iv(salt, PASS_PHRASE);
    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(&iv);
    let res = Aes256CbcDec::new(key, iv)
        .decrypt_padded_vec_mut::<Pkcs7>(&decode[16..])?;
    Ok(String::from_utf8(res)?)
}

/// graduate研究生系统的解密函数
pub fn graduate_decrypt(data: &str) -> Result<String> {
    let decode = base64.decode(data)?;
    let key =
        GenericArray::from_slice(&GRADUATE_KEY.as_bytes()[..16]);
    let res = Aes128EcbDec::new(key)
        .decrypt_padded_vec_mut::<Pkcs7>(&decode)?;
    Ok(String::from_utf8(res)?)
}

/// graduate研究生系统的加密函数
///
/// 开发时用来调试
pub fn graduate_encrypt(data: &str) -> String {
    let key =
        GenericArray::from_slice(&GRADUATE_KEY.as_bytes()[..16]);
    let res = Aes128EcbEnc::new(key)
        .encrypt_padded_vec_mut::<Pkcs7>(data.as_bytes());
    base64.encode(&res)
}

/// 个人门户登录加密函数
pub(crate) fn rsa_encrypt(
    password: &str,
    exponent: &str,
    modulus: &str,
) -> String {
    let password_bytes = password.as_bytes();
    let password_int = BigUint::from_bytes_be(password_bytes);
    let e_int =
        BigUint::parse_bytes(exponent.as_bytes(), 16).unwrap();
    let m_int = BigUint::parse_bytes(modulus.as_bytes(), 16).unwrap();

    let result_int = password_int.modpow(&e_int, &m_int);
    format!("{:0>128}", result_int.to_str_radix(16))
}

/// 教务系统的base64编码
pub fn hdjw_encrypt(e: &str) -> String {
    if e.is_empty() {
        return String::new();
    }

    let encoded = urlencoding::encode(e);
    let n = base64.encode(encoded.as_bytes());
    let t: Vec<char> = n.chars().collect();
    let mut o = t.clone();

    if t.len() < 8 {
        return String::new();
    }

    o[1] = t[t.len() - 2];
    o[3] = t[t.len() - 4];
    o[5] = t[t.len() - 6];
    o[7] = t[t.len() - 8];
    o[t.len() - 2] = t[1];
    o[t.len() - 4] = t[3];
    o[t.len() - 6] = t[5];
    o[t.len() - 8] = t[7];

    let temp: String = o.into_iter().collect();
    format!("QZDATASOFT{}", temp)
}

pub fn lab_encrypt(e: &str) -> String {
    let cipher =
        Aes128EcbEnc::new_from_slice(LAB_KEY.as_bytes()).unwrap();
    let plaintext = e.as_bytes();
    let ciphertext =
        cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    let p1 = base64.encode(&ciphertext);
    base64.encode(p1.as_bytes())
}

/// 教务系统的base64解码
///
/// 开发时使用来调试
fn hdjw_decrypt(e: &str) -> Option<String> {
    if !e.starts_with("QZDATASOFT") {
        return None;
    }
    let temp = &e["QZDATASOFT".len()..];
    let t: Vec<char> = temp.chars().collect();
    let mut o = t.clone();

    if t.len() < 8 {
        return None;
    }

    o[1] = t[t.len() - 2];
    o[3] = t[t.len() - 4];
    o[5] = t[t.len() - 6];
    o[7] = t[t.len() - 8];
    o[t.len() - 2] = t[1];
    o[t.len() - 4] = t[3];
    o[t.len() - 6] = t[5];
    o[t.len() - 8] = t[7];

    let decoded: String = o.into_iter().collect();
    let decoded_bytes = base64.decode(decoded).ok()?;
    let decoded_str = String::from_utf8(decoded_bytes).ok()?;
    let decoded_str = urlencoding::decode(&decoded_str).ok()?;
    Some(decoded_str.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = "123456";
        let encrypted = encrypt(data);
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_graduate_encrypt_decrypt() {
        let data = "123456";
        let encrypted = graduate_encrypt(data);
        let decrypted = graduate_decrypt(&encrypted).unwrap();
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_lab_encrypt() {
        let data = "202402050201";
        let encrypted = lab_encrypt(data);
        println!("{}", encrypted);
    }
}
