use crate::{
    dtos::electricity::GetElectricityReq,
    spiders,
    utils::redis::{add_cookie_to_redis, get_cookie_from_redis},
};

const CACHE_TIMEOUT: i64 = 60 * 60 * 16;

pub async fn get_electricity_handler(
    req: GetElectricityReq,
) -> Result<String, crate::Error> {
    let key = format!(
        "e{}{}{}",
        req.park.clone(),
        req.build.clone(),
        req.room.clone()
    );
    let mut res = get_cookie_from_redis(key.as_str(), "").await;
    if res.is_err() || req.refresh {
        let t = spiders::electricity::get_electricity(
            req.park, req.build, req.room,
        )
        .await?;
        add_cookie_to_redis(
            key.as_str(),
            t.as_str(),
            "",
            CACHE_TIMEOUT,
        )
        .await?;
        res = Ok(t);
    }
    Ok(res?)
}
