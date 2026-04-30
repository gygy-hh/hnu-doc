use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetElectricityReq {
    pub park: u8,
    pub build: String,
    pub room: String,
    pub refresh: bool,
}
