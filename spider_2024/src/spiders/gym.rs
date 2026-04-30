use crate::{
    spiders::login::{gym_headers, gym_headers_from_cas},
    utils::{client, request::CacheChecker},
};
use serde_json::Value;

/// 这个URL能返回体测各个项目的数据（例如跳远多少米等），
/// 但不能得到每个项目对总分的贡献
///
/// see also [`RAW_DATA_URL`]
const DATA_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getStudentScore";
/// 这个URL在湖大系统前端仅用于获取视力信息
///
/// 然而我们发现这个接口也能返回其他体测项目的*得分*
///
/// see also [`DATA_URL`]
const RAW_DATA_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Report/getEyeDetails";
const APPOINT_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getStudentClass";
const DETAIL_URL: &str = "http://gymos.hnu.edu.cn/bdlp_api_fitness_test_student_h5/public/index.php/index/Appoint/getSchoolFitClassDetail";

/// 获取体测数据，不含每个项目的得分
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

/// 获取体测原始数据，含每个项目的得分
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

/// 获取体测预约数据
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

/// 获取体测预约详情
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
