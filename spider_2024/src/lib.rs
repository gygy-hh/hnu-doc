#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(rustdoc::all)]
#![warn(clippy::allow_attributes)]
#![warn(clippy::too_many_lines)]
#![warn(clippy::too_long_first_doc_paragraph)]
#![warn(
    clippy::todo,
    reason = "在`git commit`之前，请确认代码中没有`todo!()`"
)]

pub mod config;
pub mod dtos;
mod error;
mod handlers;
mod spiders;
mod utils;

pub use error::Error;
pub use handlers::{electricity, gym, hdjw, lab, netflow, pt, xgxt};
