use std::{sync::LazyLock, time::Duration};

use moka::{Expiry, future::Cache};

pub type CacheKey = (CacheEnum, String);
pub type CacheVal = String;

#[derive(Eq, Hash, PartialEq)]
pub enum CacheEnum {
    Hdjw,
    CasCookie,
    PtCookie,
    NetflowCookie,
    GymCookie,
    GraduateCookieAndId,
    #[expect(unused)]
    HdjwFailureRecord,
    XGXTCookie,
    LabCookie,
}

impl CacheEnum {
    // moka TTL
    fn expire_after_fetch(&self) -> Option<Duration> {
        use CacheEnum::*;
        match self {
            Hdjw => Some(Duration::from_secs(1800)),
            CasCookie => Some(Duration::from_secs(1800)),
            PtCookie => Some(Duration::from_secs(1800)),
            NetflowCookie => Some(Duration::from_secs(1800)),
            GymCookie => Some(Duration::from_secs(600)),
            GraduateCookieAndId => Some(Duration::from_secs(600)),
            HdjwFailureRecord => Some(Duration::from_secs(1800)),
            XGXTCookie => Some(Duration::from_secs(600)),
            LabCookie => Some(Duration::from_secs(600)),
        }
    }
}

pub static CACHE: LazyLock<Cache<CacheKey, CacheVal>> =
    LazyLock::new(|| {
        Cache::builder()
            .weigher(|k: &CacheKey, v: &CacheVal| {
                (k.1.len() + v.len() + 1)
                    .try_into()
                    .unwrap_or(u32::MAX)
            })
            // capacity是按照上面的weigher来算的，从而可以从内存容量角度来限制
            .max_capacity(2 * 1024 * 1024 * 1024)
            .expire_after(ExpiryPolicy)
            .build()
    });

struct ExpiryPolicy;

impl Expiry<CacheKey, CacheVal> for ExpiryPolicy {
    fn expire_after_create(
        &self,
        key: &CacheKey,
        _value: &CacheVal,
        _created_at: std::time::Instant,
    ) -> Option<Duration> {
        key.0.expire_after_fetch()
    }

    fn expire_after_update(
        &self,
        key: &CacheKey,
        _value: &CacheVal,
        _updated_at: std::time::Instant,
        _duration_until_expiry: Option<std::time::Duration>,
    ) -> Option<std::time::Duration> {
        key.0.expire_after_fetch()
    }
}

pub async fn invalidate_stuid_cache(stu_id: &str) {
    use CacheEnum::*;
    CACHE.invalidate(&(Hdjw, stu_id.into())).await;
    CACHE.invalidate(&(CasCookie, stu_id.into())).await;
    CACHE.invalidate(&(PtCookie, stu_id.into())).await;
    CACHE.invalidate(&(NetflowCookie, stu_id.into())).await;
    CACHE.invalidate(&(GymCookie, stu_id.into())).await;
    CACHE
        .invalidate(&(GraduateCookieAndId, stu_id.into()))
        .await;
    CACHE.invalidate(&(XGXTCookie, stu_id.into())).await;
    CACHE.invalidate(&(LabCookie, stu_id.into())).await;
}
