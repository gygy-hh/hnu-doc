use crate::utils::client;
use anyhow::anyhow;
use serde_json::Value;

const QUERY_URL: &str =
    "http://wxpay.hnu.edu.cn/api/appElectricCharge/checkRoomNo";

pub async fn get_electricity(
    park: u8,
    building: String,
    room: String,
) -> Result<String, crate::Error> {
    let res = client
        .get(format!(
            "{}?parkNo={}&buildingNo={}&rechargeType=2&roomNo={}",
            QUERY_URL, park, building, room
        ))
        .header(
            "referer",
            "http://wxpay.hnu.edu.cn/electricCharge/home/",
        )
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(res
        .get("data")
        .ok_or(anyhow!("请求发生异常"))?
        .get("Balance")
        .ok_or(anyhow!("Balance字段缺失"))?
        .as_str()
        .ok_or(anyhow!("Balance字段异常"))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use crate::spiders::electricity::get_electricity;

    #[tokio::test]
    async fn test_get_electricity() {
        dbg!(
            get_electricity(1, "20".to_string(), "201".to_string())
                .await
                .unwrap()
        );
    }
}
