// PoW：验证 MD5(key) 前 zero 位为 0，且 key 以 ticket 为前缀

use rand::{Rng, distributions::Alphanumeric};

// 随机 ticket
pub fn gen_ticket() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}

// 校验 PoW
pub fn verify(key: &str, ticket: &str, zero: u32) -> bool {
    if !key.starts_with(ticket) {
        return false;
    }
    let digest = md5::compute(key.as_bytes());
    let bytes = digest.0;
    let full_bytes = (zero / 8) as usize;
    let rem_bits = (zero % 8) as u8;
    if bytes.iter().take(full_bytes).any(|b| *b != 0) {
        return false;
    }
    if rem_bits == 0 {
        return true;
    }
    let next = bytes.get(full_bytes).copied().unwrap_or(0);
    (next >> (8 - rem_bits)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用暴力搜 key
    fn find_key(ticket: &str, zero: u32) -> String {
        for n in 0u64.. {
            let key = format!("{ticket}{n}");
            if verify(&key, ticket, zero) {
                return key;
            }
        }
        unreachable!()
    }

    #[test]
    fn pow_correct() {
        let ticket = "abcdef";
        let key = find_key(ticket, 8);
        assert!(verify(&key, ticket, 8));
        // 前缀错误
        assert!(!verify(&format!("X{key}"), ticket, 8));
    }
}
