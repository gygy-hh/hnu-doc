// collections

use chrono::NaiveDateTime;
use sqlx::Row;

use super::pool;
use crate::result::AppResult;

#[derive(Debug, Clone)]
pub struct CollectionRow {
    pub id: u32,
    pub name: String,
    pub description: String,
    #[allow(dead_code)]
    pub created_at: NaiveDateTime,
}

// (collection_id, document_id, sort_order)
#[derive(Debug, Clone)]
pub struct CollectionItemRow {
    pub collection_id: u32,
    pub document_id: u32,
    #[allow(dead_code)]
    pub sort_order: i32,
}

pub async fn list_all() -> AppResult<Vec<CollectionRow>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, description, created_at
        FROM collections
        ORDER BY id ASC
        "#,
    )
    .fetch_all(pool().await)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| CollectionRow {
            id: r.get::<u32, _>("id"),
            name: r.get("name"),
            description: r.get("description"),
            created_at: r.get("created_at"),
        })
        .collect())
}

// 全部关联行（有序）
pub async fn list_all_items() -> AppResult<Vec<CollectionItemRow>> {
    let rows = sqlx::query(
        r#"
        SELECT collection_id, document_id, sort_order
        FROM collection_items
        ORDER BY collection_id ASC, sort_order ASC, document_id ASC
        "#,
    )
    .fetch_all(pool().await)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| CollectionItemRow {
            collection_id: r.get::<u32, _>("collection_id"),
            document_id: r.get::<u32, _>("document_id"),
            sort_order: r.get::<i32, _>("sort_order"),
        })
        .collect())
}
