// Database backup and restore commands

use crate::db::ConnectionInfo;
use mysql_async::prelude::*;
use mysql_async::{Opts, Row};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

/// 导出参数
#[derive(Deserialize)]
pub struct ExportParams {
    pub connection: ConnectionInfo,
    pub database: String,
    pub tables: Vec<String>,
    pub export_type: String,  // 'structure', 'data', 'both'
    pub format: String,        // 'sql', 'json', 'csv'
    pub file_path: String,
}

/// 导出结果
#[derive(Serialize)]
pub struct ExportResult {
    pub success: bool,
    pub size: u64,
    pub tables: usize,
    pub message: String,
}

/// 导入参数
#[derive(Deserialize)]
pub struct ImportParams {
    pub connection: ConnectionInfo,
    pub database: String,
    pub file_path: String,
    pub drop_existing: bool,
}

/// 导入结果
#[derive(Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub tables: usize,
    pub rows: usize,
    pub message: String,
}

/// 导出数据库
#[tauri::command]
pub async fn db_export(params: ExportParams) -> Result<ExportResult, String> {
    let opts = build_opts(&params.connection)?;
    let start = Instant::now();

    // 创建连接
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| format!("连接失败: {}", e))?;

    let mut sql_content = String::new();
    let mut exported_tables = 0;

    // 添加头部注释
    sql_content.push_str(&format!("-- Database Backup: {}\n", params.database));
    sql_content.push_str(&format!("-- Generated at: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    sql_content.push_str(&format!("-- Export Type: {}\n", params.export_type));
    sql_content.push_str("-- ----------------------------------------\n\n");

    // 设置数据库
    sql_content.push_str(&format!("USE `{}`;\n\n", params.database));

    // 导出每个表
    for table_name in &params.tables {
        // 获取表结构
        let create_sql = get_table_structure(&mut conn, &params.database, table_name).await?;

        if params.export_type == "structure" || params.export_type == "both" {
            sql_content.push_str(&format!("-- Table structure for `{}`\n", table_name));
            sql_content.push_str(&format!("DROP TABLE IF EXISTS `{}`;\n", table_name));
            sql_content.push_str(&create_sql);
            sql_content.push_str("\n\n");
        }

        if params.export_type == "data" || params.export_type == "both" {
            // 获取表数据
            let data_sql = get_table_data(&mut conn, &params.database, table_name).await?;
            if !data_sql.is_empty() {
                sql_content.push_str(&format!("-- Data for table `{}`\n", table_name));
                sql_content.push_str(&data_sql);
                sql_content.push_str("\n\n");
            }
        }

        exported_tables += 1;
    }

    // 根据格式处理输出
    let final_content = match params.format.as_str() {
        "sql" => sql_content,
        "json" => {
            // 简化：将SQL作为JSON字符串
            // 实际应用中可以解析为结构化JSON
            serde_json::json!({
                "database": params.database,
                "export_type": params.export_type,
                "tables": params.tables,
                "sql": sql_content,
                "timestamp": chrono::Local::now().to_rfc3339()
            }).to_string()
        }
        "csv" => {
            // 简化：CSV格式仅导出第一个表的数据
            if let Some(first_table) = params.tables.first() {
                export_table_as_csv(&mut conn, &params.database, first_table).await?
            } else {
                String::new()
            }
        }
        _ => sql_content,
    };

    // 关闭连接
    let _ = conn.disconnect().await;

    // 写入文件
    let mut file = File::create(&params.file_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    file.write_all(final_content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let file_size = std::fs::metadata(&params.file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(ExportResult {
        success: true,
        size: file_size,
        tables: exported_tables,
        message: format!("导出完成，耗时 {}ms", start.elapsed().as_millis()),
    })
}

/// 导入数据库
#[tauri::command]
pub async fn db_import(params: ImportParams) -> Result<ImportResult, String> {
    let opts = build_opts(&params.connection)?;
    let start = Instant::now();

    // 读取文件
    let sql_content = std::fs::read_to_string(&params.file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 创建连接
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| format!("连接失败: {}", e))?;

    // 设置数据库
    conn.query_drop(format!("USE `{}`", params.database)).await
        .map_err(|e| format!("选择数据库失败: {}", e))?;

    // 如果需要，删除现有表
    if params.drop_existing {
        let tables: Vec<String> = conn.query("SHOW TABLES").await
            .map_err(|e| format!("获取表列表失败: {}", e))?;

        for table in tables {
            let _ = conn.query_drop(format!("DROP TABLE IF EXISTS `{}`", table)).await;
        }
    }

    // 分割并执行SQL语句
    let statements = split_sql_statements(&sql_content);
    let mut imported_tables = 0;
    let mut imported_rows = 0;

    for stmt in statements {
        if stmt.trim().is_empty() || stmt.starts_with("--") {
            continue;
        }

        // 检查是否是建表语句（在移动 stmt 之前）
        let upper = stmt.to_uppercase();
        let is_create_table = upper.contains("CREATE TABLE");

        match conn.query::<Row, _>(&stmt).await {
            Ok(_) => {
                if is_create_table {
                    imported_tables += 1;
                }
            }
            Err(e) => {
                // 忽略已存在表等非致命错误
                let err_msg = e.to_string().to_lowercase();
                if !err_msg.contains("already exists") && !err_msg.contains("duplicate") {
                    eprintln!("SQL warning: {}", e);
                }
            }
        }
    }

    // 获取总行数（估算）
    let result = conn.query::<usize, _>("SELECT SUM(TABLE_ROWS) FROM information_schema.TABLES WHERE TABLE_SCHEMA = DATABASE()").await;
    if let Ok(counts) = result {
        if let Some(count) = counts.first() {
            imported_rows = *count;
        }
    }

    // 关闭连接
    let _ = conn.disconnect().await;

    Ok(ImportResult {
        success: true,
        tables: imported_tables,
        rows: imported_rows,
        message: format!("导入完成，耗时 {}ms", start.elapsed().as_millis()),
    })
}

/// 获取表结构
async fn get_table_structure(conn: &mut mysql_async::Conn, database: &str, table: &str) -> Result<String, String> {
    let rows: Vec<Row> = conn.query(format!("SHOW CREATE TABLE `{}`.`{}`", database, table)).await
        .map_err(|e| format!("获取表结构失败: {}", e))?;

    if let Some(row) = rows.first() {
        let create_sql: String = row.get("Create Table").unwrap_or_default();
        Ok(create_sql + ";\n")
    } else {
        Ok(String::new())
    }
}

/// 获取表数据（INSERT语句）
async fn get_table_data(conn: &mut mysql_async::Conn, database: &str, table: &str) -> Result<String, String> {
    let columns: Vec<String> = conn.query(format!(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' ORDER BY ORDINAL_POSITION",
        database, table
    )).await.map_err(|e| format!("获取列信息失败: {}", e))?;

    if columns.is_empty() {
        return Ok(String::new());
    }

    let rows: Vec<Row> = conn.query(format!("SELECT * FROM `{}`.`{}` LIMIT 10000", database, table)).await
        .map_err(|e| format!("获取表数据失败: {}", e))?;

    if rows.is_empty() {
        return Ok(String::new());
    }

    let mut sql = String::new();
    let col_names: String = columns.iter().map(|c| format!("`{}`", c)).collect::<Vec<_>>().join(", ");

    for row in rows {
        let values: Vec<String> = columns.iter().enumerate().map(|(i, _)| {
            let value: Option<String> = row.get(i).unwrap_or(None);
            match value {
                Some(v) => format!("'{}", v.replace('\\', "\\\\").replace('\'', "''")),
                None => "NULL".to_string()
            }
        }).collect();

        sql.push_str(&format!("INSERT INTO `{}` ({}) VALUES ({});\n",
            table, col_names, values.join(", ")));
    }

    Ok(sql)
}

/// 导出表为CSV
async fn export_table_as_csv(conn: &mut mysql_async::Conn, database: &str, table: &str) -> Result<String, String> {
    let columns: Vec<String> = conn.query(format!(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' ORDER BY ORDINAL_POSITION",
        database, table
    )).await.map_err(|e| format!("获取列信息失败: {}", e))?;

    let rows: Vec<Row> = conn.query(format!("SELECT * FROM `{}`.`{}` LIMIT 10000", database, table)).await
        .map_err(|e| format!("获取表数据失败: {}", e))?;

    let mut csv = String::new();

    // CSV 头部
    csv.push_str(&columns.join(","));
    csv.push('\n');

    // CSV 数据
    for row in rows {
        let values: Vec<String> = columns.iter().enumerate().map(|(i, _)| {
            let value: Option<String> = row.get(i).unwrap_or(None);
            match value {
                Some(v) => {
                    // 转义CSV特殊字符
                    let escaped = v.replace('"', "\"\"");
                    if v.contains(',') || v.contains('"') || v.contains('\n') {
                        format!("\"{}\"", escaped)
                    } else {
                        escaped
                    }
                }
                None => String::new()
            }
        }).collect();

        csv.push_str(&values.join(","));
        csv.push('\n');
    }

    Ok(csv)
}

/// 分割SQL语句（简单实现）
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = ' ';

    for ch in sql.chars() {
        if ch == '"' || ch == '\'' || ch == '`' {
            if !in_string {
                in_string = true;
                string_char = ch;
            } else if ch == string_char {
                in_string = false;
            }
        }

        if ch == ';' && !in_string {
            current.push(ch);
            let trimmed = current.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                statements.push(trimmed.to_string());
            }
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() && !trimmed.starts_with("--") {
        statements.push(trimmed.to_string());
    }

    statements
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
