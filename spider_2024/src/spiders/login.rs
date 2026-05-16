// 登录缓存允许并发回填，避免锁拖垮成功率

use std::sync::LazyLock;

use crate::{
    spiders,
    utils::{
        cache::{CACHE, CacheEnum::*},
        client,
        crypto::rsa_encrypt,
        redis::{fetch_lab_password, fetch_password},
        request::cookie_parser,
    },
};
use anyhow::{Result, anyhow};
use log::debug;
use regex::Regex;
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, LOCATION, SET_COOKIE},
};
use serde_json::{Value, json};

// CAS 常量（须 http）
// const LOGO_URL: &str = "http://cas.hnu.edu.cn/favicon.ico";
const _LOGIN_URL: &str = "http://cas.hnu.edu.cn/cas/login";
const PUBKEY_URL: &str = "http://cas.hnu.edu.cn/cas/v2/getPubKey";
const HDJW_FROM_CAS_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://hdjw.hnu.edu.cn/gld/sso.jsp";
const HDJW_ENTER_URL: &str = "http://hdjw.hnu.edu.cn/gld/sso.jsp";
const PT_URL: &str =
    "http://cas.hnu.edu.cn/cas/login?service=https://pt.hnu.edu.cn/"; // WARN 注意这个url后面必须带`/`，不然无法正常跳转
const NETFLOW_URL: &str = "http://cas.hnu.edu.cn/application/sso.zf?login=B5712DC2FA281C96E053026B3E0A80A6";
const SERVICE_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://cas.hnu.edu.cn/system/login/login.zf"; // 这个是sso.zf跳转用到的一个链接
const GYM_URL_DIRECT_LOGIN: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Login/login";
const GYM_URL_FROM_CAS: &str = "http://cas.hnu.edu.cn/application/sso.zf?login=898A822E9695C137E053026B3E0A65D7";
const GRADUATE_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://yjsxt.hnu.edu.cn/gmis/oauthLogin/hndxnew";
const CA_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=https://ca.hnu.edu.cn/student/";
const XGXT_URL: &str = "http://cas.hnu.edu.cn/cas/login?service=http://xgxt.hnu.edu.cn/zftal-xgxt-web/teacher/xtgl/index/check.zf";

pub struct LoginParams {
    modulus: String,
    exponent: String,
    execution: String,
    event_id: String,
    cookies: Vec<String>,
}

pub enum GetLoginParamsRes {
    Success(LoginParams), // 成功获取到登录参数
    Skip(String),         // 已经登录成立了自动跳转
}

// 拉 CAS 登录表单参数
#[inline]
pub async fn get_login_params(
    service_url: &str,
    cas_cookie: Option<&str>,
) -> Result<GetLoginParamsRes> {
    // 尝试登录对应系统
    let mut login_req = client.get(service_url);
    if let Some(v) = cas_cookie {
        login_req = login_req.header(COOKIE, v)
    };
    let login_res = match login_req.send().await?.error_for_status() {
        Ok(res) => res,
        // 这种情况可能是cookie失效
        Err(e) => {
            debug!("登录失败，尝试二次登录：{}", e);
            client
                .get(service_url)
                .send()
                .await?
                .error_for_status()?
        }
    };
    // 302就提前返回
    if login_res.status() == StatusCode::FOUND {
        debug!("已经登录成功，跳转到对应服务");
        let ticket_url = login_res
            .headers()
            .get("location")
            .ok_or(anyhow!("获取ticket失败"))?
            .to_str()?;
        return Ok(GetLoginParamsRes::Skip(ticket_url.to_string()));
    }
    if login_res.status() != StatusCode::OK {
        return Err(anyhow!("访问失败"));
    }
    // 获取到登录页的set-cookie
    let mut cookies =
        cookie_parser(login_res.headers().get_all(SET_COOKIE));
    // 拿到登录表单的execution和_eventId
    let login_text = login_res.text().await?;
    static EXECUTION_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"name="execution" value="(.*?)""#).unwrap()
    });
    static EVENT_ID_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"name="_eventId" value="(.*?)""#).unwrap()
    });
    let execution = EXECUTION_RE
        .captures(&login_text)
        .and_then(|cap| cap.get(1))
        .map_or("", |m| m.as_str())
        .to_string();
    let event_id = EVENT_ID_RE
        .captures(&login_text)
        .and_then(|cap| cap.get(1))
        .map_or("", |m| m.as_str())
        .to_string();
    // 通过pubkey接口获取modulus和exponent
    let pubkey = client
        .get(PUBKEY_URL)
        .header(COOKIE, &cookies.join("; "))
        .send()
        .await?
        .error_for_status()?;
    // 获取pubkey的cookies
    cookies
        .extend(cookie_parser(pubkey.headers().get_all(SET_COOKIE)));
    let pubkey: Value = pubkey.json().await?;
    let (modulus, exponent) = (
        pubkey["modulus"]
            .as_str()
            .ok_or(anyhow!("modulus not found"))?
            .to_string(),
        pubkey["exponent"]
            .as_str()
            .ok_or(anyhow!("exponent not found"))?
            .to_string(),
    );

    let login_params = LoginParams {
        modulus,
        exponent,
        execution,
        event_id,
        cookies,
    };
    Ok(GetLoginParamsRes::Success(login_params))
}

// ticket URL；可选缓存 CasCookie；显式 password 则绕过缓存
pub async fn get_ticket_url(
    stu_id: &str,
    service_url: &str,
    password: Option<&str>,
) -> Result<String, crate::Error> {
    let mut cas_cache = CACHE.get(&(CasCookie, stu_id.into())).await;
    if password.is_some() {
        // 提供密码则不适用 CasCookie 缓存
        cas_cache = None;
    }
    let login_params =
        match get_login_params(service_url, cas_cache.as_deref())
            .await?
        {
            // 如果是跳过登录的情况，就提前返回ticket_url
            GetLoginParamsRes::Skip(ticket_url) => {
                return Ok(ticket_url);
            }
            GetLoginParamsRes::Success(v) => v,
        };

    let pending_password = match password {
        Some(v) => v.to_string(),
        None => fetch_password(stu_id).await?,
    };
    let rsa_password = rsa_encrypt(
        &pending_password,
        &login_params.exponent,
        &login_params.modulus,
    );

    // Post登录表单
    let login = client
        .post(service_url)
        // Content-Type 由 reqwest 自动带
        .header(COOKIE, &login_params.cookies.join("; "))
        .form(&[
            ("username", stu_id),
            ("password", &rsa_password),
            ("authcode", ""),
            ("execution", &login_params.execution),
            ("_eventId", &login_params.event_id),
        ])
        .send()
        .await?;
    if login.status() == StatusCode::FORBIDDEN {
        return Err(crate::Error::PasswordLocked);
    }
    debug!("{stu_id} 发送了登录请求");
    // login_params里面的pv0在后面的请求也会有用(netflow)
    let addition: Vec<String> = login_params
        .cookies
        .into_iter()
        .filter(|cookie| cookie.starts_with("_pv0="))
        .collect(); // 错误已在前面被处理，一定会有_pv0
    let mut cookies =
        cookie_parser(login.headers().get_all(SET_COOKIE));
    cookies.extend(addition);
    let location = login
        .headers()
        .get(LOCATION)
        .ok_or(crate::Error::PasswordError)?
        .to_str()?
        .to_string();
    const PASSWORD_SHOULD_CHANGE_PAT: &str =
        "cas.hnu.edu.cn/securitycenter/modifyPwd/index.zf";
    if location.contains(PASSWORD_SHOULD_CHANGE_PAT) {
        return Err(crate::Error::PasswordShouldChange);
    }
    let to_store = cookies.join("; ");
    let to_return = Ok(location);
    if password.is_none() {
        CACHE.insert((CasCookie, stu_id.into()), to_store).await;
    }
    to_return
}

// 教务 hdjw Cookie
pub async fn hdjw_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((Hdjw, stu_id.into()), async {
            // 需要先请求 hdjw 的登录页面，获取到相关的 cookie
            let cookies = cookie_parser(
                client
                    .get(HDJW_ENTER_URL)
                    .send()
                    .await?
                    .error_for_status()?
                    .headers()
                    .get_all(SET_COOKIE),
            )
            .join("; ");
            let ticket_url =
                get_ticket_url(stu_id, HDJW_FROM_CAS_URL, None)
                    .await?;
            debug!("{stu_id} 尝试通过 {} 访问教务系统", ticket_url);
            // ticket 请求需带前置 cookie，否则 500
            client
                .get(ticket_url)
                .header(COOKIE, &cookies)
                .send()
                .await?
                .error_for_status()?;
            // 上面的请求会重定向到 HDJW_ENTER_URL，我们再访问一下。
            let res = client
                .get(HDJW_ENTER_URL)
                .header(COOKIE, &cookies)
                .send()
                .await?
                .error_for_status()?;
            // 随后又会被重定向到一个新的链接，再请求一下就会得到 hdjw 鉴权的 cookie
            if res.status() != StatusCode::FOUND {
                return Err(anyhow!(
                    "获取教务系统失败，HTTP代码 {} {}",
                    res.status(),
                    res.text().await.unwrap_or_default()
                )
                .into());
            }
            let target_url = res
                .headers()
                .get(LOCATION)
                .ok_or(anyhow!("获取重定向链接失败"))?
                .to_str()?;
            let new_cookies = cookie_parser(
                client
                    .get(target_url)
                    .header(COOKIE, &cookies)
                    .send()
                    .await?
                    .error_for_status()?
                    .headers()
                    .get_all(SET_COOKIE),
            )
            .join("; ");
            // 保险起见，将两次 cookie 合并一下
            Ok(format!("{}; {}", cookies, new_cookies))
        })
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

// 个人门户 Cookie；可纯验密；缓存 PtCookie
pub async fn pt_headers(
    stu_id: &str,
    password: Option<&str>,
) -> Result<HeaderMap, crate::Error> {
    let cached_cookies = if password.is_none() {
        CACHE.get(&(PtCookie, stu_id.into())).await
    } else {
        None
    };
    let cookies = if let Some(v) = cached_cookies {
        v
    } else {
        let ticket_url =
            get_ticket_url(stu_id, PT_URL, password).await?;
        debug!("{stu_id} 尝试通过 {} 访问个人门户", ticket_url);
        let res = client
            .get(ticket_url)
            .send()
            .await?
            .error_for_status()?;
        if res.status() != StatusCode::FOUND {
            return Err(anyhow!("获取个人门户失败").into());
        }
        let res = cookie_parser(res.headers().get_all(SET_COOKIE))
            .join("; ");
        if password.is_none() {
            CACHE
                .insert((PtCookie, stu_id.into()), res.clone())
                .await;
        }
        res
    };
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

// sso.zf：手动跟重定向拿 s_ticket（网费/体测等）
async fn get_sticket(
    stu_id: &str,
    url: &str,
) -> Result<(String, String), crate::Error> {
    // 先请求一下，防止还没登录。拿到登录后的 cookies
    get_ticket_url(stu_id, SERVICE_URL, None).await?;
    let cas_cache = CACHE
        .get(&(CasCookie, stu_id.into()))
        .await
        .unwrap_or_default();
    // client 禁止自动跳转，手动跟随
    let mut now_url = url.to_string();
    let mut cookies = cas_cache;
    let mut s_ticket = None;
    // 至多 6 跳取 s_ticket
    for _ in 0..6 {
        if now_url.starts_with(
            "https://cas.hnu.edu.cn/sprcialapp/zf_form/index.zf",
        ) {
            s_ticket = Some(
                now_url
                    .split('&')
                    .find(|s| s.starts_with("s_ticket="))
                    .ok_or(anyhow!("获取s_ticket失败"))?
                    .split('=')
                    .collect::<Vec<&str>>()[1],
            );
            break;
        }
        let res = client
            .get(now_url)
            .header(COOKIE, &cookies)
            .send()
            .await?
            .error_for_status()?;
        if res.status() != StatusCode::FOUND {
            return Err(anyhow!(
                "获取s_ticket时失败，HTTP代码 {}",
                res.status()
            )
            .into());
        }
        now_url = res
            .headers()
            .get(LOCATION)
            .ok_or(anyhow!("获取重定向链接失败"))?
            .to_str()?
            .to_string();
        cookies = format!(
            "{}; {}",
            cookies,
            cookie_parser(res.headers().get_all(SET_COOKIE))
                .join("; ")
        );
    }
    if let Some(v) = s_ticket {
        Ok((v.to_string(), cookies))
    } else {
        Err(anyhow!("获取s_ticket失败，未找到s_ticket").into())
    }
}

// 校园网流量页
pub async fn netflow_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((NetflowCookie, stu_id.into()), async {
            let (s_ticket, cookies) =
                get_sticket(stu_id, NETFLOW_URL).await?;
            // 发送请求
            let res = client
                .get("http://ll.hnu.edu.cn/login/validate")
                .header(COOKIE, &cookies)
                .form(&[
                    ("s_ticket", s_ticket.as_str()),
                    ("login_id", stu_id),
                    ("password", ""),
                    ("null", ""),
                ])
                .send()
                .await?
                .error_for_status()?;
            // 获取cookies
            let cookies =
                cookie_parser(res.headers().get_all(SET_COOKIE));
            // 保留Token，有三个.ASPXAUTH，
            // 只要最后面的一个（这里就不写死是cookies[0]和cookies[3]了）
            if cookies.is_empty() {
                return Err(anyhow!("校园网流量登录失败").into());
            }
            let res = format!(
                "{}; {}",
                cookies.first().unwrap(),
                cookies.last().unwrap()
            );
            Ok(res)
        })
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

// 体测直连登录
pub async fn gym_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = if let Some(v) =
        CACHE.get(&(GymCookie, stu_id.into())).await
    {
        v
    } else {
        let password = fetch_password(stu_id).await?;
        let res = client
            .post(GYM_URL_DIRECT_LOGIN)
            .form(&[("student_num", stu_id), ("password", &password)])
            .send()
            .await?
            .error_for_status()?;
        let cookies =
            cookie_parser(res.headers().get_all(SET_COOKIE))
                .join("; ");
        let res: Value = res.json().await?;
        if res["info"] != "登录成功" {
            return Err(anyhow!("登录失败").into());
        }
        CACHE
            .insert((GymCookie, stu_id.into()), cookies.clone())
            .await;
        cookies
    };
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

// 体测经 CAS
pub async fn gym_headers_from_cas(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE.try_get_with((GymCookie, stu_id.into()), async {
        let (s_ticket, _) =
                get_sticket(stu_id, GYM_URL_FROM_CAS).await?;
        // 发送请求
        let _res = client
            .get("http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/view/login/loginPage.html")
            .query(&[("s_ticket", s_ticket.as_str()), ("login_id", stu_id)])
            .send()
            .await?
            .error_for_status()?;
        let res = client
            .post("http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Login/ticketLogin")
            .form(&[("s_ticket", s_ticket.as_str()), ("login_id", stu_id)])
            .send()
            .await?
            .error_for_status()?;
        // 解除并发锁
        // map_remove(GYM_LOCK, stu_id);
        let cookie = cookie_parser(res.headers().get_all(SET_COOKIE));
        let res: Value = res.json().await?;
        if res["info"] != "登录成功" {
            return Err(anyhow!("登录失败").into());
        }
        let res = cookie.join("; ");
        CACHE.insert((GymCookie, stu_id.into()), res.clone()).await;
        Ok(res)
    }).await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

// 研究生 GMIS：Cookie + id
pub async fn graduate_headers_and_id(
    stu_id: &str,
) -> Result<(HeaderMap, String), crate::Error> {
    let result = CACHE
        .try_get_with((GraduateCookieAndId, stu_id.into()), async {
            let ticket_url =
                get_ticket_url(stu_id, GRADUATE_URL, None).await?;
            debug!("{stu_id} 尝试通过 {} 访问研究生系统", ticket_url);
            // 获取id，就是ticket_url的/gmis/和/之间的内容
            let res = client
                .get(ticket_url)
                .send()
                .await?
                .error_for_status()?;
            if res.status() != StatusCode::FOUND {
                return Err(anyhow!("获取研究生系统失败").into());
            }
            // 访问跳转路径
            let redirection = res
                .headers()
                .get(LOCATION)
                .ok_or(anyhow!("获取研究生跳转路径失败"))?
                .to_str()?;
            let id =
                redirection.split("/gmis/").collect::<Vec<&str>>()[1]
                    .split('/')
                    .collect::<Vec<&str>>()[0]
                    .to_string();
            let new_url =
                format!("http://yjsxt.hnu.edu.cn{}", redirection);
            let res = client
                .get(&new_url)
                .send()
                .await?
                .error_for_status()?;
            let cookies: String =
                cookie_parser(res.headers().get_all(SET_COOKIE))
                    .join("; ");
            if cookies.is_empty() {
                return Err(anyhow!("获取研究生系统失败").into());
            }
            Ok(json!((cookies, id)).to_string())
        })
        .await;

    let (cookies, id): (String, String) =
        serde_json::from_str(&result?)?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok((headers, id))
}

// CA 可信电子凭证
pub async fn ca_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let ticket_url = get_ticket_url(stu_id, CA_URL, None).await?;
    debug!("{stu_id} 尝试通过 {} 访问可信电子凭证", ticket_url);
    client.get(&ticket_url).send().await?.error_for_status()?;
    let ticket =
        ticket_url.split("ticket=").collect::<Vec<&str>>()[1];
    let res: Value =
        client.get(format!("https://ca.hnu.edu.cn/student/cas/client/validateLogin?ticket={ticket}%23%2F&service=https:%2F%2Fca.hnu.edu.cn%2Fstudent%2F"))
        .send().await?
        .error_for_status()?
        .json().await?;

    if res["message"] != "登录成功" {
        return Err(anyhow!("登录失败").into());
    }
    let token = res["result"]["token"].as_str().unwrap();
    let cookie = format!("X-Access-Token={token}");
    let mut headers = HeaderMap::new();
    headers.insert("X-Access-Token", token.parse()?);
    headers.insert("Cookie", cookie.parse()?);
    Ok(headers)
}

// 学工 Cookie
pub async fn xgxt_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((XGXTCookie, stu_id.into()), async {
            let ticket_url =
                get_ticket_url(stu_id, XGXT_URL, None).await?;
            debug!("{stu_id} 尝试通过 {} 访问学工系统", ticket_url);
            // cas 下发的 ticket_url 是 http 的，但是学工系统要用 https
            let res = client
                .get(ticket_url.replace("http://", "https://"))
                .send()
                .await?;
            if res.status() != StatusCode::FOUND {
                return Err(anyhow!(
                    "获取学工系统失败，HTTP代码 {}",
                    res.status()
                )
                .into());
            }
            let cookies: String =
                cookie_parser(res.headers().get_all(SET_COOKIE))
                    .join("; ");
            if cookies.is_empty() {
                return Err(anyhow!(
                    "获取学工系统失败，接收到空的 cookie"
                )
                .into());
            }
            Ok(cookies)
        })
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

// 大物实验：密码错/无密码时返回 null，上层透传
pub async fn lab_headers(
    stu_id: &str,
) -> Result<HeaderMap, crate::Error> {
    let cookies = CACHE
        .try_get_with((LabCookie, stu_id.into()), async {
            let password = fetch_lab_password(stu_id).await?;
            if password.is_none() {
                return Err(crate::Error::PasswordError);
            }
            let password = password.unwrap();
            let (obj, cookies) =
                spiders::lab::check_password(stu_id, &password)
                    .await?;
            if obj["RTNCode"] == -1 {
                // 密码错误
                return Err(crate::Error::PasswordError);
            }
            if obj["RTNCode"] != 1 {
                let msg = obj["Data"].as_str().unwrap_or("未知错误");
                return Err(
                    anyhow!("登录实验平台失败：{}", msg).into()
                );
            }
            if cookies.is_empty() {
                return Err(anyhow!(
                    "登录实验平台失败：接收到空的 cookie"
                )
                .into());
            }
            Ok(cookies)
        })
        .await?;
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookies.parse()?);
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_hdjw() {
        let (hdjw1, hdjw2) = tokio::join!(
            hdjw_headers(&STU_ID),
            hdjw_headers(&STU_ID)
        );
        println!("{:#?} {:#?}", hdjw1.unwrap(), hdjw2.unwrap());
    }

    #[tokio::test]
    async fn test_pt() {
        let (pt1, pt2, pt3, pt4, pt5) = tokio::join!(
            pt_headers(&STU_ID, None),
            pt_headers(&STU_ID, None),
            pt_headers(&STU_ID, None),
            pt_headers(&STU_ID, None),
            pt_headers(&STU_ID, None)
        );
        println!(
            "{:#?} {:#?} {:#?} {:#?} {:#?}",
            pt1.unwrap(),
            pt2.unwrap(),
            pt3.unwrap(),
            pt4.unwrap(),
            pt5.unwrap()
        );
    }

    #[tokio::test]
    async fn test_netflow() {
        // 缓存cas的cookie
        // let cas_cache = pt_headers(&STU_ID).await.unwrap();
        // tokio::time::sleep(Duration::from_secs(2)).await;
        let (netflow1, netflow2) = tokio::join!(
            netflow_headers(&STU_ID),
            netflow_headers(&STU_ID)
        );
        println!("{:#?} {:#?}", netflow1.unwrap(), netflow2.unwrap());
    }

    #[tokio::test]
    async fn test_gym() {
        let (gym1, gym2) =
            tokio::join!(gym_headers(&STU_ID), gym_headers(&STU_ID));
        println!("{:#?} {:#?}", gym1.unwrap(), gym2.unwrap());
    }

    #[tokio::test]
    async fn test_gym_from_cas() {
        let gym = gym_headers_from_cas(&STU_ID).await;
        println!("{:#?}", gym.unwrap());
    }

    #[tokio::test]
    async fn test_graduate() {
        let graduate = graduate_headers_and_id(&STU_ID).await;
        println!("{:#?}", graduate.unwrap());
    }

    #[tokio::test]
    async fn test_ca() {
        let ca = ca_headers(&STU_ID).await;
        println!("{:#?}", ca.unwrap());
    }

    #[tokio::test]
    async fn test_xgxt() {
        let xgxt = xgxt_headers(&STU_ID).await;
        println!("{:#?}", xgxt.unwrap());
    }

    #[tokio::test]
    async fn test_lab() {
        let lab = lab_headers(&STU_ID).await;
        println!("{:#?}", lab.unwrap());
    }
}
