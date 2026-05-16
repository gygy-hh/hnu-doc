use std::{collections::HashMap, time::Duration};

use crate::{
    spiders::login::lab_headers,
    utils::{
        self,
        cache::{CACHE, CacheEnum},
        captcha::{CaptchaType, captcha_solve},
        client,
        request::cookie_parser,
    },
};
use anyhow::anyhow;
use log::{debug, error};
use rand::Rng;
use reqwest::header::SET_COOKIE;
use serde_json::Value;

#[expect(clippy::upper_case_acronyms)]
enum RequestMethod {
    GET,
    POST(HashMap<&'static str, String>),
}

const LAB_LIST_URL: &str =
    "http://10.62.106.112/XPK/StuCourseElectiveLook/LoadTableInfo";
const LOGIN_URL: &str =
    "http://10.62.106.112/BaseInfo/Login/ValidateLogin";
const CAPTCHA_URL: &str =
    "http://10.62.106.112/Ashx/CheckCode.ashx?t=0.29911677684547566";
const SEM_INFO_URL: &str =
    "http://10.62.106.112/Common/Common/GetSemDropDownList?HasNull=0";
const COURSE_LIST_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentScoreList";
const LAB_SCORE_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentLabScore";
const VIRTUAL_LAB_SCORE_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetStudentFZLabScore";
const LAB_SCORE_STRUCTURE_URL: &str = "http://10.62.106.112/XPK/StudentScoreSearch/GetLabScoreStructure";
const LAB_SCORE_DETAIL_URL: &str =
    "http://10.62.106.112/XPK/StudentScoreSearch/ShowScore";

// 大物实验 API；密码错/缺密码 → Ok(Null)；自动换 cookie
async fn request_lab(
    url: &str,
    stu_id: &str,
    method: RequestMethod,
) -> Result<Value, crate::Error> {
    let mut tried = 0;
    let mut err_log = String::new();
    let data;
    loop {
        if tried >= 2 {
            error!("请求实验平台失败多次，错误日志：{}", err_log);
            return Err(anyhow!("请求实验平台失败").into());
        }
        if tried > 0 {
            // 失败了就等一会儿再试
            let wait_time = rand::thread_rng().gen_range(200..500);
            tokio::time::sleep(Duration::from_millis(wait_time))
                .await;
        }
        let lab_headers = match lab_headers(stu_id).await {
            Ok(data) => data,
            Err(crate::Error::PasswordError) => {
                // 密码错误或是没有密码，直接返回，不重试了
                // 我们这里不直接往上抛 PasswordError，因为 PasswordError
                // 会被后端直接转发到前端，而前端目前认为这个错误一律是个人门户密码错误
                return Ok(Value::Null);
            }
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 获取实验平台请求头失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        let res = match method {
            RequestMethod::GET => {
                client.get(url).headers(lab_headers).send().await
            }
            RequestMethod::POST(ref form_data) => {
                client
                    .post(url)
                    .headers(lab_headers)
                    .form(&form_data)
                    .send()
                    .await
            }
        };
        if let Err(e) = res {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 请求实验平台失败: err = {}; stuid = {}",
                tried, e, stu_id
            ));
            continue;
        }
        let res = res.unwrap();
        let res = res.error_for_status();
        if let Err(e) = res {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 请求实验平台失败: err = {}; stuid = {}",
                tried, e, stu_id
            ));
            continue;
        }
        let res = res.unwrap();
        let body = res.text().await;
        if let Err(e) = body {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 读取实验平台响应失败: err = {}; stuid = {}",
                tried, e, stu_id
            ));
            continue;
        }
        let body = body.unwrap();
        match serde_json::from_str::<Value>(&body) {
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 解析实验平台响应失败: err = {}; body = {}; stuid = {}",
                    tried, e, body, stu_id
                ));
                // 这种情况（200 返回码但不是 json 格式（应该是 html 格式））大概是 cookie
                // 过期，我们清理缓存
                CACHE
                    .invalidate(&(
                        CacheEnum::LabCookie,
                        stu_id.into(),
                    ))
                    .await;
                continue;
            }
            Ok(json) => {
                data = json;
                break;
            }
        }
    }
    Ok(data)
}

// 本学期实验课表
pub async fn get_lab_list(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("CourseID", "-999".to_string());
    form_data.insert("weeks", "-999".to_string());
    form_data.insert("labID", "-999".to_string());
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "200".to_string());
    let res = request_lab(
        LAB_LIST_URL,
        stu_id,
        RequestMethod::POST(form_data),
    )
    .await?;
    Ok(res)
}

// 验密 + 验证码；(Json, cookies)
pub async fn check_password(
    stu_id: &str,
    password: &str,
) -> Result<(Value, String), crate::Error> {
    let password = utils::crypto::lab_encrypt(password);
    let mut tried = 0;
    let mut checkcode = String::new();
    let mut all_cookies = String::new();
    while tried < 5 {
        let res = client
            .post(LOGIN_URL)
            .form(&[
                ("uname", stu_id),
                ("pwd", &password),
                ("checkcode", &checkcode),
            ])
            .header("Cookie", &all_cookies)
            .send()
            .await?
            .error_for_status()?;
        let cookies =
            cookie_parser(res.headers().get_all(SET_COOKIE));
        if !cookies.is_empty() {
            all_cookies
                .push_str(&format!("; {}", cookies.join("; ")));
        }
        let data: Value = res.json().await?;
        if let Some(code) = data["RTNCode"].as_i64() {
            if code == -2 {
                // 需要验证码
                let res = client
                    .get(CAPTCHA_URL)
                    .header("Cookie", &all_cookies)
                    .send()
                    .await?
                    .error_for_status()?;
                let img_bytes = res.bytes().await?;
                checkcode =
                    captcha_solve(&img_bytes, CaptchaType::Default)
                        .await?;
                tried += 1;
            } else {
                debug!("经过 {} 次尝试后成功登录实验平台", tried + 1);
                return Ok((data, all_cookies));
            }
        } else {
            return Err(anyhow!("意料之外的响应: {}", data))?;
        }
    }
    // 尝试多次后仍然无法通过，返回错误
    Err(anyhow!("解析验证码失败").into())
}

// 学期字典
pub async fn get_sem_info(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let res =
        request_lab(SEM_INFO_URL, stu_id, RequestMethod::GET).await?;
    Ok(res)
}

// 某学期课程+总评（sem 来自学期接口）
pub async fn get_course_list(
    stu_id: &str,
    sem: &str,
) -> Result<Value, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "15".to_string());
    form_data.insert("SemID", sem.to_string());
    form_data.insert("UserID", stu_id.to_string());
    let res = request_lab(
        COURSE_LIST_URL,
        stu_id,
        RequestMethod::POST(form_data),
    )
    .await?;
    Ok(res)
}

// 单次实验成绩明细
pub async fn get_lab_score(
    stu_id: &str,
    sem: &str,
    course_id: &str,
) -> Result<Value, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "15".to_string());
    form_data.insert("SemID", sem.to_string());
    form_data.insert("CourseID", course_id.to_string());
    form_data.insert("UserID", stu_id.to_string());
    let res = request_lab(
        LAB_SCORE_URL,
        stu_id,
        RequestMethod::POST(form_data),
    )
    .await?;
    Ok(res)
}

// 虚拟实验成绩（参数实际影响弱）
pub async fn get_virtual_lab_score(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let mut form_data = HashMap::new();
    form_data.insert("page", "1".to_string());
    form_data.insert("rows", "15".to_string());
    // 既然怎么给都无所谓，就随便给
    form_data.insert("SemID", "0".to_string());
    form_data.insert("CourseID", "0".to_string());
    form_data.insert("UserID", stu_id.to_string());
    let res = request_lab(
        VIRTUAL_LAB_SCORE_URL,
        stu_id,
        RequestMethod::POST(form_data),
    )
    .await?;
    Ok(res)
}

// 成绩结构树
pub async fn get_score_structure(
    stu_id: &str,
    course_id: &str,
) -> Result<Value, crate::Error> {
    let url =
        format!("{}?CourseID={}", LAB_SCORE_STRUCTURE_URL, course_id);
    let res = request_lab(&url, stu_id, RequestMethod::GET).await?;
    Ok(res)
}

// 成绩结构下明细
pub async fn get_score_detail(
    stu_id: &str,
    course_id: &str,
) -> Result<Value, crate::Error> {
    let url = format!(
        "{}?CourseID={}&StudentID={}",
        LAB_SCORE_DETAIL_URL, course_id, stu_id
    );
    let res = request_lab(&url, stu_id, RequestMethod::GET).await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::utils::request::STU_ID;

    use super::*;

    const SEM_ID: &str = "18";
    const COURSE_ID: &str = "68";

    #[tokio::test]
    async fn test_get_sem_info() {
        let res = get_sem_info(&STU_ID).await.unwrap();
        dbg!(&res);
    }

    #[tokio::test]
    async fn test_get_course_list() {
        let res = get_course_list(&STU_ID, SEM_ID).await.unwrap();
        dbg!(&res);
    }

    #[tokio::test]
    async fn test_get_lab_score() {
        let res =
            get_lab_score(&STU_ID, SEM_ID, COURSE_ID).await.unwrap();
        dbg!(&res);
    }

    #[tokio::test]
    async fn test_get_virtual_lab_score() {
        let res = get_virtual_lab_score(&STU_ID).await.unwrap();
        dbg!(&res);
    }

    #[tokio::test]
    async fn test_get_score_structure() {
        let res =
            get_score_structure(&STU_ID, COURSE_ID).await.unwrap();
        dbg!(&res);
    }

    #[tokio::test]
    async fn test_get_score_detail() {
        let res = get_score_detail(&STU_ID, COURSE_ID).await.unwrap();
        dbg!(&res);
    }
}
