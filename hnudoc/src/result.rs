// 统一 JSON：成功 status=OK，失败 status=错误码；msg/data 可选

use anyhow::anyhow;
use salvo::http::StatusCode;
use salvo::prelude::Json;
use salvo::{Response, Scribe};
use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;

// 业务错误码

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrCode {
    Unauthorized,
    PasswordError,
    #[allow(dead_code)]
    NotBindWeihuda,
    PermissionDenied,
    FileExisted,
    FileSizeLimitExceeded,
    PowKeyInvalid,
    NotFound,
    BadRequest,
    #[allow(dead_code)]
    Custom(&'static str),
}

impl ErrCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::PasswordError => "PASSWORD_ERROR",
            Self::NotBindWeihuda => "NOT_BIND_WEIHUDA",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::FileExisted => "FILE_EXISTED",
            Self::FileSizeLimitExceeded => "FILE_SIZE_LIMIT_EXCEEDED",
            Self::PowKeyInvalid => "POW_KEY_INVALID",
            Self::NotFound => "NOT_FOUND",
            Self::BadRequest => "BAD_REQUEST",
            Self::Custom(s) => s,
        }
    }

    pub fn http_status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest
            | Self::FileSizeLimitExceeded
            | Self::PowKeyInvalid
            | Self::FileExisted => StatusCode::BAD_REQUEST,
            Self::PasswordError
            | Self::NotBindWeihuda
            | Self::PermissionDenied => StatusCode::FORBIDDEN,
            Self::Custom(_) => StatusCode::OK,
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("服务器内部错误: {0:?}")]
    AnyHow(#[from] anyhow::Error),
    #[error("参数解析错误: {0}")]
    SalvoParseError(#[from] salvo::http::ParseError),
    #[error("参数解析错误")]
    #[allow(dead_code)]
    ParseError,
    #[error("数据库 SQL 执行错误: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("JWT 编解码错误: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("解析 JSON 错误: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Redis 错误: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("{code:?}: {msg}")]
    Biz {
        code: ErrCode,
        msg: String,
        data: Value,
    },
    #[error("请求超时")]
    TimeoutError,
}

impl AppError {
    pub fn biz(code: ErrCode, msg: impl Into<String>) -> Self {
        Self::Biz {
            code,
            msg: msg.into(),
            data: Value::Null,
        }
    }

    pub fn biz_with_data(
        code: ErrCode,
        msg: impl Into<String>,
        data: Value,
    ) -> Self {
        Self::Biz {
            code,
            msg: msg.into(),
            data,
        }
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::AnyHow(anyhow!(s.to_string()))
    }
}

// OK 响应包装
pub struct Success(Value);

impl<T: Serialize> From<T> for Success {
    fn from(value: T) -> Self {
        Self(json!({
            "status": "OK",
            "data": value,
            "msg": Value::Null,
        }))
    }
}

impl Scribe for Success {
    fn render(self, res: &mut Response) {
        res.stuff(StatusCode::OK, Json(self.0));
    }
}

impl Scribe for AppError {
    fn render(self, res: &mut Response) {
        tracing::error!("{}", self);
        match self {
            AppError::Biz { code, msg, data } => {
                let status = code.http_status();
                res.stuff(
                    status,
                    Json(json!({
                        "status": code.as_str(),
                        "data": data,
                        "msg": msg,
                    })),
                );
            }
            AppError::ParseError | AppError::SalvoParseError(_) => {
                res.stuff(
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "BAD_REQUEST",
                        "data": null,
                        "msg": "参数解析错误",
                    })),
                );
            }
            AppError::JwtError(_) => {
                res.stuff(
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "status": "UNAUTHORIZED",
                        "data": null,
                        "msg": "身份验证失败",
                    })),
                );
            }
            AppError::TimeoutError => {
                res.stuff(
                    StatusCode::REQUEST_TIMEOUT,
                    Json(json!({
                        "status": "TIMEOUT",
                        "data": null,
                        "msg": "请求超时，请重试",
                    })),
                );
            }
            other => {
                let msg = format!("{}", other);
                res.stuff(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "INTERNAL_ERROR",
                        "data": null,
                        "msg": msg,
                    })),
                );
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type RouterResult = AppResult<Success>;