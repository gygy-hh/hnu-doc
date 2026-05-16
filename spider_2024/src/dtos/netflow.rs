use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NetflowMonthDetailReq {
    pub stu_id: String,
    pub year: String,
    pub month: String,
}

#[derive(Deserialize, Debug)]
pub struct NetflowDayDetailReq {
    pub stu_id: String,
    pub year: String,
    pub month: String,
    pub day: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowThisMonthRes {
    pub allBasePackageAmount: f64,
    pub allExtendPackageAmount: f64,
    pub allTraffic: String,
    pub basePackageUsed: f64,
    pub basePackageUsedPer: f64,
    pub downloadTraffic: String,
    pub extendPackageUsed: f64,
    pub extendPackageUsedPer: f64,
    pub surplusBasePackage: f64,
    pub surplusExtendPackage: f64,
    pub uploadTraffic: String,
}

#[derive(Deserialize, Debug)]
pub struct NetflowUnlockStatusRes {
    pub status: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowPayInfoRes {
    pub Total: f64,
}

#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowDetailRes {
    // #[serde(with = "serialize_f64")]
    pub AllDownload: f64,
    // #[serde(with = "serialize_f64")]
    pub AllTotal: f64,
    // #[serde(with = "serialize_f64")]
    pub AllUpload: f64,
    pub FloatDetailList: Vec<NetflowDetailItemRes>,
}
#[derive(Deserialize, Serialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowDetailItemRes {
    pub App: String,
    pub Download: f64,
    pub Per: f64,
    pub Total: f64,
    pub Upload: f64,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowOrderRes {
    // pub AddTime: String,
    // pub AllowOverTraffic: f64,
    // pub BaseTraffic: f64,
    pub Download: Option<f64>,
    // pub ExtTraffic: f64,
    pub Month: String,
    // pub PayOrderCode: Option<String>,
    // 1:已支付 0:未支付
    // pub PayState: u32,
    pub RealOverTraffic: f64,
    pub ShouldPay: f64,
    // pub Total: f64,
    pub UpdateTime: String,
    pub Upload: Option<f64>,
    // pub Year: String,
}

#[derive(Deserialize, Debug)]
#[expect(non_snake_case)]
pub struct NetflowOrderReturnItem {
    pub Download: Option<f64>,
    pub DownloadName: String,
    pub Month: String,
    pub RealOverTraffic: f64,
    pub ShouldPay: f64,
    pub UpdateTime: String,
    pub Upload: Option<f64>,
    pub UploadName: String,
}
