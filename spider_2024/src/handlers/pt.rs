use anyhow::anyhow;
use serde_json::Value;

use crate::{
    dtos::pt::{
        CardHistoryReq, CardHistoryRes, CardHistoryReturn,
        CardInfoRes, CasPasswordStatus, UnreadEmailRes,
    },
    spiders::{self},
};

pub async fn check_password_handler(
    stu_id: &str,
    pt_password: &str,
) -> Result<CasPasswordStatus, crate::Error> {
    let result =
        spiders::pt::check_password_with_cas(stu_id, pt_password)
            .await?;

    Ok(result)
}

pub async fn get_unread_email_handler(
    stu_id: &str,
) -> Result<UnreadEmailRes, crate::Error> {
    let mut res = spiders::pt::get_unread_email(stu_id).await?;

    let res: UnreadEmailRes =
        serde_json::from_value(res["data"].take())
            .map_err(|e| anyhow!("获取未读邮件数据失败 {}", e))?;

    Ok(res)
}

pub async fn get_card_info_handler(
    stu_id: &str,
) -> Result<CardInfoRes, crate::Error> {
    let mut res = spiders::pt::get_card_info(stu_id).await?;
    let res: CardInfoRes = serde_json::from_value(res["data"].take())
        .map_err(|e| anyhow!("获取校园卡余额信息失败 {}", e))?;
    Ok(res)
}

/// 返回消费/充值记录，需要注意消费总额默认是负数
pub async fn get_card_history_handler(
    req: CardHistoryReq,
) -> Result<CardHistoryReturn, crate::Error> {
    let mut res = spiders::pt::get_card_history(
        &req.stu_id,
        &req.year,
        &req.month,
        &req.typ,
    )
    .await?;
    if res["data"].is_null() || res["data"]["amt"].is_null() {
        return Err(anyhow!("校园卡交易记录获取失败").into());
    }

    let data: CardHistoryRes =
        serde_json::from_value(res["data"].take())
            .map_err(|e| anyhow!("校园卡交易记录解析失败 {e}"))?;

    // 把原始数据传给前端，不处理正负，由前端负责决定要不要取反
    let return_value = CardHistoryReturn {
        total: data.amt / 100.0,
        TranCount: data.count,
        items: data.webTrjnDTO.unwrap_or_default(),
    };

    Ok(return_value)
}

#[deprecated(
    note = "这个pt个人门户的个人信息接口不可以使用，要用hdjw的个人信息接口"
)] // 警告下防止误用
pub async fn get_user_info_handler(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let res = spiders::pt::get_user_info(stu_id).await?;
    Ok(res)
}
