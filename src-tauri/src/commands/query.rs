// Query commands - SQL 执行

use crate::db::{Column, QueryResultMeta, QueryResultSet, RowChunk, RowValue};
use crate::db::ConnectionInfo;
use mysql_async::prelude::*;
use mysql_async::{Opts, Row};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 更新单元格的参数
#[derive(Deserialize)]
pub struct UpdateCellParams {
    pub connection: ConnectionInfo,
    pub table: String,
    pub column: String,
    /// 主键列名
    pub primary_key: String,
    /// 主键值
    pub primary_key_value: String,
    /// 新值
    pub new_value: String,
    /// 是否为 NULL
    pub is_null: bool,
}

/// 更新单元格的结果
#[derive(Serialize)]
pub struct UpdateCellResult {
    pub success: bool,
    pub message: String,
    #[serde(rename = "affectedRows")]
    pub affected_rows: u64,
}

/// 查询结果
#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    #[serde(rename = "queryId")]
    pub query_id: String,
    pub sets: Vec<QueryResultSet>,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
}

/// 更新单个单元格
#[tauri::command]
pub async fn query_update_cell(params: UpdateCellParams) -> Result<UpdateCellResult, String> {
    let opts = build_opts(&params.connection)?;

    // 创建连接
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 构建 UPDATE 语句
    let set_clause = if params.is_null {
        format!("`{}` = NULL", params.column)
    } else {
        format!("`{}` = {}", params.column, quote_value(&params.new_value))
    };

    let sql = format!(
        "UPDATE `{}` SET {} WHERE `{}` = {} LIMIT 1",
        format_table_ident(&params.table),
        set_clause,
        params.primary_key,
        quote_value(&params.primary_key_value)
    );

    // 执行更新
    conn.query_drop(&sql).await
        .map_err(|e| format!("更新失败: {}", e))?;

    let affected_rows = conn.affected_rows();

    // 关闭连接
    let _ = conn.disconnect().await;

    Ok(UpdateCellResult {
        success: true,
        message: format!("更新成功，影响 {} 行", affected_rows),
        affected_rows,
    })
}

/// 引用字符串值（用于 SQL）
fn quote_value(value: &str) -> String {
    // 如果是数字，直接返回
    if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
        return value.to_string();
    }
    // 否则用单引号包裹并转义
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
}

fn format_table_ident(table: &str) -> String {
    table
        .split('.')
        .filter(|part| !part.is_empty())
        .map(|part| format!("`{}`", part))
        .collect::<Vec<_>>()
        .join(".")
}

/// 执行 SQL
#[tauri::command]
pub async fn query_execute(
    connection: ConnectionInfo,
    sql: String,
    max_rows: usize,
) -> Result<QueryResult, String> {
    let opts = build_opts(&connection)?;
    let start = Instant::now();
    let query_id = uuid::Uuid::new_v4().to_string();

    // 创建连接
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 简单分割 SQL 语句
    let statements: Vec<&str> = sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut result_sets = Vec::new();

    for (idx, stmt) in statements.iter().enumerate() {
        let stmt_upper = stmt.to_uppercase();
        let is_select = stmt_upper.starts_with("SELECT")
            || stmt_upper.starts_with("SHOW")
            || stmt_upper.starts_with("DESCRIBE")
            || stmt_upper.starts_with("EXPLAIN")
            || stmt_upper.starts_with("WITH");

        if is_select {
            // 执行查询 - 使用 query_iter 以便即使没有行也能获取列信息
            let result = conn.query_iter(*stmt).await
                .map_err(|e| format!("SQL 错误: {}", e))?;

            // 获取列信息（从结果元数据中，不依赖行数据）
            let columns: Vec<Column> = result.columns()
                .iter()
                .flat_map(|col_slice| col_slice.iter())
                .map(|c: &mysql_async::Column| Column {
                    name: c.name_str().to_string(),
                    column_type: format!("{:?}", c.column_type()),
                    nullable: true,
                    default: None,
                    comment: None,
                    extra: None,
                })
                .collect();

            // 收集行数据
            let rows: Vec<Row> = result.collect_and_drop().await
                .map_err(|e| format!("SQL 错误: {}", e))?;

            // 转换行数据
            let result_rows: Vec<Vec<RowValue>> = rows
                .into_iter()
                .take(max_rows)
                .map(|row| {
                    (0..row.len())
                        .map(|i| row.get(i))
                        .map(|v| convert_mysql_value(v.unwrap_or(mysql_async::Value::NULL)))
                        .collect()
                })
                .collect();

            result_sets.push(QueryResultSet {
                set_index: idx,
                columns: columns.clone(),
                meta: QueryResultMeta {
                    columns,
                    affected_rows: result_rows.len() as u64,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    warning_count: 0,
                },
                chunks: vec![RowChunk {
                    chunk_index: 0,
                    rows: result_rows,
                }],
                paging: None,
            });
        } else {
            // 非查询语句
            conn.query_drop(*stmt).await
                .map_err(|e| format!("SQL 错误: {}", e))?;

            result_sets.push(QueryResultSet {
                set_index: idx,
                columns: vec![],
                meta: QueryResultMeta {
                    columns: vec![],
                    affected_rows: 0,
                    elapsed_ms: start.elapsed().as_millis() as u64,
                    warning_count: 0,
                },
                chunks: vec![],
                paging: None,
            });
        }
    }

    // 关闭连接
    let _ = conn.disconnect().await;

    Ok(QueryResult {
        query_id,
        sets: result_sets,
        elapsed_ms: start.elapsed().as_millis() as u64,
    })
}

fn convert_mysql_value(value: mysql_async::Value) -> RowValue {
    match value {
        mysql_async::Value::NULL => RowValue::Null,
        mysql_async::Value::Bytes(b) => {
            String::from_utf8(b.clone())
                .map(RowValue::String)
                .unwrap_or_else(|_| RowValue::Bytes(b))
        }
        mysql_async::Value::Int(i) => RowValue::Number(i as i64),
        mysql_async::Value::UInt(u) => RowValue::Number(u as i64),
        mysql_async::Value::Float(f) => RowValue::Float(f as f64),
        mysql_async::Value::Double(d) => RowValue::Float(d),
        _ => RowValue::String(format!("{:?}", value)),
    }
}

fn build_opts(conn: &ConnectionInfo) -> Result<Opts, String> {
    Opts::try_from(
        mysql_async::OptsBuilder::default()
            .ip_or_hostname(conn.host.clone())
            .tcp_port(conn.port)
            .user(Some(conn.user.clone()))
            .pass(Some(conn.password.clone()))
            .db_name(conn.default_db.as_ref().map(|s| s.as_str())),
    )
    .map_err(|e| format!("无效的连接参数: {}", e))
}

#[cfg(test)]
mod tests {
    use super::format_table_ident;

    #[test]
    fn format_table_ident_supports_qualified_name() {
        assert_eq!(format_table_ident("demo.users"), "`demo`.`users`");
        assert_eq!(format_table_ident("users"), "`users`");
    }
}
