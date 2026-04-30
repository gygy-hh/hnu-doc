//! HnuDoc 的统一响应类型与错误类型
//!
//! 与参考后端不同，HnuDoc 的接口约定使用 `status` 字段表示请求结果：
//! - 成功时 `status = "OK"`
//! - 失败时 `status = <错误代码>`，例如 `PASSWORD_ERROR`、`POW_KEY_INVALID`
//!
//! 失败时若有附加数据可放入 `data`，可读信息放入 `msg`。

use anyhow::anyhow;
use salvo::http::StatusCode;
use salvo::prelude::Json;
use salvo::{Response, Scribe};
use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;

/// 业务侧通用错误代码
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrCode {
    /// 通用未授权（缺少/无效 token）
    Unauthorized,
    /// 个人门户密码错误
    PasswordError,
    /// 该账号没有绑定湖大微生活
    #[allow(dead_code)]
    NotBindWeihuda,
    /// 该账号被禁止使用试卷库
    PermissionDenied,
    /// 文件已经存在（上传重复）
    FileExisted,
    /// 文件超出大小限制
    FileSizeLimitExceeded,
    /// POW 计算的 key 错误（同时 ticket 也失效）
    PowKeyInvalid,
    /// 资源不存在
    NotFound,
    /// 参数错误
    BadRequest,
    /// 自定义错误代码（透传到前端）
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

/// 自定义错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 未知错误，500
    #[error("服务器内部错误: {0:?}")]
    AnyHow(#[from] anyhow::Error),
    /// 参数解析错误
    #[error("参数解析错误: {0}")]
    SalvoParseError(#[from] salvo::http::ParseError),
    /// 与 Salvo 解析失败区分时的占位，路由层可显式返回
    #[error("参数解析错误")]
    #[allow(dead_code)]
    ParseError,
    /// 数据库错误
    #[error("数据库 SQL 执行错误: {0}")]
    SqlxError(#[from] sqlx::Error),
    /// JWT 错误
    #[error("JWT 编解码错误: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    /// JSON 错误
    #[error("解析 JSON 错误: {0}")]
    JsonParseError(#[from] serde_json::Error),
    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    /// Redis 错误
    #[error("Redis 错误: {0}")]
    RedisError(#[from] redis::RedisError),
    /// 业务侧错误（带错误代码 + 附加数据）
    #[error("{code:?}: {msg}")]
    Biz {
        code: ErrCode,
        msg: String,
        data: Value,
    },
    /// 请求超时
    #[error("请求超时")]
    TimeoutError,
}

impl AppError {
    /// 构造一个不附带 data 的业务错误
    pub fn biz(code: ErrCode, msg: impl Into<String>) -> Self {
        Self::Biz {
            code,
            msg: msg.into(),
            data: Value::Null,
        }
    }

    /// 构造一个附带 data 的业务错误
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

impl From<spider_2024::Error> for AppError {
    fn from(e: spider_2024::Error) -> Self {
        use spider_2024::Error as SE;
        match e {
            SE::AnyHow(error) => Self::AnyHow(error),
            SE::PasswordError => {
                Self::biz(ErrCode::PasswordError, "密码错误")
            }
            SE::PasswordShouldChange => Self::biz(
                ErrCode::PasswordError,
                "请前往个人门户修改密码后重试",
            ),
            SE::PasswordLocked => Self::biz(
                ErrCode::PasswordError,
                "账号被锁定，请暂停使用 10 分钟后重试",
            ),
            SE::SqlxError(error) => Self::SqlxError(error),
            SE::RedisErr(redis_error) => Self::RedisError(redis_error),
        }
    }
}

/// 成功响应：`{"status":"OK","data":<value>,"msg":null}`
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
