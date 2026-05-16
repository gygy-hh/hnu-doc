pub mod crypto;
pub mod jwt;
pub mod pdf;
pub mod pow;
pub mod serde;
pub mod time;

// 学号：trim + 大写
pub fn format_stuid(stu_id: &str) -> String {
    stu_id.trim().to_uppercase()
}
