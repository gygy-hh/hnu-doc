use anyhow::anyhow;

use crate::{
    dtos::lab::{
        LabArrangeItem, LabCourseItem, LabLoginInfoRes,
        LabScoreDetailItem, LabScoreItem, LabScoreStructureItem,
        LabSemInfoRes, VirtualLabGradeItem,
    },
    spiders,
};

pub async fn get_lab_list_handler(
    stu_id: &str,
) -> Result<Option<Vec<LabArrangeItem>>, crate::Error> {
    let mut res = spiders::lab::get_lab_list(stu_id).await?;
    if res.is_null() {
        return Ok(None);
    }

    if !res["rows"].is_array() {
        return Err(anyhow!("意料之外的数据 {res}").into());
    }

    let rows: Vec<LabArrangeItem> =
        serde_json::from_value(res["rows"].take())
            .map_err(|e| anyhow!("解析数据失败 {e}"))?;

    Ok(Some(rows))
}

pub async fn check_lab_password_handler(
    stu_id: &str,
    password: &str,
) -> Result<LabLoginInfoRes, crate::Error> {
    let (res, _) =
        spiders::lab::check_password(stu_id, password).await?;
    Ok(serde_json::from_value(res)?)
}

pub async fn get_lab_sem_info_handler(
    stu_id: &str,
) -> Result<Option<Vec<LabSemInfoRes>>, crate::Error> {
    let res = spiders::lab::get_sem_info(stu_id).await?;
    Ok(serde_json::from_value(res)?)
}

pub async fn get_lab_course_list_handler(
    stu_id: &str,
    sem: &str,
) -> Result<Option<Vec<LabCourseItem>>, crate::Error> {
    let mut res = spiders::lab::get_course_list(stu_id, sem).await?;
    if res.is_null() {
        return Ok(None);
    }

    if !res["rows"].is_array() {
        return Err(anyhow!("意料之外的数据 {res}").into());
    }

    let rows: Vec<LabCourseItem> =
        serde_json::from_value(res["rows"].take())
            .map_err(|e| anyhow!("解析数据失败 {e}"))?;

    Ok(Some(rows))
}

pub async fn get_lab_score_handler(
    stu_id: &str,
    course_id: &str,
    sem: &str,
) -> Result<Option<Vec<LabScoreItem>>, crate::Error> {
    let mut res =
        spiders::lab::get_lab_score(stu_id, sem, course_id).await?;

    if res.is_null() {
        return Ok(None);
    }

    if !res["rows"].is_array() {
        return Err(anyhow!("意料之外的数据 {res}").into());
    }

    let rows: Vec<LabScoreItem> =
        serde_json::from_value(res["rows"].take())
            .map_err(|e| anyhow!("解析数据失败 {e}"))?;

    Ok(Some(rows))
}

pub async fn get_virtual_lab_score_handler(
    stu_id: &str,
) -> Result<Option<Vec<VirtualLabGradeItem>>, crate::Error> {
    let mut res = spiders::lab::get_virtual_lab_score(stu_id).await?;
    if res.is_null() {
        return Ok(None);
    }

    if !res["rows"].is_array() {
        return Err(
            anyhow!("意料之外的数据 (rows 不是数组) {res}").into()
        );
    }

    let rows: Vec<VirtualLabGradeItem> =
        serde_json::from_value(res["rows"].take())
            .map_err(|e| anyhow!("解析数据失败 {e}"))?;

    Ok(Some(rows))
}

pub async fn get_lab_score_structure_handler(
    stu_id: &str,
    course_id: &str,
) -> Result<Option<Vec<LabScoreStructureItem>>, crate::Error> {
    let mut res =
        spiders::lab::get_score_structure(stu_id, course_id).await?;
    if res.is_null() {
        return Ok(None);
    }

    if !res["Data"].is_array() {
        return Err(
            anyhow!("意料之外的数据 (Data 不是数组) {res}").into()
        );
    }

    let data: Vec<LabScoreStructureItem> =
        serde_json::from_value(res["Data"].take())?;

    Ok(Some(data))
}

pub async fn get_lab_score_detail_handler(
    stu_id: &str,
    course_id: &str,
) -> Result<Option<Vec<LabScoreDetailItem>>, crate::Error> {
    let mut res =
        spiders::lab::get_score_detail(stu_id, course_id).await?;
    if res.is_null() {
        return Ok(None);
    }

    if !res["Data"].is_object() {
        return Err(
            anyhow!("意料之外的数据 (Data 不是对象) {res}").into()
        );
    }

    let mut data = res["Data"].take();

    if !data["Lablist"].is_array() {
        return Err(anyhow!(
            "意料之外的数据 (Lablist 不是数组) {data}"
        )
        .into());
    }

    let result: Vec<LabScoreDetailItem> =
        serde_json::from_value(data["Lablist"].take())?;

    Ok(Some(result))
}
