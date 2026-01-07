// Database driver abstraction - 为多数据库准备

use async_trait::async_trait;

use super::types::*;

/// 数据库能力
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub supports_explain: bool,
    pub supports_procedures: bool,
    pub supports_ssh_tunnel: bool,
    pub supports_returning: bool,
    pub supports_limit_offset: bool,
}

/// 数据库 Driver trait
#[async_trait]
pub trait Driver: Send + Sync {
    /// Driver ID (mysql, postgres, sqlite)
    fn id(&self) -> &'static str;

    /// 支持的能力
    fn capabilities(&self) -> &Capabilities;

    /// 连接数据库
    async fn connect(&self, conn_info: &ConnectionInfo) -> Result<Box<dyn DbConnection>, DbError>;
}

/// 数据库连接
#[async_trait]
pub trait DbConnection: Send + Sync {
    /// 列出数据库
    async fn list_databases(&self) -> Result<Vec<String>, DbError>;

    /// 列出表
    async fn list_tables(&self, database: &str, include_views: bool) -> Result<Vec<TableInfo>, DbError>;

    /// 获取表结构
    async fn get_table_schema(&self, database: &str, table: &str) -> Result<TableSchema, DbError>;

    /// 执行查询
    async fn query(&self, sql: &str, options: &QueryOptions) -> Result<QueryResultSet, DbError>;

    /// 执行语句（INSERT/UPDATE/DELETE）
    async fn execute(&self, sql: &str) -> Result<ExecResult, DbError>;

    /// 取消查询
    async fn cancel(&self, query_id: &str) -> Result<bool, DbError>;
}

/// 连接信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "driver")]
    pub driver_type: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultDb")]
    pub default_db: Option<String>,
}

/// 表信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub table_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(rename = "rowsEst")]
    pub rows_est: u64,
}

/// 查询选项
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(rename = "maxRows")]
    pub max_rows: usize,
    #[serde(rename = "timeoutMs")]
    pub timeout_ms: u64,
    pub paging: Option<PagingOptions>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PagingOptions {
    pub enabled: bool,
    pub page: usize,
    #[serde(rename = "pageSize")]
    pub page_size: usize,
}

/// 执行结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecResult {
    #[serde(rename = "affectedRows")]
    pub affected_rows: u64,
    #[serde(rename = "lastInsertId")]
    pub last_insert_id: Option<u64>,
}

/// 数据库错误
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Timeout")]
    Timeout,

    #[error("Cancelled")]
    Cancelled,
}

// MySQL driver placeholder
pub mod mysql {
    use super::*;

    pub struct MySQLDriver;

    impl MySQLDriver {
        pub fn new() -> Self {
            Self
        }
    }

    // TODO: 实现 Driver trait
}
