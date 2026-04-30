//! 工作量证明（Proof of Work）
//!
//! 服务端下发：
//! - `ticket`：随机字符串
//! - `zero`：要求计算出来的 hash 前 `zero` 个比特全为 0
//!
//! 客户端任务：找一个字符串 `s`，使得 `MD5(ticket || s)` 的前 `zero`
//! 比特为 0。我们约定客户端最终上送的 `key = ticket + s`，所以这里
//! 同时校验 `key` 是否以 `ticket` 开头。

use rand::{Rng, distributions::Alphanumeric};

/// 生成一个 24 位的随机 ticket
pub fn gen_ticket() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}

/// 检查 `key` 是否满足 PoW：
/// 1. `key` 以 `ticket` 为前缀
/// 2. `MD5(key)` 的前 `zero` 个比特全为 0
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

    /// 暴力寻找一个满足 zero 比特要求的 key（仅测试小 zero）
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
