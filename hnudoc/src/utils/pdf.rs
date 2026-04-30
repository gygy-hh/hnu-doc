//! PDF 页数解析。
//!
//! 用 `lopdf` 打开文件读 catalog 中的 `/Pages -> /Count`。
//! 解析失败时返回 0，由调用方决定是否拒绝上传。

use std::path::Path;

/// 从磁盘路径读页数（与 [`page_count_bytes`](fn.page_count_bytes) 二选一即可）
#[allow(dead_code)]
pub fn page_count(path: impl AsRef<Path>) -> Option<u32> {
    let doc = lopdf::Document::load(path).ok()?;
    let pages = doc.get_pages();
    Some(pages.len() as u32)
}

/// 直接根据字节流解析页数
pub fn page_count_bytes(bytes: &[u8]) -> Option<u32> {
    let doc = lopdf::Document::load_mem(bytes).ok()?;
    let pages = doc.get_pages();
    Some(pages.len() as u32)
}
