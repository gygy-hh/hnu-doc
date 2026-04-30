use std::{collections::HashMap, time::Duration};

use crate::{
    spiders::login::{ca_headers, hdjw_headers},
    utils::{
        cache::{CACHE, CacheEnum},
        client,
    },
};
use anyhow::anyhow;
use log::error;
use rand::Rng;
use serde_json::Value;

// 课表这里的课程信息接口是分页的，我这里设置了一页 50 条，应该没有人一学期超过 50 门课吧（）
// 教务系统有点诡异，这个 pageSize 最好不要设置太大。我们发现，如果设置 200 这个特殊数字就会返回 html 的格式，其他数字都会返回 json 格式。具体原因不明，但是不建议太大，适量就好
// 该 URL 缺少学期的参数，需要后续再 format
const CLASS_TABLE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xskb/xskb_list.do?viweType=1&needData=1&pageNum=1&pageSize=50&viweType=1&demoStr=&needData=1&baseUrl=%2Fjsxsd&sfykb=2&xsflMapListJsonStr=%E8%AE%B2%E8%AF%BE%E5%AD%A6%E6%97%B6%2C%E6%8C%87%E5%AF%BC%E5%AD%A6%E6%97%B6%2C%E5%AE%9E%E9%AA%8C%E5%AD%A6%E6%97%B6%2C%E5%85%B6%E4%BB%96%2C&zc=&kbjcmsid=1";
// 无课表课程
const CLASS_TABLE_EXTRA: &str = "http://hdjw.hnu.edu.cn/jsxsd/xskb/xskb_list.do?viweType=2&needData=1&pageNum=1&pageSize=20&viweType=2&demoStr=&needData=1&baseUrl=%2Fjsxsd&sfykb=1&xsflMapListJsonStr=%E8%AE%B2%E8%AF%BE%E5%AD%A6%E6%97%B6%2C%E6%8C%87%E5%AF%BC%E5%AD%A6%E6%97%B6%2C%E5%AE%9E%E9%AA%8C%E5%AD%A6%E6%97%B6%2C%E5%85%B6%E4%BB%96%2C&zc=&kbjcmsid=1";
// 课程成绩查询接口。其他说明同上
const GRADE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/kscj/cjcx_list?pageNum=1&pageSize=50&kcxz=&kcsx=&kcmc=&xsfs=all&sfxsbcxq=1";
// 总成绩排名接口
const GRADE_RANK_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xscjsq/cjpmcx_list.do?&pageNum=1&pageSize=20&kclx=&kcly=1";
const EMPTY_CLASSROOM_URL: &str =
    "http://hdjw.hnu.edu.cn/jsxsd/kbxx/jsjy_query2";
const EXAM_SCHEDULE_URL: &str = "http://hdjw.hnu.edu.cn/jsxsd/xsks/xsksap_list?pageNum=1&pageSize=20&xqlb=";
const GRADE_DETAIL_URL: &str =
    "http://hdjw.hnu.edu.cn/jsxsd/kscj/pscj_list.do?zcj=&jx0404id=";

enum RequestMethod {
    Get,
    Post(HashMap<&'static str, String>),
}

/// 专门用于请求教务系统的函数。该函数可以自动进行错误处理和处理 cookie 失效
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
async fn request_hdjw(
    url: &str,
    stu_id: &str,
    method: RequestMethod,
) -> Result<Value, crate::Error> {
    if stu_id.starts_with('S') {
        // 研究生就先别凑热闹了
        return Err(anyhow!("暂不支持研究生教务系统").into());
    }
    let mut tried = 0;
    let mut err_log = String::new();
    let data;
    loop {
        if tried >= 2 {
            error!("请求教务系统失败多次，错误日志：{}", err_log);
            return Err(anyhow!("请求教务系统失败").into());
        }
        if tried > 0 {
            // 失败了就等一会儿再试
            let wait_time = rand::thread_rng().gen_range(200..500);
            tokio::time::sleep(Duration::from_millis(wait_time))
                .await;
        }
        let hdjw_headers = match hdjw_headers(stu_id).await {
            Ok(data) => data,
            // 账号异常直接返回，不重试了
            Err(crate::Error::PasswordError) => {
                return Err(crate::Error::PasswordError);
            }
            Err(crate::Error::PasswordShouldChange) => {
                return Err(crate::Error::PasswordShouldChange);
            }
            Err(crate::Error::PasswordLocked) => {
                return Err(crate::Error::PasswordLocked);
            }
            Err(e) => {
                tried += 1;
                err_log.push_str(&format!(
                    "({}) 获取教务系统请求头失败: err = {}; stuid = {}",
                    tried, e, stu_id
                ));
                continue;
            }
        };
        let res = match method {
            RequestMethod::Get => {
                client.get(url).headers(hdjw_headers).send().await
            }
            RequestMethod::Post(ref form_data) => {
                client
                    .post(url)
                    .headers(hdjw_headers)
                    .form(&form_data)
                    .send()
                    .await
            }
        };
        if let Err(e) = res {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 请求教务系统失败: err = {}; stuid = {}",
                tried, e, stu_id
            ));
            continue;
        }
        let res = res.unwrap();
        let res = res.error_for_status();
        if let Err(e) = res {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 请求教务系统失败: err = {}; stuid = {}",
                tried, e, stu_id
            ));
            continue;
        }
        let res = res.unwrap();
        let body = res.text().await;
        if let Err(e) = body {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 读取教务系统响应失败: err = {}; stuid = {}",
                tried, e, stu_id
            ));
            continue;
        }
        let body = body.unwrap();
        if body.contains("window.initQzTable") {
            // 说明是课程分数详情的响应，我们特殊处理对待一下
            return Ok(Value::String(body));
        }
        let json = serde_json::from_str::<Value>(&body);
        if let Err(e) = json {
            tried += 1;
            err_log.push_str(&format!(
                "({}) 解析教务系统响应失败: err = {}; body = {}; stuid = {}",
                tried, e, body, stu_id
            ));
            // 这种情况（200 返回码但不是 json 格式（应该是 html 格式））大概是 cookie
            // 过期，我们清理缓存
            CACHE.invalidate(&(CacheEnum::Hdjw, stu_id.into())).await;
            continue;
        }
        let json = json.unwrap();
        // 典型的 cookie 失效时的 response body：
        // {"flag1":2,"msgContent":"è¯·å…ˆç™»å½•ç³»ç»Ÿ"}
        // 这里只判断 flag1 字段，因为 msgContent 是乱码，不好说
        if let Some(Value::Number(flag1)) = json.get("flag1")
            && flag1.as_i64() == Some(2)
        {
            CACHE.invalidate(&(CacheEnum::Hdjw, stu_id.into())).await;
            tried += 1;
            err_log.push_str(&format!(
                "({}) 教务系统 cookie 失效; stuid = {}",
                tried, stu_id
            ));
            continue;
        }
        data = json;
        break;
    }
    Ok(data)
}

/// 获取课表信息
pub async fn get_class_table(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Value, crate::Error> {
    let url = format!(
        "{}&xnxq01id={}-{}-{}",
        CLASS_TABLE_URL,
        xn,
        xn + 1,
        xq
    );
    let res = request_hdjw(&url, stu_id, RequestMethod::Get).await?;
    Ok(res)
}

/// 获取课程成绩
pub async fn get_grade(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Value, crate::Error> {
    let url = format!("{}&kksj={}-{}-{}", GRADE_URL, xn, xn + 1, xq);
    let res = request_hdjw(&url, stu_id, RequestMethod::Get).await?;
    Ok(res)
}

/// 获取成绩排名的内部通用方法，其他的方法都是基于此方法的wrapper包装。并且目前只查主修课程
///
/// `selection`: 学期列表，item格式为"xxxx-xxxx-x"，如"2023-2024-1"
///
/// `range`: 课程范围，目前有如下选择：
///
///  * 全部课程：01,02,03,04,05,06,07,08,09,10,11,12,13,14,15,16,17,18,88
///  * 必修课程：01,02,03,04,08,10,11,12,16
///  * 20 版核心课方案：08,12,16
///  * 24 版核心课方案: 03,16
///
/// 上述编号中，其中 07 表示马克思主义经典，09 表示科学与艺术经典，13 表示西方经典，14
/// 表示中国经典，18 表示其他，88 表示国际化，这些应该都属于选修课的一部分？
/// 10 表示实践环节，应该视为必修课的一部分？
///
/// `rank`: 排名方式，目前有如下选择：
///
///  * 绩点：3
///  * 加权平均：2
///  * 算数平均：4
///
/// 返回一个 tuple，其中第一个元素表示排名，第二个元素表示对应的成绩。
/// 如果请求成功但是解析数据时出现了问题，那么会返回一个 ("无数据", "无数据") 的元组。
#[inline]
pub async fn get_grade_rank_common(
    stu_id: &str,
    selection: &[String],
    range: String,
    rank: u8,
) -> Result<(String, String), crate::Error> {
    // 暂时不使用缓存
    // // 缓存数据到redis，减少对教务系统的压力，到这里了说明数据已经获取到了，可以直接缓存
    // let key = format!("grade_rank_{selection:?}_{range}_{rank}");
    // // 如果有缓存直接提前返回
    // if let Ok(res) = get_cookie_from_redis(&key, stu_id).await {
    //     let mut res = res.split(',');
    //     return Ok((
    //         res.next().unwrap().to_string(),
    //         res.next().unwrap().to_string(),
    //     ));
    // }
    let selection = selection.join(",");
    let url = format!(
        "{}&xnxq={}&kkxz={}&pmfs={}",
        GRADE_RANK_URL, selection, range, rank
    );
    println!("url: {}", url);
    // 随机等待一段时间，防止被教务系统drop掉连续的多次请求
    let wait_time = rand::thread_rng().gen_range(0..1000);
    tokio::time::sleep(Duration::from_millis(wait_time)).await;
    let res = request_hdjw(&url, stu_id, RequestMethod::Get).await?;
    // 尝试了一下失败重试方案，发现还是不行
    // let mut tried = 0;
    // loop {
    //     if tried >= 3 {
    //         return Err(anyhow!("请求成绩排名失败").into());
    //     }
    //     // 随机等待一段时间，防止被教务系统drop掉连续的多次请求
    //     let wait_time = rand::thread_rng().gen_range(0..1000);
    //     tokio::time::sleep(Duration::from_millis(wait_time)).await;
    //     let response = client
    //         .get(url.as_str())
    //         .headers(hdjw_headers.clone())
    //         .send()
    //         .await?;
    //     if response.status() == StatusCode::NOT_FOUND { // 404 说明请求过于频繁了
    //         tried += 1;
    //         continue;
    //     }
    //     // println!("获取成绩排名响应：{:#?}", res);
    //     res = response
    //         .error_for_status()?
    //         .json()
    //         .await?;
    //     break;
    // }
    let res = res["data"].as_array();
    if res.is_none() {
        return Ok(("无数据".to_string(), "无数据".to_string()));
    }
    let res = res.unwrap();
    if res.is_empty() {
        return Ok(("无数据".to_string(), "无数据".to_string()));
    }
    let res = res[0].as_object();
    if res.is_none() {
        return Ok(("无数据".to_string(), "无数据".to_string()));
    }
    let res = res.unwrap();
    let score = match rank {
        2 => res["pjxfj"].as_str().unwrap_or("无数据").to_string(),
        3 => res["pjxfjd"].as_str().unwrap_or("无数据").to_string(),
        4 => res["avgzcj"].as_str().unwrap_or("无数据").to_string(),
        _ => unreachable!(),
    };
    // 理论上这里的排名数据应该是数字，但是被湖大接口的类型搞怕了，字符串也接受
    let rank = if let Some(v) = res["numrow"].as_str() {
        v.to_string()
    } else if let Some(v) = res["numrow"].as_i64() {
        v.to_string()
    } else {
        "无数据".to_string()
    };
    // // 缓存时间也随机化，1h到3h，防止缓存雪崩
    // let cache_time = rand::thread_rng().gen_range(60 * 60..3 * 60 * 60);
    // add_cookie_to_redis(&key, &format!("{},{}", rank, score), stu_id, cache_time).await?;
    let res = (rank, score);
    Ok(res)
}

/// 获取考试安排
pub async fn get_exam_schedule(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Value, crate::Error> {
    //xnxqid=2025-2026-1
    let url = format!(
        "{}&xnxqid={}-{}-{}",
        EXAM_SCHEDULE_URL,
        xn,
        xn + 1,
        xq
    );
    dbg!(&url);
    let res = request_hdjw(&url, stu_id, RequestMethod::Get).await?;
    Ok(res)
}

/// 获取空教室信息
///
/// `jc`: 节次
pub async fn get_empty_classroom(
    stu_id: &str,
    xn: u16,
    xq: u8,
    week: &str,
    day: u8,
    jc: &str,
    build_id: &str,
) -> Result<Value, crate::Error> {
    let mut form_data = HashMap::new();
    let day = if day == 0 { 7 } else { day };
    form_data.insert("xnxqh", format!("{}-{}-{}", xn, xn + 1, xq));
    form_data.insert("jxlbh", build_id.to_string());
    form_data.insert("selectZc", week.to_string());
    form_data.insert("selectXq", day.to_string());
    form_data.insert("selectJc", jc.to_string());
    form_data.insert("typewhere", "jszq".to_string());
    let res = request_hdjw(
        EMPTY_CLASSROOM_URL,
        stu_id,
        RequestMethod::Post(form_data),
    )
    .await?;
    Ok(res)
}

/// 从可信电子凭证获取成绩
/// 返回成绩单 pdf 的文本内容，具体解析交给后端
pub async fn get_grade_from_ca(
    stu_id: &str,
) -> Result<String, crate::Error> {
    let ca_headers = ca_headers(stu_id).await?;
    let tempelate_url = if stu_id.starts_with('S')
        || stu_id.starts_with('B')
    {
        "https://ca.hnu.edu.cn/student/student/caTemplate/preview_file?templateId=99d8c1fcae15a7bf1a791f05cbc77f6f&isbzf=0&kcxz=&xfjd=&xzkc="
    } else {
        "https://ca.hnu.edu.cn/student/student/caTemplate/preview_file?templateId=02a70e11bc89b40dc2ef6ed14851ce25&isbzf=0&kcxz=&xfjd=&xzkc="
    };
    let res: Value = client
        .get(tempelate_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_headers.clone())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if res["code"] != 200 {
        return Err(anyhow!("获取文件失败").into());
    }
    let file_url = format!(
        "https://ca.hnu.edu.cn/student/sys/common/view/{}",
        res["message"].as_str().unwrap()
    );
    // 下载文件
    let res = client
        .get(&file_url)
        .timeout(Duration::from_secs(60))
        .headers(ca_headers)
        .send()
        .await?
        .error_for_status()?;
    let bytes = res.bytes().await?;
    let pdf = pdf_extract::extract_text_from_mem(&bytes).unwrap();
    Ok(pdf)
}

/// 获取课程具体的成绩详情
/// 返回的是 html 格式，需要交给后端来解析
pub async fn get_grade_detail(
    stu_id: &str,
    jx0404id: &str,
) -> Result<String, crate::Error> {
    let url = format!("{}{}", GRADE_DETAIL_URL, jx0404id);
    let res = request_hdjw(&url, stu_id, RequestMethod::Get).await?;
    if let Value::String(html) = res {
        Ok(html)
    } else {
        Err(anyhow!("获取课程成绩详情失败").into())
    }
}

/// 获取无课表课程
pub async fn get_class_table_extra(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Value, crate::Error> {
    let url = format!(
        "{}&xnxq01id={}-{}-{}",
        CLASS_TABLE_EXTRA,
        xn,
        xn + 1,
        xq
    );
    let res = request_hdjw(&url, stu_id, RequestMethod::Get).await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_get_class_table() {
        let res = get_class_table(&STU_ID, 2025, 1).await.unwrap();
        println!("{:}", res);
    }

    #[tokio::test]
    async fn test_get_grade() {
        let res = get_grade(&STU_ID, 2024, 2).await.unwrap();
        println!("{:#?}", res);
    }

    // #[tokio::test]
    // async fn test_grade_rank() {
    //     let start = tokio::time::Instant::now();
    //     let res = get_grade_rank_all(STU_ID.clone(), vec!["2024-2025-1".to_string()]).await;
    //     println!("全部课程{:#?}", res.unwrap());
    //     let res = get_grade_rank_must(STU_ID.clone(), vec!["2024-2025-1".to_string()]).await;
    //     println!("必修课程{:#?}", res.unwrap());
    //     let res = get_grade_rank_core(STU_ID.clone(), vec!["2024-2025-1".to_string()]).await;
    //     println!("核心课程{:#?}", res.unwrap());
    //     let duration = start.elapsed().as_millis();
    //     println!("总共耗时：{} ms", duration);
    // }

    #[tokio::test]
    async fn test_grade_rank() {
        let res = get_grade_rank_common(
            &STU_ID,
            &["2024-2025-1".to_string()],
            "01,02,03,04,05,06,07,08,09,10,11,12,13,14,15,16,17,18,88".to_string(),
            3,
        )
        .await;
        println!("绩点{:#?}", res.unwrap());
    }

    // #[tokio::test]
    // async fn test_xn_grade_rank() {
    //     let xn = 2024;
    //     let stuid = STU_ID.clone();
    //     let selections = vec![
    //         vec![
    //             format!("{}-{}-1", xn, xn + 1),
    //             format!("{}-{}-2", xn, xn + 1),
    //             format!("{}-{}-3", xn, xn + 1),
    //         ],
    //         vec![format!("{}-{}-1", xn, xn + 1)],
    //         vec![format!("{}-{}-2", xn, xn + 1)],
    //         vec![format!("{}-{}-3", xn, xn + 1)],
    //     ];
    //     let mut res = Vec::new();
    //     for selection in selections {
    //         res.push(crate::handlers::hdjw::get_all_grade_rank(stuid.clone(), selection).await);
    //     }
    //     // let res = join_all(tasks).await;
    //     let res = json!({
    //         "all": res[0].as_ref().map_err(|e| anyhow!("请求成绩排名失败：{}", e)).unwrap(),
    //         "autumn": res[1].as_ref().map_err(|e| anyhow!("请求成绩排名失败：{}", e)).unwrap(),
    //         "spring": res[2].as_ref().map_err(|e| anyhow!("请求成绩排名失败：{}", e)).unwrap(),
    //         "summer": res[3].as_ref().map_err(|e| anyhow!("请求成绩排名失败：{}", e)).unwrap(),
    //     });
    //     println!("{:#?}", res);
    // }

    #[tokio::test]
    async fn test_exam_schedule() {
        let res = get_exam_schedule(&STU_ID, 2025, 1).await;
        println!("考试信息{:#?}", res.unwrap());
    }

    #[tokio::test]
    async fn test_empty_classroom() {
        let res = get_empty_classroom(
            &STU_ID, 2024, 3, "16", 1, "0102", "106",
        )
        .await;
        println!("空教室{:#?}", res.unwrap());
    }

    #[tokio::test]
    async fn test_get_grade_from_ca() {
        let res = get_grade_from_ca(&STU_ID).await;
        println!("{:#?}", res.unwrap());
    }

    #[tokio::test]
    async fn test_get_grade_detail() {
        let res = get_grade_detail(&STU_ID, "aaa").await;
        println!("{:#?}", res.unwrap());
    }

    #[tokio::test]
    async fn test_get_class_table_extra() {
        let res = get_class_table_extra(&STU_ID, 2025, 1).await;
        println!("{:#?}", res.unwrap());
    }
}
