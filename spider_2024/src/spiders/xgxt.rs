use crate::{
    dtos::xgxt::PersonInfo,
    spiders::login::xgxt_headers,
    utils::{
        client,
        redis::{add_cookie_to_redis, get_cookie_from_redis},
    },
};
use anyhow::anyhow;
use serde_json::Value;
use std::collections::HashMap;
use tokio::try_join;

// 个人信息
const USER_INFO_URL: &str = "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/userInfo/default.zf?dataId=null";
// 在校信息
const IN_SCHOOL_INFO_URL: &str = "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/zxxx/default.zf?dataId=null";
// 联系方式
const CONTACT_INFO_URL: &str = "https://xgxt.hnu.edu.cn/zftal-xgxt-web/dynamic/form/group/lxfs1/default.zf?dataId=null";

// 这个接口请求比较多，且请求一次数据量也比较大（有个接口直接把近十年所有的班级数据全部返回了），
// 需要进行缓存，且可能是一个性能瓶颈 TODO 考虑并发三个请求
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
pub async fn get_person_info(
    stu_id: &str,
) -> Result<PersonInfo, crate::Error> {
    // 如果有缓存直接提前返回
    if let Ok(res) =
        get_cookie_from_redis("person_info", stu_id).await
    {
        return Ok(serde_json::from_str::<PersonInfo>(&res)?);
    }
    let xgxt_headers = xgxt_headers(stu_id).await?;
    let user_info_req = client
        .get(USER_INFO_URL)
        .headers(xgxt_headers.clone())
        .send();
    let in_school_info_req = client
        .get(IN_SCHOOL_INFO_URL)
        .headers(xgxt_headers.clone())
        .send();
    let contact_info_req =
        client.get(CONTACT_INFO_URL).headers(xgxt_headers).send();
    let mut entries = HashMap::<String, String>::new();
    let res = try_join!(
        user_info_req,
        in_school_info_req,
        contact_info_req
    )?;
    for i in [res.0, res.1, res.2] {
        i.error_for_status()?
            .json::<Value>()
            .await?
            .get("data")
            .ok_or(anyhow!("no data"))?
            .get("groupFields")
            .ok_or(anyhow!("no groupFields"))?
            .get(0)
            .ok_or(anyhow!("no groupFields"))?
            .get("fields")
            .ok_or(anyhow!("no fields"))?
            .as_array()
            .ok_or(anyhow!("no fields"))?
            .iter()
            .for_each(|field| {
                if let Some(field_name) = field.get("fieldName")
                    && let Some(value) = field.get("defaultValue")
                {
                    if !field_name.is_string() {
                        return;
                    }
                    let key =
                        field_name.as_str().unwrap().to_string();
                    // 这个 value 是什么类型不好说，所以分别考虑
                    if let Some(v) = value.as_str() {
                        entries.insert(key, v.to_string());
                    } else if let Some(v) = value.as_i64() {
                        entries.insert(key, v.to_string());
                    }
                }
            })
    }
    // 特殊处理一下性别
    // 事实上，下面获取的很多字段都是数字代码，需要进一步解析
    let gender = if let Some(g) = entries.remove("性别") {
        if g == "1" {
            "男".to_string()
        } else if g == "2" {
            "女".to_string()
        } else {
            g
        }
    } else {
        "".to_string()
    };
    // 这里全部 unwrap_or("")，因为可能有的人有些字段没有值
    let res = PersonInfo {
        name: entries.remove("姓名").unwrap_or("".to_string()),
        gender,
        politic: entries.remove("政治面貌").unwrap_or("".to_string()),
        race: entries.remove("民族").unwrap_or("".to_string()),
        hometown: entries.remove("籍贯").unwrap_or("".to_string()),
        level: entries.remove("培养层次").unwrap_or("".to_string()),
        academy: entries.remove("学院").unwrap_or("".to_string()),
        major: entries.remove("专业").unwrap_or("".to_string()),
        class: entries.remove("班级").unwrap_or("".to_string()),
        dormitory: entries.remove("寝室楼").unwrap_or("".to_string()),
        room: entries.remove("寝室号").unwrap_or("".to_string()),
        phone: entries.remove("手机号码").unwrap_or("".to_string()),
        wechat: entries.remove("微信号").unwrap_or("".to_string()),
        qq: entries.remove("QQ号码").unwrap_or("".to_string()),
        email: entries.remove("电子邮箱").unwrap_or("".to_string()),
        enter_year: entries
            .remove("年级")
            .unwrap_or("".to_string())
            .parse()
            .unwrap_or(0),
        xz: entries
            .remove("学制(年)")
            .unwrap_or("".to_string())
            .parse()
            .unwrap_or(0),
        stu_id: entries.remove("学号").unwrap_or("".to_string()),
    };
    // 个人信息基本不会变动，直接缓存一周
    let cache_time = 7 * 24 * 60 * 60;
    add_cookie_to_redis(
        "person_info",
        &serde_json::to_string(&res)?,
        stu_id,
        cache_time,
    )
    .await?;
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_get_person_info() {
        dbg!(get_person_info(&STU_ID).await.unwrap());
    }
}
