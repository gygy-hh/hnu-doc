pub mod crypto;
pub mod jwt;
pub mod pdf;
pub mod pow;
pub mod serde;
pub mod time;

/// 标准化学号：去除首尾空白并大写
pub fn format_stuid(stu_id: &str) -> String {
    stu_id.trim().to_uppercase()
}
