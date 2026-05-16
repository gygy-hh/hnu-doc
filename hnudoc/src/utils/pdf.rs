// PDF 页数（lopdf 读 /Pages /Count，失败返回 0）

use std::path::Path;

// 从路径读页数
#[allow(dead_code)]
pub fn page_count(path: impl AsRef<Path>) -> Option<u32> {
    let doc = lopdf::Document::load(path).ok()?;
    let pages = doc.get_pages();
    Some(pages.len() as u32)
}

// 从字节解析页数
pub fn page_count_bytes(bytes: &[u8]) -> Option<u32> {
    let doc = lopdf::Document::load_mem(bytes).ok()?;
    let pages = doc.get_pages();
    Some(pages.len() as u32)
}
