use crate::{
    spiders::login::{gym_headers, gym_headers_from_cas},
    utils::{client, request::CacheChecker},
};
use serde_json::Value;

// DATA_URL：分项数值；RAW_DATA_URL：分项得分
const DATA_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getStudentScore";
// RAW_DATA_URL：视力接口也可拿各项得分
const RAW_DATA_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getEyeDetails";
const APPOINT_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getStudentClass";
const DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getSchoolFitClassDetail";

// 体测分项数据（无单项得分）
pub(crate) async fn get_data(
    stu_id: &str,
    xn: u16,
) -> Result<Value, crate::Error> {
    let gym_headers =
        if let Ok(direct_login) = gym_headers(stu_id).await {
            direct_login
        } else {
            gym_headers_from_cas(stu_id).await?
        };
    let res = client
        .post(DATA_URL)
        .form(&[("year_num", xn)])
        .headers(gym_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?
        .check_gym(stu_id)
        .await;
    Ok(res)
}

// 原始数据含得分
pub(crate) async fn get_raw_data(
    stu_id: &str,
    xn: u16,
) -> Result<Value, crate::Error> {
    let gym_headers =
        if let Ok(direct_login) = gym_headers(stu_id).await {
            direct_login
        } else {
            gym_headers_from_cas(stu_id).await?
        };
    let res = client
        .post(RAW_DATA_URL)
        .form(&[("year_num", xn)])
        .headers(gym_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?
        .check_gym(stu_id)
        .await;
    Ok(res)
}

// 预约列表
pub(crate) async fn get_appoint(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let gym_headers =
        if let Ok(direct_login) = gym_headers(stu_id).await {
            direct_login
        } else {
            gym_headers_from_cas(stu_id).await?
        };
    let res = client
        .post(APPOINT_URL)
        .headers(gym_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?
        .check_gym(stu_id)
        .await;
    Ok(res)
}

// 预约详情
pub(crate) async fn get_appoint_detail(
    stu_id: &str,
    class_id: &str,
    class_time: &str,
    test_time: &str,
) -> Result<Value, crate::Error> {
    let gym_headers =
        if let Ok(direct_login) = gym_headers(stu_id).await {
            direct_login
        } else {
            gym_headers_from_cas(stu_id).await?
        };
    let res = client
        .post(DETAIL_URL)
        .form(&[
            ("class_id", class_id),
            ("class_time", class_time),
            ("test_time", test_time),
        ])
        .headers(gym_headers)
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?
        .check_gym(stu_id)
        .await;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_get_data() {
        let res = get_data(&STU_ID, 2025).await.unwrap();
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_get_raw_data() {
        let res = get_raw_data(&STU_ID, 2024).await.unwrap();
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_get_appoint() {
        let res = get_appoint(&STU_ID).await.unwrap();
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_get_appoint_detail() {
        let res = get_appoint_detail(
            &STU_ID,
            "152",
            "2025-12-15",
            "10:00 - 11:30",
        )
        .await
        .unwrap();
        println!("{:?}", res);
    }
}
