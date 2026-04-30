//! 时间相关工具：与参考后端约定一致，数据库中的时间均为 UTC+8

/// 当前 UTC+8 时间（NaiveDateTime）
pub fn now() -> chrono::NaiveDateTime {
    (chrono::Utc::now() + chrono::Duration::hours(8)).naive_utc()
}
