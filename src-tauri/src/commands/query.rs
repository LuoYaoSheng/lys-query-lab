// Query commands - SQL 执行

use crate::db::{Column, QueryResultMeta, QueryResultSet, RowChunk, RowValue};
use crate::db::ConnectionInfo;
use mysql_async::prelude::*;
use mysql_async::{Opts, Row};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 查询结果
#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    #[serde(rename = "queryId")]
    pub query_id: String,
    pub sets: Vec<QueryResultSet>,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u64,
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
            // 执行查询
            let rows: Vec<Row> = conn.query(*stmt).await
                .map_err(|e| format!("SQL 错误: {}", e))?;

            // 获取列信息
            let columns: Vec<Column> = rows.first()
                .map(|first_row| {
                    first_row.columns()
                        .iter()
                        .map(|c: &mysql_async::Column| Column {
                            name: c.name_str().to_string(),
                            column_type: format!("{:?}", c.column_type()),
                            nullable: true,
                            default: None,
                            comment: None,
                            extra: None,
                        })
                        .collect()
                })
                .unwrap_or_default();

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
