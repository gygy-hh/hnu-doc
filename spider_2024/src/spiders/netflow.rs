use crate::{spiders::login::netflow_headers, utils::client};
use serde_json::Value;

const NETFLOW_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/history/gettrafficinfobythismonth";
const NETFLOW_MONTH_URL: &str = "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbymonth?month=";
const NETFLOW_DAY_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/history/getfloatdetailbyday?day=";
const NETFLOW_USER_INFO_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/account/getuserinfo";
const NETFLOW_PAY_INFO_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/pay/getpayinfo";
const NETFLOW_ORDER_URL: &str =
    "http://ll.hnu.edu.cn/api/v1/historyorder/getpagedlist";
// 实测X-TOKEN不要也能正常请求
// const X_TOKEN: &str =
// "ABED4D22E7983976C642827C515F4696192DAECD2A82543E22093E7886F293FFB2840CAEFCBA4F394087F3DC470B471B83189F2808A8648B87592D34F67D976B4EBEA27C4BF451FA0D79D9ACDA53C287800ECA25CD24D457177246E5B409342DD4EEBB7FFB84F0DA09A1723FBFCEE0CFDFDB1166461746CE846AC529885B0217D5EFDA7C47A51B792B7AA2D2C1B563868694C7C74AFB6DC6827B426F6CBD71391576B826F6C2397F8FB97EDB8508754AA2F6C3C61284EFEBB0999535FFC9F2B9152D95C727CC8C465C5D82A505FABF8C93ACB48154D212F51E3BEDA1A1834583EDF2F538B7039C2E4CD646578F521E77CB50356277821ABA46D28242FFD851CEB06A02939319EEE1E06E0B6540DF39C4153AB217164E6EB76021C8E70217BE65D6BF26B6E98DD8E12BAEEE1589B00CCE8E93B28F506CC4FECD06B12B8B89B5F57E823DB874D56276817AF1C31074C7F501BB0CE59684C1A863591DEE61993552E13A9FACCF0B75FD7AD17AAA6E3B9A4D8213CEFD1FD8D1C91E7222C67991860D1ACB35FF33A8E55380C1CA19B63AA527405EFF920AA5CD166BBA8AE87C2B5BBB27D87A71861A1C0FF05282E78271C4C24674E86693064469E801CB61E2C9324C2C6924953A87D99D0EAC700D3C1C7F971DCEA07AA1271E7A1384D9E1C802EB2D3F3AB96BD0FD134D9F4ABE5ADBEC71AE1853F01D8BF32FC5B2D12F3C07B535E2A1B0E2597A98826D707811F489A9ABD23BA271BC4E19013AD31FC2FE78A4A36DB4E0D629D339EA88AE01A88BD99C99751A01E89AD8494090F75291B7FD1C863F905E657CA3215834CC6051F3CA381AE9CA70AC1570EB96FF41BE719FE214A5A2DA4D86598F2DBDE76A697796CCC25D3E486D3731FE3803AF896B3AB44DFB9B55B7FC0E951509A81EA972B253957DD3C8D6CC4573518D2F55E71E0C6A1699B9250AE83F33AE8318F4D3FFDF240BF5AFDE23C517179535386A49E060B2911138D4D02E34A865E8F3D89B52D618F46A3201B1A4F15D1183CDDA5D5701EE129DD9DA487B66147A5A60EE50F401E63A2FA31D"
// ;

// 网费：当月用量
pub async fn get_netflow(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let res = client
        .get(NETFLOW_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}

// 月明细
pub async fn get_netflow_month_detail(
    stu_id: &str,
    year: &str,
    month: &str,
) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let url = format!("{NETFLOW_MONTH_URL}{}-{:0>2}", year, month);
    let res = client
        .get(url)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}

// 日明细
pub async fn get_netflow_day_detail(
    stu_id: &str,
    year: &str,
    month: &str,
    day: &str,
) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let url =
        format!("{NETFLOW_DAY_URL}{}{:0>2}{:0>2}", year, month, day);
    let res = client
        .get(url)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}

// 用户信息
pub async fn get_user_status(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let res = client
        .get(NETFLOW_USER_INFO_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}

// 支付记录
pub async fn get_pay_status(
    stu_id: &str,
) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let res = client
        .get(NETFLOW_PAY_INFO_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}

// 订单
pub async fn get_order(stu_id: &str) -> Result<Value, crate::Error> {
    let netflow_headers = netflow_headers(stu_id).await?;
    let res = client
        .get(NETFLOW_ORDER_URL)
        .headers(netflow_headers)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::request::STU_ID;

    #[tokio::test]
    async fn test_get_netflow() {
        let res = get_netflow(&STU_ID).await.unwrap();
        dbg!(res);
    }

    #[tokio::test]
    async fn test_get_netflow_month() {
        let res = get_netflow_month_detail(&STU_ID, "2024", "10")
            .await
            .unwrap();
        dbg!(res);
    }

    #[tokio::test]
    async fn test_get_netflow_day() {
        let res = get_netflow_day_detail(&STU_ID, "2024", "10", "7")
            .await
            .unwrap();
        dbg!(res);
    }

    #[tokio::test]
    async fn test_get_user_status() {
        let res = get_user_status(&STU_ID).await.unwrap();
        dbg!(res);
    }

    #[tokio::test]
    async fn test_get_pay_status() {
        let res = get_pay_status(&STU_ID).await.unwrap();
        dbg!(res);
    }

    #[tokio::test]
    async fn test_get_order() {
        let res = get_order(&STU_ID).await.unwrap();
        dbg!(res);
    }
}
