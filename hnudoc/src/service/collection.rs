// 试卷集 CRUD 聚合

use std::collections::HashMap;

use serde::Serialize;

use crate::{
    infra,
    infra::mysql::document::Document,
    result::AppResult,
};

#[derive(Debug, Serialize)]
pub struct Collection {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub items: Vec<Document>,
}

// 全集 + 文档列表
pub async fn list_all() -> AppResult<Vec<Collection>> {
    let cols = infra::mysql::collection::list_all().await?;
    if cols.is_empty() {
        return Ok(vec![]);
    }

    let items = infra::mysql::collection::list_all_items().await?;
    if items.is_empty() {
        return Ok(cols
            .into_iter()
            .map(|c| Collection {
                id: c.id,
                name: c.name,
                description: c.description,
                items: vec![],
            })
            .collect());
    }

    // 把所有 document_id 一次性取出来
    let doc_ids: Vec<u32> =
        items.iter().map(|x| x.document_id).collect();
    let docs =
        infra::mysql::document::get_by_ids(&doc_ids).await?;
    let docs_map: HashMap<u32, Document> = docs
        .into_iter()
        .map(|d| {
            let dto = d.into_dto();
            (dto.id, dto)
        })
        .collect();

    // 按 collection_id 分组
    let mut groups: HashMap<u32, Vec<Document>> = HashMap::new();
    for it in items {
        if let Some(d) = docs_map.get(&it.document_id).cloned() {
            groups.entry(it.collection_id).or_default().push(d);
        }
    }

    Ok(cols
        .into_iter()
        .map(|c| Collection {
            id: c.id,
            name: c.name,
            description: c.description,
            items: groups.remove(&c.id).unwrap_or_default(),
        })
        .collect())
}
