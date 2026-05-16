use crate::{
    dtos::pt::CasPasswordStatus,
    spiders::login::{self, pt_headers},
    utils::{cache::invalidate_stuid_cache, client},
};
use anyhow::anyhow;
use log::debug;
use reqwest::StatusCode;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

const USER_INFO_URL: &str =
    "https://pt.hnu.edu.cn/api/basic/info/hndxryxx";
const CARD_INFO_URL: &str =
    "https://pt.hnu.edu.cn/api/hndxYkt/getCardUserInfo/info";
const CARD_HISTORY_URL: &str =
    "https://pt.hnu.edu.cn/api/hndxYkt/getAccHisConsubDzzfLog/detail";
const UNREAD_EMAIL_URL: &str =
    "https://pt.hnu.edu.cn/api/v1/email/unRead/count";
const CSRF_TOKEN_URL: &str =
    "https://pt.hnu.edu.cn/api/security/token";
const PASSWORD_CHECK_URL: &str =
    "http://authority.hnu.cn/authority/services/simpleAuthWS?wsdl";

// 已废弃 → check_password_with_cas
#[cfg_attr(not(test), expect(unused))]
pub async fn check_password(
    stu_id: &str,
    password: &str,
) -> Result<bool, crate::Error> {
    // 对password进行XML实体编码
    // 事实上好像只编码&和<就行了
    // 为什么防止再有问题，都实体编码一下吧
    // 注意&要放在最前面，否则会导致后面的实体编码失效
    let encoded_password = password
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("'", "&apos;")
        .replace(r#"""#, "&quot;");
    let soap_request = format!(
        r#"
        <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/" xmlns:auc="{}">
        <soapenv:Header/>
        <soapenv:Body>
            <auc:commonSimpleAuth>
                <!-- 传递的参数 -->
                <auc:stuid>{}</auc:stuid>
                <auc:ptpass>{}</auc:ptpass>
            </auc:commonSimpleAuth>
        </soapenv:Body>
        </soapenv:Envelope>
        "#,
        PASSWORD_CHECK_URL, stu_id, encoded_password
    );

    let res = client
        .post(PASSWORD_CHECK_URL)
        .header("Content-Type", "text/xml")
        .header("SOAPAction", "commonSimpleAuth")
        .body(soap_request)
        .send()
        .await?;
    let res = res.text().await?;
    if res.contains("SUCCESS") {
        Ok(true)
    } else if res.contains("FAIL") {
        Ok(false)
    } else {
        Err(anyhow!("密码验证失败").into())
    }
}

pub async fn check_password_with_cas(
    stu_id: &str,
    password: &str,
) -> Result<CasPasswordStatus, crate::Error> {
    let res = login::pt_headers(stu_id, Some(password)).await;
    match res {
        Ok(_) => {
            // 把缓存全部重置
            invalidate_stuid_cache(stu_id).await;
            Ok(CasPasswordStatus::Success)
        }
        Err(crate::Error::PasswordError) => {
            Ok(CasPasswordStatus::Fail)
        }
        Err(crate::Error::PasswordShouldChange) => {
            Ok(CasPasswordStatus::ShouldChange)
        }
        Err(crate::Error::PasswordLocked) => {
            Ok(CasPasswordStatus::Lock)
        }
        Err(e) => Err(e),
    }
}

// user_info 未用
pub(crate) async fn get_user_info(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    let url = format!("{USER_INFO_URL}?_={}", duration.as_millis());
    let pt_headers = pt_headers(stu_id, None).await?;
    debug!("{stu_id} 请求用户信息：{}", &url);
    let res = client.get(url).headers(pt_headers).send().await?;
    if res.status() != StatusCode::OK {
        return Err(anyhow!("获取个人信息失败").into());
    }
    let res = res.json().await?;
    Ok(res)
}

pub async fn get_unread_email(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let pt_headers = pt_headers(stu_id, None).await?;
    let res = client
        .get(UNREAD_EMAIL_URL)
        .headers(pt_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    debug!("{stu_id} 请求未读邮件数：{}", UNREAD_EMAIL_URL);
    Ok(res)
}

pub async fn get_card_info(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let pt_headers = pt_headers(stu_id, None).await?;
    let res = client
        .get(CARD_INFO_URL)
        .headers(pt_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    debug!("{stu_id} 请求校园卡信息：{}", CARD_INFO_URL);
    Ok(res)
}

// 一卡通流水；_type：1 消费否则充值
pub async fn get_card_history(
    stu_id: &str,
    year: &str,
    month: &str,
    _type: &str,
) -> Result<Value, crate::Error> {
    let pt_headers = pt_headers(stu_id, None).await?;
    // 字符串格式化默认是左对齐，这里要手动改成右对齐，并且两位宽左侧补0
    let begin_date = format!("{}-{:0>2}-01", year, month);
    // 这里没有必要精确查询日历好像是？直接取31号
    let end_date = format!("{}-{:0>2}-31", year, month);
    let token: Value = client
        .get(CSRF_TOKEN_URL)
        .headers(pt_headers.clone())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let token = token["data"].as_str();
    let token = match token {
        Some(t) => t,
        None => return Err(anyhow!("获取token失败").into()),
    };
    // debug!("{stu_id} 请求校园卡消费历史的token：{}", token);
    let tran_code = if _type == "1" { "15" } else { "16" };
    let form_data = [
        ("beginDate", begin_date.as_str()),
        ("endDate", end_date.as_str()),
        ("pageSize", "100000"),
        ("trancode", tran_code),
    ];

    let res = client
        .post(CARD_HISTORY_URL)
        .headers(pt_headers)
        .header("X-XSRF-TOKEN", token)
        .form(&form_data)
        // .timeout(Duration::from_secs(5)) // 这个请求消费数据比较多的时候比较慢，单独设置5s超时
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    debug!("{stu_id} 请求校园卡消费历史：{}", CARD_HISTORY_URL);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_check_password() {
        let res = check_password(&STU_ID, "").await;
        dbg!(&res.unwrap());
    }

    #[tokio::test]
    async fn test_check_password_with_cas() {
        let res = check_password_with_cas(&STU_ID, "").await;
        dbg!(&res.unwrap());
    }

    #[tokio::test]
    async fn test_get_user_info() {
        let res = get_user_info(&STU_ID).await;
        dbg!(&res.unwrap());
    }

    #[tokio::test]
    async fn test_get_unread_email() {
        let res = get_unread_email(&STU_ID).await;
        dbg!(&res.unwrap());
    }

    #[tokio::test]
    async fn test_get_card_info() {
        let res = get_card_info(&STU_ID).await;
        dbg!(&res.unwrap());
    }

    #[tokio::test]
    async fn test_get_card_history() {
        let res = get_card_history(&STU_ID, "2024", "9", "1").await;
        dbg!(&res.unwrap());
    }
}
