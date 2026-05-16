// documents 查询与写入

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use super::pool;
use crate::{result::AppResult, utils};

// 日期（未知年份则两项为空）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDate {
    // year / semester / grade
    pub typ: String,
    pub year: i32,
}

// 前端 Document DTO
#[derive(Debug, Clone, Serialize)]
pub struct Document {
    pub id: u32,
    pub date: Option<DocumentDate>,
    pub typ: String,
    pub name: String,
    pub answer: bool,
    pub page: u32,
    pub tags: Vec<String>,
    pub comment: Option<String>,
    pub md5: String,
    pub categories: Vec<String>,
}

// 含 file_path、created_at
#[derive(Debug, Clone)]
pub struct DocumentRow {
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
    // 未映射到 DTO
    #[allow(dead_code)]
    pub created_at: NaiveDateTime,
}

impl DocumentRow {
    pub fn into_dto(self) -> Document {
        Document {
            id: self.id,
            date: build_date(self.date_typ, self.date_year),
            typ: self.typ,
            name: self.name,
            answer: self.answer,
            page: self.page,
            tags: self.tags,
            comment: self.comment,
            md5: self.md5,
            categories: self.categories,
        }
    }
}

pub fn build_date(
    typ: Option<String>,
    year: Option<i32>,
) -> Option<DocumentDate> {
    match (typ, year) {
        (Some(t), Some(y)) => Some(DocumentDate { typ: t, year: y }),
        _ => None,
    }
}

fn parse_string_array(v: Value) -> Vec<String> {
    serde_json::from_value(v).unwrap_or_default()
}

fn row_to_doc(r: sqlx::mysql::MySqlRow) -> DocumentRow {
    DocumentRow {
        id: r.get::<u32, _>("id"),
        name: r.get("name"),
        typ: r.get("typ"),
        date_typ: r.get("date_typ"),
        date_year: r.get("date_year"),
        answer: r.get::<i8, _>("answer") != 0,
        page: r.get::<u32, _>("page"),
        tags: parse_string_array(r.get::<Value, _>("tags")),
        comment: r.get("comment"),
        md5: r.get("md5"),
        categories: parse_string_array(r.get::<Value, _>("categories")),
        file_path: r.get("file_path"),
        created_at: r.get("created_at"),
    }
}

pub async fn get_by_id(id: u32) -> AppResult<Option<DocumentRow>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, typ, date_typ, date_year, answer, page,
               tags, comment, md5, categories, file_path, created_at
        FROM documents WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool().await)
    .await?;
    Ok(row.map(row_to_doc))
}

pub async fn get_by_md5(md5: &str) -> AppResult<Option<DocumentRow>> {
    let row = sqlx::query(
        r#"
        SELECT id, name, typ, date_typ, date_year, answer, page,
               tags, comment, md5, categories, file_path, created_at
        FROM documents WHERE md5 = ?
        "#,
    )
    .bind(md5)
    .fetch_optional(pool().await)
    .await?;
    Ok(row.map(row_to_doc))
}

pub async fn get_by_ids(
    ids: &[u32],
) -> AppResult<Vec<DocumentRow>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    // 手动拼 IN(?,?,..) 占位符
    let placeholders =
        ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        r#"
        SELECT id, name, typ, date_typ, date_year, answer, page,
               tags, comment, md5, categories, file_path, created_at
        FROM documents WHERE id IN ({placeholders})
        "#
    );
    let mut q = sqlx::query(&sql);
    for id in ids {
        q = q.bind(*id);
    }
    let rows = q.fetch_all(pool().await).await?;
    Ok(rows.into_iter().map(row_to_doc).collect())
}

// insert → id
pub async fn insert(input: &NewDocument<'_>) -> AppResult<u32> {
    let res = sqlx::query(
        r#"
        INSERT INTO documents
            (name, typ, date_typ, date_year, answer, page, tags,
             comment, md5, categories, file_path, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
    .bind(utils::time::now())
    .execute(pool().await)
    .await?;
    Ok(res.last_insert_id() as u32)
}

#[derive(Debug)]
pub struct NewDocument<'a> {
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
}

// 搜索聚合（科目维度）
#[derive(Debug, Clone, Serialize)]
pub struct SubjectAgg {
    pub name: String,
    pub year: Option<i32>,
    pub count: i64,
}

// SubjectAgg：搜索列表项
pub async fn search_subjects(
    key: Option<&str>,
    typs: &[String],
    page: u32,
    page_size: u32,
) -> AppResult<(Vec<SubjectAgg>, i64)> {
    if typs.is_empty() {
        return Ok((vec![], 0));
    }

    let placeholders =
        typs.iter().map(|_| "?").collect::<Vec<_>>().join(",");

    // 总数
    let mut count_sql = format!(
        "SELECT COUNT(DISTINCT name) FROM documents WHERE typ IN ({placeholders})"
    );
    if key.is_some() {
        count_sql.push_str(" AND name LIKE ?");
    }
    let mut q = sqlx::query_scalar::<_, i64>(&count_sql);
    for t in typs {
        q = q.bind(t);
    }
    if let Some(k) = key {
        q = q.bind(format!("%{k}%"));
    }
    let total = q.fetch_one(pool().await).await?;

    // 列表
    let mut list_sql = format!(
        r#"
        SELECT name, MAX(date_year) AS year, COUNT(*) AS cnt
        FROM documents
        WHERE typ IN ({placeholders})
        "#
    );
    if key.is_some() {
        list_sql.push_str(" AND name LIKE ?");
    }
    list_sql.push_str(
        " GROUP BY name ORDER BY MAX(created_at) DESC LIMIT ? OFFSET ?",
    );
    let mut q = sqlx::query(&list_sql);
    for t in typs {
        q = q.bind(t);
    }
    if let Some(k) = key {
        q = q.bind(format!("%{k}%"));
    }
    let offset = page.saturating_sub(1) * page_size;
    q = q.bind(page_size).bind(offset);

    let rows = q.fetch_all(pool().await).await?;
    let list = rows
        .into_iter()
        .map(|r| SubjectAgg {
            name: r.get("name"),
            year: r.get("year"),
            count: r.get::<i64, _>("cnt"),
        })
        .collect();
    Ok((list, total))
}

// 科目下列全部试卷
pub async fn list_by_subject(
    name: Option<&str>,
    typ: Option<&str>,
) -> AppResult<Vec<DocumentRow>> {
    let mut sql = String::from(
        r#"
        SELECT id, name, typ, date_typ, date_year, answer, page,
               tags, comment, md5, categories, file_path, created_at
        FROM documents WHERE 1 = 1
        "#,
    );
    if name.is_some() {
        sql.push_str(" AND name = ?");
    }
    if typ.is_some() {
        sql.push_str(" AND typ = ?");
    }
    sql.push_str(" ORDER BY date_year DESC, id DESC");

    let mut q = sqlx::query(&sql);
    if let Some(n) = name {
        q = q.bind(n);
    }
    if let Some(t) = typ {
        q = q.bind(t);
    }
    let rows = q.fetch_all(pool().await).await?;
    Ok(rows.into_iter().map(row_to_doc).collect())
}
