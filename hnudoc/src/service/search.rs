//! 搜索业务

use serde::Serialize;

use crate::{
    infra,
    infra::mysql::document::{Document, SubjectAgg},
    result::AppResult,
};

#[derive(Debug, Serialize)]
pub struct SearchSubjectsRes {
    pub subjects: Vec<SubjectAgg>,
    pub pages: u32,
}

/// 搜索科目（按 name 聚合）
pub async fn search_subjects(
    key: Option<&str>,
    typs: &[String],
    page: u32,
    page_size: u32,
) -> AppResult<SearchSubjectsRes> {
    let (subjects, total) =
        infra::mysql::document::search_subjects(
            key, typs, page, page_size,
        )
        .await?;
    let pages = if page_size == 0 {
        0
    } else {
        ((total as u64).div_ceil(page_size as u64)) as u32
    };
    Ok(SearchSubjectsRes { subjects, pages })
}

/// 列出某个科目下的全部试卷
pub async fn list_by_subject(
    name: Option<&str>,
    typ: Option<&str>,
) -> AppResult<Vec<Document>> {
    let rows = infra::mysql::document::list_by_subject(name, typ).await?;
    Ok(rows.into_iter().map(|r| r.into_dto()).collect())
}
