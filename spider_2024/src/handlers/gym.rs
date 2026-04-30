use crate::{
    dtos::gym::{
        FitnessAppointDetailRes, FitnessAppointRes, FitnessRawRes,
        FitnessRes, GymReq,
    },
    spiders,
};
use anyhow::anyhow;

pub async fn get_gym_grade_handler(
    req: GymReq,
) -> Result<FitnessRes, crate::Error> {
    let mut res = spiders::gym::get_data(&req.stu_id, req.xn).await?;

    let res_data = &res["data"];
    if !res_data.is_object() {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }

    let res_data: FitnessRes =
        serde_json::from_value(res["data"].take())?;

    Ok(res_data)
}

pub async fn get_gym_raw_grade_handler(
    req: GymReq,
) -> Result<FitnessRawRes, crate::Error> {
    let mut res =
        spiders::gym::get_raw_data(&req.stu_id, req.xn).await?;

    let res_data = &res["data"];
    if !res_data.is_object() {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }

    let res_data: FitnessRawRes =
        serde_json::from_value(res["data"].take())?;

    Ok(res_data)
}

pub async fn get_gym_appoint_handler(
    stu_id: &str,
) -> Result<Vec<FitnessAppointRes>, crate::Error> {
    let mut res = spiders::gym::get_appoint(stu_id).await?;
    let res_data = &res["data"];
    if !res_data.is_array() {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }

    let res_data: Vec<FitnessAppointRes> =
        serde_json::from_value(res["data"].take())?;
    Ok(res_data)
}

pub async fn get_gym_appoint_detail_handler(
    stu_id: &str,
    class_id: &str,
    class_time: &str,
    test_time: &str,
) -> Result<FitnessAppointDetailRes, crate::Error> {
    let mut res = spiders::gym::get_appoint_detail(
        stu_id, class_id, class_time, test_time,
    )
    .await?;

    let res_data = &res["data"];
    if !res_data.is_object() {
        return Err(anyhow!("意料之外的体测平台数据：{res}").into());
    }

    let res_data: FitnessAppointDetailRes =
        serde_json::from_value(res["data"].take())?;
    Ok(res_data)
}
