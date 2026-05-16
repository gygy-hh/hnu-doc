// 时间工具（DB 存 UTC+8）

// 当前 UTC+8
pub fn now() -> chrono::NaiveDateTime {
    (chrono::Utc::now() + chrono::Duration::hours(8)).naive_utc()
}
