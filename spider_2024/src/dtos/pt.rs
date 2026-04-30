use serde::Deserialize;

/// 个人门户密码验证结果
#[derive(Debug)]
pub enum CasPasswordStatus {
    /// 密码正确
    Success,
    /// 密码错误
    Fail,
    /// 需要更换密码
    ShouldChange,
    /// 账号被锁定
    Lock,
}

#[derive(Deserialize, Debug)]
pub struct CardHistoryReq {
    pub stu_id: String,
    pub year: String,
    pub month: String,
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct UnreadEmailRes {
    pub unReadCount: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct CardInfoRes {
    pub account: u32,
    pub balance: String,
}

/// 由代码推断的校园卡交易历史接口返回值
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistoryRes {
    /// 总额
    pub amt: f64,
    /// 交易数量，见下 CardHistoryReturn.TranCount
    pub count: f64,
    /// 交易项列表
    pub webTrjnDTO: Option<Vec<CardHistoryItem>>,
}

/// 一卡通历史账单
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistoryReturn {
    pub TranCount: f64,
    pub total: f64,
    pub items: Vec<CardHistoryItem>,
}
#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct CardHistoryItem {
    pub fTranAmt: String,
    pub effectdate: String,
    pub jndatetime: String,
    pub jourName: String,
    pub usedcardnum: u32,
    pub nowAmt: String,
    pub sysname1: Option<String>,
    pub tranname: String,
}
