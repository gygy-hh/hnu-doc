use crate::{dtos::xgxt::PersonInfo, spiders};

// TODO：学工个人信息接口数据量大
pub async fn get_person_info_handler(
    stu_id: &str,
) -> Result<PersonInfo, crate::Error> {
    let res = spiders::xgxt::get_person_info(stu_id).await?;
    Ok(res)
}
