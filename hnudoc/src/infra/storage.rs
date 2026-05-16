// 上传目录：<md5>.<ext>

use std::path::{Path, PathBuf};

use tokio::{fs, io::AsyncWriteExt};

use crate::{config::CFG, result::AppResult};

// md5 hex
pub fn md5_hex(bytes: &[u8]) -> String {
    let d = md5::compute(bytes);
    hex::encode(d.0)
}

// mkdir uploads
pub async fn ensure_dir() -> AppResult<()> {
    let p = Path::new(&CFG.server.upload_dir);
    if !p.exists() {
        fs::create_dir_all(p).await?;
    }
    Ok(())
}

// 写入并返回相对文件名
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

// 绝对路径
pub fn absolute_path(relative: &str) -> PathBuf {
    let mut p = PathBuf::from(&CFG.server.upload_dir);
    p.push(relative);
    p
}

// 删文件（不存在忽略）
#[allow(dead_code)]
pub async fn remove(relative: &str) -> AppResult<()> {
    let p = absolute_path(relative);
    if p.exists() {
        fs::remove_file(p).await?;
    }
    Ok(())
}
