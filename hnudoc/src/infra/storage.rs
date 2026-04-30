//! 本地文件存储：把上传的试卷写到 `upload_dir`，
//! 文件名使用 `<md5>.<ext>`（同名直接覆盖）。

use std::path::{Path, PathBuf};

use tokio::{fs, io::AsyncWriteExt};

use crate::{config::CFG, result::AppResult};

/// 计算字节流的 md5（小写十六进制）
pub fn md5_hex(bytes: &[u8]) -> String {
    let d = md5::compute(bytes);
    hex::encode(d.0)
}

/// 确保上传目录存在
pub async fn ensure_dir() -> AppResult<()> {
    let p = Path::new(&CFG.server.upload_dir);
    if !p.exists() {
        fs::create_dir_all(p).await?;
    }
    Ok(())
}

/// 把文件保存到 `upload_dir/<md5>.<ext>`，返回相对路径（即 `<md5>.<ext>`）
pub async fn save_bytes(
    bytes: &[u8],
    md5: &str,
    ext: &str,
) -> AppResult<String> {
    ensure_dir().await?;
    let ext = ext.trim_start_matches('.').to_ascii_lowercase();
    let filename = if ext.is_empty() {
        md5.to_string()
    } else {
        format!("{md5}.{ext}")
    };
    let mut full = PathBuf::from(&CFG.server.upload_dir);
    full.push(&filename);
    let mut f = fs::File::create(&full).await?;
    f.write_all(bytes).await?;
    f.flush().await?;
    Ok(filename)
}

/// 给定相对路径，得到完整本地路径
pub fn absolute_path(relative: &str) -> PathBuf {
    let mut p = PathBuf::from(&CFG.server.upload_dir);
    p.push(relative);
    p
}

/// 删除某个相对路径下的文件，找不到时不报错（如替换/拒绝 pending 时可用）
#[allow(dead_code)]
pub async fn remove(relative: &str) -> AppResult<()> {
    let p = absolute_path(relative);
    if p.exists() {
        fs::remove_file(p).await?;
    }
    Ok(())
}
