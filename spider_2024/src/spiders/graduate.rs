//! 注意，研究生是用不了体测系统和本科生教务系统爬虫的

use std::{collections::HashMap, sync::LazyLock};

use anyhow::anyhow;
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    spiders::login::graduate_headers_and_id,
    utils::crypto::graduate_decrypt,
};

const GRADUATE_HOST_URL: &str = "http://yjsxt.hnu.edu.cn/gmis/"; // 注意末尾一定带上斜杠
const CLASS_TABLE_URL: &str = "/student/pygl/py_kbcx_ew"; // 注意开头一定要带斜杠

// static CLASS_ID_AFFIXS: Lazy<Vec<&str>> = Lazy::new(||{vec!["6️⃣", "👍", "💯", "🛜", "😀",
// "📶"]});
static START_MAP: LazyLock<HashMap<u8, &str>> = LazyLock::new(|| {
    [
        (1, "8:00"),
        (2, "8:55"),
        (3, "10:00"),
        (4, "10:55"),
        (5, "14:30"),
        (6, "15:15"),
        (7, "16:10"),
        (8, "16:55"),
        (9, "19:00"),
        (10, "19:55"),
        (11, "20:50"),
        (12, "21:35"),
    ]
    .into_iter()
    .collect()
});
static END_MAP: LazyLock<HashMap<u8, &str>> = LazyLock::new(|| {
    [
        (1, "8:45"),
        (2, "9:40"),
        (3, "10:45"),
        (4, "11:40"),
        (5, "15:15"),
        (6, "16:00"),
        (7, "16:55"),
        (8, "17:40"),
        (9, "19:45"),
        (10, "20:40"),
        (11, "21:35"),
        (12, "22:20"),
    ]
    .into_iter()
    .collect()
});

/// 用来兼容本科生的爬虫返回数据
#[derive(Serialize, Debug)]
struct Course {
    /// 开始时间
    djjssj: String,
    /// 结束时间
    djkssj: String,
    /// 课程地点
    js_name: String,
    /// 课程名称
    kc_name: String,
    /// 课程id  
    ktmc_name: String,
    /// 周次
    pkzcmx: String,
    /// 节次
    jczy01501ids: String,
    /// 教师名称
    teachernames: String,
    /// 星期几
    pksj: String,
    /// 课程id
    id: String,
    /// 不知道干嘛的，目前一直是空
    skqk: String,
    // 无用，留空
    jczy013id: String,
}

#[derive(Debug)]
struct CourseInfo {
    course_id: String,
    course_name: String,
    #[expect(dead_code)]
    class_name: String,
    class_time: String,
    teacher: String,
    classroom: String,
}

pub(crate) async fn get_class_table(
    stu_id: &str,
    xn: u16,
    xq: u8,
) -> Result<Value, crate::Error> {
    let new_client = reqwest::Client::builder().no_proxy().build()?;
    let (graduate_headers, id) =
        graduate_headers_and_id(stu_id).await?;
    // 目前只支持秋季学期，春季学期，和夏季小学期，寒假小学期不支持
    let xq = match xq {
        1 => 1,
        2 => 3,
        3 => 4,
        _ => return Err(anyhow!("Invalid xq").into()),
    };
    let term_code = (xn - 2022) * 4 + xq as u16 + 47;
    let url =
        format!("{}{}{}", GRADUATE_HOST_URL, id, CLASS_TABLE_URL);
    let res = new_client
        .post(&url)
        .headers(graduate_headers)
        .form(&[("kblx", "xs"), ("termcode", &term_code.to_string())])
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let res =
        serde_json::from_str::<Value>(&graduate_decrypt(&res)?)?;

    let mut courses: Vec<Course> = Vec::new();
    // 外层循环为节次，内层循环为星期几
    for item in
        res["rows"].as_array().ok_or(anyhow!("Invalid data"))?
    {
        if item["mc"] == json!("无节次") {
            continue;
        }
        let jc = item["mc"]
            .as_str()
            .ok_or(anyhow!("Invalid data"))?
            .parse::<u8>()?;
        let jczy01501id = format!("{:0>2}", jc);
        let pksj = 1..=7;
        for day in pksj {
            let key = format!("z{}", day);
            if item[&key] == Value::Null {
                continue;
            }
            let z =
                item[&key].as_str().ok_or(anyhow!("Invalid data"))?;
            let course_info = parse_course_info(z)
                .ok_or(anyhow!("Invalid data"))?;
            let course = Course {
                djjssj: END_MAP[&jc].to_string(),
                djkssj: START_MAP[&jc].to_string(),
                js_name: course_info.classroom.clone(),
                kc_name: course_info.course_name.clone(),
                ktmc_name: course_info.course_id.clone(),
                pkzcmx: course_info.class_time.clone(),
                jczy01501ids: jczy01501id.clone(),
                teachernames: course_info.teacher.clone(),
                pksj: day.to_string(),
                id: course_info.course_id.clone(),
                skqk: "".to_string(),
                jczy013id: "".to_string(),
            };
            // 与上一个时间段的校验，看看是否要合并
            let mut flag = false;
            for item in courses.iter_mut() {
                if item.kc_name == course.kc_name
                    && item.pkzcmx == course.pkzcmx
                    && item.teachernames == course.teachernames
                    && item.pksj == course.pksj
                {
                    // 用逗号去jczy01501ids，得出目前已有的节次，然后看看可不可以往上加
                    let jczy01501ids = item
                        .jczy01501ids
                        .split(',')
                        .map(|x| x.parse::<u8>().unwrap())
                        .collect::<Vec<u8>>();
                    // 如果存在一个比当前节次小1的，那么就合并
                    if jczy01501ids.contains(&(jc - 1)) {
                        // 修改现有item中的jczy01501ids
                        let new_jczy01501ids = format!(
                            "{},{}",
                            item.jczy01501ids, jczy01501id
                        );
                        item.jczy01501ids = new_jczy01501ids;
                        // 修改现有item中的djjssj
                        item.djjssj = END_MAP[&jc].to_string();
                        flag = true;
                        break;
                    }
                }
            }
            if !flag {
                courses.push(course);
            }
        }
    }
    // 如果courses中存在course的id相同的两项，则加上一个后缀，后缀为相同的第几个项
    let mut id_count: HashMap<String, usize> = HashMap::new();

    for course in courses.iter_mut() {
        let count = id_count.entry(course.id.clone()).or_insert(0);
        *count += 1;

        if *count > 1 {
            course.id = format!("{}_{}", course.id, count);
        }
    }
    Ok(json!(courses))
}

fn parse_course_info(input: &str) -> Option<CourseInfo> {
    // 用 <br/> 进行分割
    let parts: Vec<&str> =
        input.split("<br/>").filter(|s| !s.is_empty()).collect();

    // 提取各个部分的数据
    let course_id =
        parts[0].replace("课程编号:", "").trim().to_string();
    let course_name =
        parts[1].replace("课程名称:", "").trim().to_string();
    let class_name = parts[2].replace("班级:", "").trim().to_string();
    let class_time =
        parts[4].replace("上课时间:", "").trim().to_string();
    // class_time再处理，原为此数据：[9-16周] 连续周
    // 生成类型于1,3,5,7,9,11,13,15这样的数据
    // 取得“-”左右的两个数字，然后生成区间，然后用逗号join
    let class_time = class_time
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect::<String>();
    let class_time = class_time.split('-').collect::<Vec<&str>>();
    let class_time = (class_time[0].parse::<u8>().unwrap()
        ..=class_time[1].parse::<u8>().unwrap())
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",");
    // 老师和教室信息在最后一行
    let teacher_and_classroom = parts[5].trim();

    // 使用 chars() 来遍历字符，确保正确处理 UTF-8 字符串
    let teacher_end = teacher_and_classroom
        .chars()
        .position(|c| c == '[')
        .unwrap();
    let teacher: String =
        teacher_and_classroom.chars().take(teacher_end).collect();

    let classroom_start = teacher_end + 1; // 跳过 [
    let classroom_end = teacher_and_classroom
        .chars()
        .position(|c| c == ']')
        .unwrap();
    let classroom: String = teacher_and_classroom
        .chars()
        .skip(classroom_start)
        .take(classroom_end - classroom_start)
        .collect();

    Some(CourseInfo {
        course_id,
        course_name,
        class_name,
        class_time,
        teacher,
        classroom,
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_get_class_table() {
        let res = get_class_table(&STU_ID, 2024, 1).await.unwrap();
        println!("{}", res);
    }
}
