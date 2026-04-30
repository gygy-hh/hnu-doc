//! `pending_documents` 表

use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::Row;

use super::pool;
use crate::{result::AppResult, utils};

#[derive(Debug, Clone)]
pub struct PendingDocumentRow {
    pub id: u32,
    pub name: String,
    pub typ: String,
    pub date_typ: Option<String>,
    pub date_year: Option<i32>,
    pub answer: bool,
    pub page: u32,
    pub tags: Vec<String>,
    pub comment: Option<String>,
    pub md5: String,
    pub categories: Vec<String>,
    pub file_path: String,
    pub status: String,
    pub stu_id: String,
    pub audit_comment: Option<String>,
    pub target: Option<u32>,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
}

fn parse_arr(v: Value) -> Vec<String> {
    serde_json::from_value(v).unwrap_or_default()
}

fn map_row(r: sqlx::mysql::MySqlRow) -> PendingDocumentRow {
    PendingDocumentRow {
        id: r.get::<u32, _>("id"),
        name: r.get("name"),
        typ: r.get("typ"),
        date_typ: r.get("date_typ"),
        date_year: r.get("date_year"),
        answer: r.get::<i8, _>("answer") != 0,
        page: r.get::<u32, _>("page"),
        tags: parse_arr(r.get::<Value, _>("tags")),
        comment: r.get("comment"),
        md5: r.get("md5"),
        categories: parse_arr(r.get::<Value, _>("categories")),
        file_path: r.get("file_path"),
        status: r.get("status"),
        stu_id: r.get("stu_id"),
        audit_comment: r.get("audit_comment"),
        target: r.get::<Option<u32>, _>("target"),
        create_time: r.get("create_time"),
        update_time: r.get("update_time"),
    }
}

pub async fn get_by_id(
    id: u32,
) -> AppResult<Option<PendingDocumentRow>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, typ, date_typ, date_year, answer, page,
               tags, comment, md5, categories, file_path, status,
               stu_id, audit_comment, target, create_time, update_time
        FROM pending_documents WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool().await)
    .await?;
    Ok(row.map(map_row))
}

pub async fn list_by_status(
    status: &str,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<PendingDocumentRow>> {
    let offset = page.saturating_sub(1) * page_size;
    let rows = sqlx::query(
        r#"
        SELECT id, name, typ, date_typ, date_year, answer, page,
               tags, comment, md5, categories, file_path, status,
               stu_id, audit_comment, target, create_time, update_time
        FROM pending_documents
        WHERE status = ?
        ORDER BY id DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(status)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool().await)
    .await?;
    Ok(rows.into_iter().map(map_row).collect())
}

#[derive(Debug)]
pub struct NewPendingDocument<'a> {
    pub name: &'a str,
    pub typ: &'a str,
    pub date_typ: Option<&'a str>,
    pub date_year: Option<i32>,
    pub answer: bool,
    pub page: u32,
    pub tags: &'a [String],
    pub comment: Option<&'a str>,
    pub md5: &'a str,
    pub categories: &'a [String],
    pub file_path: &'a str,
    pub stu_id: &'a str,
}

pub async fn insert(
    input: &NewPendingDocument<'_>,
) -> AppResult<u32> {
    let now = utils::time::now();
    let res = sqlx::query(
        r#"
        INSERT INTO pending_documents
            (name, typ, date_typ, date_year, answer, page, tags,
             comment, md5, categories, file_path, status, stu_id,
             create_time, update_time)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?)
        "#,
    )
    .bind(input.name)
    .bind(input.typ)
    .bind(input.date_typ)
    .bind(input.date_year)
    .bind(input.answer as i8)
    .bind(input.page)
    .bind(serde_json::to_value(input.tags)?)
    .bind(input.comment)
    .bind(input.md5)
    .bind(serde_json::to_value(input.categories)?)
    .bind(input.file_path)
    .bind(input.stu_id)
    .bind(now)
    .bind(now)
    .execute(pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

#[derive(Debug)]
pub struct UpdatePending<'a> {
    pub name: &'a str,
    pub typ: &'a str,
    pub date_typ: Option<&'a str>,
    pub date_year: Option<i32>,
    pub answer: bool,
    pub page: u32,
    pub tags: &'a [String],
    pub comment: Option<&'a str>,
    pub md5: &'a str,
    pub categories: &'a [String],
    /// 仅当替换文件时填充
    pub file_path: Option<&'a str>,
}

pub async fn update_meta(
    id: u32,
    u: &UpdatePending<'_>,
) -> AppResult<()> {
    let now = utils::time::now();
    if let Some(file_path) = u.file_path {
        sqlx::query(
            r#"
            UPDATE pending_documents
            SET name = ?, typ = ?, date_typ = ?, date_year = ?,
                answer = ?, page = ?, tags = ?, comment = ?,
                md5 = ?, categories = ?, file_path = ?, update_time = ?
            WHERE id = ?
            "#,
        )
        .bind(u.name)
        .bind(u.typ)
        .bind(u.date_typ)
        .bind(u.date_year)
        .bind(u.answer as i8)
        .bind(u.page)
        .bind(serde_json::to_value(u.tags)?)
        .bind(u.comment)
        .bind(u.md5)
        .bind(serde_json::to_value(u.categories)?)
        .bind(file_path)
        .bind(now)
        .bind(id)
        .execute(pool().await)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE pending_documents
            SET name = ?, typ = ?, date_typ = ?, date_year = ?,
                answer = ?, page = ?, tags = ?, comment = ?,
                md5 = ?, categories = ?, update_time = ?
            WHERE id = ?
            "#,
        )
        .bind(u.name)
        .bind(u.typ)
        .bind(u.date_typ)
        .bind(u.date_year)
        .bind(u.answer as i8)
        .bind(u.page)
        .bind(serde_json::to_value(u.tags)?)
        .bind(u.comment)
        .bind(u.md5)
        .bind(serde_json::to_value(u.categories)?)
        .bind(now)
        .bind(id)
        .execute(pool().await)
        .await?;
    }
    Ok(())
}

pub async fn set_status(
    id: u32,
    status: &str,
    audit_comment: Option<&str>,
    target: Option<u32>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE pending_documents
        SET status = ?, audit_comment = ?, target = ?, update_time = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(audit_comment)
    .bind(target)
    .bind(utils::time::now())
    .bind(id)
    .execute(pool().await)
    .await?;
    Ok(())
}
