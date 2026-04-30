use crate::{config::CFG, utils::client};
use anyhow::anyhow;
use serde::Deserialize;

pub enum CaptchaType {
    Default,
}

impl std::fmt::Display for CaptchaType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            CaptchaType::Default => write!(f, "default"),
        }
    }
}

#[derive(Deserialize)]
struct CaptchaResponse {
    error: Option<String>,
    result: Option<String>,
}

pub async fn captcha_solve(
    img_bytes: &[u8],
    captcha_type: CaptchaType,
) -> Result<String, crate::Error> {
    let url = format!(
        "{}/ocr?type={}",
        CFG.captcha.captcha_url, captcha_type
    );
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(img_bytes.to_vec())
            .file_name("captcha.jpg"),
    );
    let res = client.post(&url).multipart(form).send().await?;
    let res = res.error_for_status()?;
    let body = res.text().await?;
    let res: CaptchaResponse = serde_json::from_str(&body)?;
    if let Some(result) = res.result {
        Ok(result)
    } else {
        Err(anyhow!(
            "验证码服务错误: {}",
            res.error.unwrap_or("未知错误".to_string())
        )
        .into())
    }
}
