// 通用数据类型 - 前后端共享

use serde::{Deserialize, Serialize};

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub nullable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub columns: Vec<String>,
}

/// 外键信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    #[serde(rename = "refTable")]
    pub ref_table: String,
    #[serde(rename = "refColumns")]
    pub ref_columns: Vec<String>,
}

/// 表结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub database: String,
    pub table: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub foreign_keys: Vec<ForeignKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_sql: Option<String>,
}

/// 查询结果元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultMeta {
    pub columns: Vec<Column>,
    #[serde(rename = "affectedRows")]
    pub affected_rows: u64,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
    #[serde(rename = "warningCount")]
    pub warning_count: u32,
}

/// 行值（统一类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RowValue {
    Null,
    Bool(bool),
    Number(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
}

/// 查询结果集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResultSet {
    #[serde(rename = "setIndex")]
    pub set_index: usize,
    pub columns: Vec<Column>,
    pub meta: QueryResultMeta,
    pub chunks: Vec<RowChunk>,
    pub paging: Option<PagingInfo>,
}

/// 行数据块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowChunk {
    #[serde(rename = "chunkIndex")]
    pub chunk_index: usize,
    pub rows: Vec<Vec<RowValue>>,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagingInfo {
    pub page: usize,
    #[serde(rename = "pageSize")]
    pub page_size: usize,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}
