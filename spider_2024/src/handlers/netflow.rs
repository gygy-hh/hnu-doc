use anyhow::anyhow;

use crate::{
    dtos::netflow::{
        NetflowDayDetailReq, NetflowDetailRes, NetflowMonthDetailReq,
        NetflowOrderRes, NetflowOrderReturnItem, NetflowPayInfoRes,
        NetflowThisMonthRes, NetflowUnlockStatusRes,
    },
    spiders,
};

pub async fn get_netflow_handler(
    stu_id: &str,
) -> Result<NetflowThisMonthRes, crate::Error> {
    let mut res = spiders::netflow::get_netflow(stu_id).await?;

    if !&res["data"].is_object() {
        return Err(anyhow!("netflow data不是对象").into());
    }

    let mut data: NetflowThisMonthRes =
        serde_json::from_value(res["data"].take())
            .map_err(|e| anyhow!("netflow data解析失败 {e}"))?;

    // 给流量数据加上单位
    // 流量字符串可能返回"小于0.01GB"，此时不重复添加单位

    fn try_add_suffix(s: &mut String, suffix: &str) {
        if !s.ends_with(suffix) {
            *s += suffix;
        }
    }

    fn try_add_gb_to_str_value(place: &mut String) {
        let mut orig = place.clone();
        try_add_suffix(&mut orig, "GB");
        *place = orig;
    }

    try_add_gb_to_str_value(&mut data.downloadTraffic);
    try_add_gb_to_str_value(&mut data.uploadTraffic);
    try_add_gb_to_str_value(&mut data.allTraffic);

    Ok(data)
}

pub async fn get_netflow_pay_info_handler(
    stu_id: &str,
) -> Result<NetflowPayInfoRes, crate::Error> {
    let mut res = spiders::netflow::get_pay_status(stu_id).await?;

    let data: NetflowPayInfoRes =
        serde_json::from_value(res["data"].take())
            .map_err(|e| anyhow!("netflow pay info 解析失败 {e}"))?;

    Ok(data)
}

pub async fn get_unlock_status_handler(
    stu_id: &str,
) -> Result<NetflowUnlockStatusRes, crate::Error> {
    let res = spiders::netflow::get_user_status(stu_id).await?;

    let is_locked = &res["data"]["IsLocked"];
    if !is_locked.is_i64() {
        return Err(
            anyhow!("校园网锁定状态不是 i64: `{is_locked}`").into()
        );
    }

    let is_locked = is_locked.as_i64().unwrap();
    let status = match is_locked {
        0 => "未锁定",
        1 => "已锁定",
        _ => "未知",
    };

    Ok(NetflowUnlockStatusRes {
        status: status.to_string(),
    })
}

pub async fn get_netflow_month_detail_handler(
    req: NetflowMonthDetailReq,
) -> Result<NetflowDetailRes, crate::Error> {
    let mut res = spiders::netflow::get_netflow_month_detail(
        &req.stu_id,
        &req.year,
        &req.month,
    )
    .await?;

    let data: NetflowDetailRes =
        serde_json::from_value(res["data"].take()).map_err(|e| {
            anyhow!("netflow month detail 解析失败 {e}")
        })?;

    Ok(data)
}

pub async fn get_netflow_day_detail_handler(
    req: NetflowDayDetailReq,
) -> Result<NetflowDetailRes, crate::Error> {
    let mut res = spiders::netflow::get_netflow_day_detail(
        &req.stu_id,
        &req.year,
        &req.month,
        &req.day,
    )
    .await?;

    let data: NetflowDetailRes =
        serde_json::from_value(res["data"].take()).map_err(|e| {
            anyhow!("netflow day detail 解析失败 {e}")
        })?;

    Ok(data)
}

pub async fn get_netflow_order_handler(
    stu_id: &str,
) -> Result<Vec<NetflowOrderReturnItem>, crate::Error> {
    let mut res = spiders::netflow::get_order(stu_id).await?;

    if !res["data"].is_array() {
        return Err(anyhow!("netflow order data不是数组").into());
    }

    let res: Vec<NetflowOrderRes> =
        serde_json::from_value(res["data"].take())
            .map_err(|e| anyhow!("netflow order data解析失败 {e}"))?;

    let return_values = res
        .iter()
        .map(|item| {
            let upload = item.Upload.unwrap_or_default();
            let upload_name = if upload == 0.0 {
                "0 GB".to_string()
            } else {
                format!("{:.2} GB", upload / 1024.0 / 1024.0 / 1024.0)
            };

            let download = item.Download.unwrap_or_default();
            let download_name = if download == 0.0 {
                "0 GB".to_string()
            } else {
                format!(
                    "{:.2} GB",
                    download / 1024.0 / 1024.0 / 1024.0
                )
            };

            NetflowOrderReturnItem {
                DownloadName: download_name,
                UploadName: upload_name,
                Download: item.Download,
                Upload: item.Upload,
                Month: item.Month.clone(),
                RealOverTraffic: item.RealOverTraffic,
                ShouldPay: item.ShouldPay,
                UpdateTime: item.UpdateTime.clone(),
            }
        })
        .collect();

    Ok(return_values)
}
