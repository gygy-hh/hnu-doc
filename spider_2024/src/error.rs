use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // 不需要特殊处理的错误
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error),
    // 登录服务时提供的账号密码错误，前端可能需要重新处理
    #[error("password error")]
    PasswordError,
    #[error("password should change")]
    PasswordShouldChange,
    #[error("password is locked")]
    PasswordLocked,
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("redis::RedisError:`{0}`")]
    RedisErr(#[from] redis::RedisError),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<reqwest::header::ToStrError> for Error {
    fn from(err: reqwest::header::ToStrError) -> Self {
        Error::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        Error::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<Arc<Error>> for Error {
    fn from(err: Arc<Error>) -> Self {
        match &(*err) {
            Error::AnyHow(_)
            | Error::SqlxError(_)
            | Error::RedisErr(_) => {
                Error::AnyHow(anyhow::anyhow!(err))
            }
            Error::PasswordError => Error::PasswordError,
            Error::PasswordShouldChange => {
                Error::PasswordShouldChange
            }
            Error::PasswordLocked => Error::PasswordLocked,
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::AnyHow(anyhow::anyhow!(err))
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(err: std::time::SystemTimeError) -> Self {
        Error::AnyHow(anyhow::anyhow!(err))
    }
}
