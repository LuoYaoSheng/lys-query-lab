// Metadata commands - Schema 浏览

use crate::db::{Column, Index, TableInfo, TableSchema};
use crate::db::ConnectionInfo;
use mysql_async::prelude::*;
use mysql_async::{Opts, Row};
use serde::{Deserialize, Serialize};

/// 创建数据库参数
#[derive(Deserialize)]
pub struct CreateDatabaseParams {
    pub connection: ConnectionInfo,
    pub name: String,
    #[serde(default)]
    pub charset: String,
    #[serde(default)]
    pub collation: String,
}

/// 创建表参数
#[derive(Deserialize, Debug)]
pub struct CreateTableParams {
    pub connection: ConnectionInfo,
    pub database: String,
    pub table: String,
    pub columns: Vec<ColumnDefinition>,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub charset: String,
    #[serde(default)]
    pub collation: String,
    #[serde(default)]
    pub comment: String,
}

/// 列定义
#[derive(Deserialize, Clone, Debug)]
pub struct ColumnDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    #[serde(default)]
    pub length: Option<String>,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    #[serde(alias = "primaryKey")]
    pub primary_key: bool,
    #[serde(default)]
    #[serde(alias = "autoIncrement")]
    pub auto_increment: bool,
    #[serde(default)]
    #[serde(alias = "defaultValue")]
    pub default_value: Option<String>,
    #[serde(default)]
    pub comment: String,
}

/// Schema 树节点
#[derive(Serialize, Deserialize, Clone)]
pub struct SchemaNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Vec<SchemaNode>,
}

/// 创建数据库
#[tauri::command]
pub async fn meta_create_database(params: CreateDatabaseParams) -> Result<String, String> {
    let opts = build_opts(&params.connection)?;
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| e.to_string())?;

    let charset_part = if !params.charset.is_empty() {
        format!("CHARACTER SET {}", params.charset)
    } else {
        String::new()
    };

    let collation_part = if !params.collation.is_empty() {
        format!("COLLATE {}", params.collation)
    } else {
        String::new()
    };

    let sql = format!(
        "CREATE DATABASE `{}` {} {}",
        escape_sql(&params.name),
        charset_part,
        collation_part
    );

    conn.query_drop(&sql)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("数据库 '{}' 创建成功", params.name))
}

/// 创建表
#[tauri::command]
pub async fn meta_create_table(params: CreateTableParams) -> Result<String, String> {
    let opts = build_opts(&params.connection)?;
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| e.to_string())?;

    // 收集主键列名（自增列必须作为主键）
    let mut primary_key_names: Vec<String> = Vec::new();
    for col in &params.columns {
        if col.auto_increment || col.primary_key {
            primary_key_names.push(col.name.clone());
        }
    }

    // 构建列定义
    let column_defs: Vec<String> = params.columns
        .iter()
        .map(|col| {
            let mut def = format!("`{}` {}", col.name, col.col_type);

            if let Some(len) = &col.length {
                if !len.is_empty() {
                    def = format!("{}({})", def, len);
                }
            }

            // 自增列或主键列必须 NOT NULL
            if col.auto_increment || col.primary_key || !col.nullable {
                def.push_str(" NOT NULL");
            }

            // AUTO_INCREMENT（单独定义，PRIMARY KEY 在表级定义）
            if col.auto_increment {
                def.push_str(" AUTO_INCREMENT");
            }

            if let Some(default_val) = &col.default_value {
                if !default_val.is_empty() {
                    def = format!("{} DEFAULT {}", def, quote_sql_value(default_val));
                }
            }

            if !col.comment.is_empty() {
                // 转义注释中的单引号
                let escaped_comment = col.comment.replace('\'', "''");
                def = format!("{} COMMENT '{}'", def, escaped_comment);
            }

            Ok(def)
        })
        .collect::<Result<Vec<_>, String>>()?;

    // 构建主键定义（在表级定义，包含所有主键列）
    let pk_def = if !primary_key_names.is_empty() {
        format!(", PRIMARY KEY ({})", primary_key_names.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))
    } else {
        String::new()
    };
    // 构建表选项
    let engine_part = if !params.engine.is_empty() {
        format!("ENGINE={}", params.engine)
    } else {
        String::from("ENGINE=InnoDB")
    };

    let charset_part = if !params.charset.is_empty() {
        format!("CHARACTER SET {}", params.charset)
    } else {
        String::from("CHARACTER SET=utf8mb4")
    };

    let collation_part = if !params.collation.is_empty() {
        format!("COLLATE {}", params.collation)
    } else {
        String::from("COLLATE=utf8mb4_unicode_ci")
    };

    let comment_part = if !params.comment.is_empty() {
        format!("COMMENT='{}'", params.comment)
    } else {
        String::new()
    };

    let sql = format!(
        "CREATE TABLE `{}`.`{}` ({}{}) {} {} {}",
        params.database,
        params.table,
        column_defs.join(", "),
        pk_def,
        engine_part,
        charset_part,
        collation_part
    );

    let final_sql = if !comment_part.is_empty() {
        format!("{} {}", sql, comment_part)
    } else {
        sql
    };

    conn.query_drop(&final_sql)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("表 '{}.{}' 创建成功", params.database, params.table))
}

/// SQL 值引用
fn quote_sql_value(value: &str) -> String {
    if value.eq_ignore_ascii_case("NULL") {
        return "NULL".to_string();
    }
    if value.eq_ignore_ascii_case("CURRENT_TIMESTAMP") {
        return value.to_uppercase();
    }
    format!("'{}'", value.replace('\'', "''").replace('\\', "\\\\"))
}

/// 获取数据库列表
#[tauri::command]
pub async fn meta_list_databases(connection: ConnectionInfo) -> Result<Vec<String>, String> {
    let opts = build_opts(&connection)?;
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| e.to_string())?;

    let databases: Vec<String> = conn
        .query("SELECT SCHEMA_NAME FROM information_schema.SCHEMATA ORDER BY SCHEMA_NAME")
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .filter_map(|r: &Row| r.get::<Option<String>, _>("SCHEMA_NAME").flatten())
        .collect();

    Ok(databases)
}

/// 获取表列表
#[tauri::command]
pub async fn meta_list_tables(
    connection: ConnectionInfo,
    database: String,
    include_views: bool,
) -> Result<Vec<TableInfo>, String> {
    let opts = build_opts(&connection)?;
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| e.to_string())?;

    let sql = format!(
        "SELECT TABLE_NAME, TABLE_TYPE, IFNULL(TABLE_COMMENT,'') AS TABLE_COMMENT,
                IFNULL(ENGINE,'') AS ENGINE, IFNULL(TABLE_ROWS,0) AS ROWS_EST
         FROM information_schema.TABLES
         WHERE TABLE_SCHEMA = '{}'
         ORDER BY TABLE_NAME",
        escape_sql(&database)
    );

    let mut tables: Vec<TableInfo> = conn
        .query::<Row, _>(&*sql)
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .map(|r: &Row| TableInfo {
            name: r.get::<Option<String>, _>("TABLE_NAME").flatten().unwrap_or_default(),
            table_type: r.get::<Option<String>, _>("TABLE_TYPE").flatten().unwrap_or_default(),
            comment: r.get::<Option<String>, _>("TABLE_COMMENT").flatten(),
            engine: r.get::<Option<String>, _>("ENGINE").flatten(),
            rows_est: r.get::<Option<u64>, _>("ROWS_EST").flatten().unwrap_or(0),
        })
        .collect();

    if !include_views {
        tables.retain(|table| table.table_type.to_uppercase() != "VIEW");
    }

    Ok(tables)
}

/// 获取表结构
#[tauri::command]
pub async fn meta_get_table_schema(
    connection: ConnectionInfo,
    database: String,
    table: String,
) -> Result<TableSchema, String> {
    let opts = build_opts(&connection)?;
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| e.to_string())?;

    // 获取列信息
    let sql = format!(
        "SELECT ORDINAL_POSITION, COLUMN_NAME, COLUMN_TYPE, DATA_TYPE, IS_NULLABLE,
                COLUMN_DEFAULT, EXTRA, IFNULL(COLUMN_COMMENT,'') AS COLUMN_COMMENT
         FROM information_schema.COLUMNS
         WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}'
         ORDER BY ORDINAL_POSITION",
        escape_sql(&database),
        escape_sql(&table)
    );

    let columns: Vec<Column> = conn
        .query::<Row, _>(&*sql)
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .map(|r: &Row| Column {
            name: r.get::<Option<String>, _>("COLUMN_NAME").flatten().unwrap_or_default(),
            column_type: r.get::<Option<String>, _>("COLUMN_TYPE").flatten().unwrap_or_default(),
            nullable: r.get::<Option<String>, _>("IS_NULLABLE").flatten().as_deref() == Some("YES"),
            default: r.get::<Option<String>, _>("COLUMN_DEFAULT").flatten(),
            comment: r.get::<Option<String>, _>("COLUMN_COMMENT").flatten(),
            extra: r.get::<Option<String>, _>("EXTRA").flatten(),
        })
        .collect();

    // 获取索引信息
    let sql = format!(
        "SELECT INDEX_NAME, NON_UNIQUE, COLUMN_NAME FROM information_schema.STATISTICS
         WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' ORDER BY INDEX_NAME, SEQ_IN_INDEX",
        escape_sql(&database),
        escape_sql(&table)
    );

    let index_rows: Vec<(String, bool, String)> = conn
        .query::<Row, _>(&*sql)
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .map(|r: &Row| {
            (
                r.get::<Option<String>, _>("INDEX_NAME").flatten().unwrap_or_default(),
                r.get::<Option<u8>, _>("NON_UNIQUE").flatten() == Some(0),
                r.get::<Option<String>, _>("COLUMN_NAME").flatten().unwrap_or_default(),
            )
        })
        .collect();

    let mut indexes: Vec<Index> = Vec::new();
    for (index_name, unique, column) in index_rows {
        if let Some(idx) = indexes.iter_mut().find(|i| i.name == index_name) {
            idx.columns.push(column);
        } else {
            indexes.push(Index {
                name: index_name,
                unique,
                columns: vec![column],
            });
        }
    }

    // 获取建表 SQL
    let sql = format!("SHOW CREATE TABLE `{}`.`{}`", database, table);
    let create_sql: Option<String> = conn
        .query::<Row, _>(&*sql)
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .next()
        .and_then(|r: &Row| r.get::<Option<String>, _>("Create Table").flatten());

    Ok(TableSchema {
        database,
        table,
        columns,
        indexes,
        foreign_keys: Vec::new(),
        create_sql,
    })
}

/// 获取 Schema 树
#[tauri::command]
pub async fn meta_get_schema_tree(connection: ConnectionInfo) -> Result<Vec<SchemaNode>, String> {
    let opts = build_opts(&connection)?;
    let mut conn = mysql_async::Conn::new(opts).await
        .map_err(|e| e.to_string())?;

    // 获取所有数据库
    let databases: Vec<String> = conn
        .query("SELECT SCHEMA_NAME FROM information_schema.SCHEMATA ORDER BY SCHEMA_NAME")
        .await
        .map_err(|e| e.to_string())?
        .iter()
        .filter_map(|r: &Row| r.get::<Option<String>, _>("SCHEMA_NAME").flatten())
        .collect();

    let mut root_nodes = Vec::new();

    for db in databases {
        // 获取该数据库的表
        let sql = format!(
            "SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = '{}' ORDER BY TABLE_NAME",
            escape_sql(&db)
        );

        let tables: Vec<String> = conn
            .query::<Row, _>(&*sql)
            .await
            .map_err(|e| e.to_string())?
            .iter()
            .filter_map(|r: &Row| r.get::<Option<String>, _>("TABLE_NAME").flatten())
            .collect();

        let children: Vec<SchemaNode> = tables
            .into_iter()
            .map(|t| SchemaNode {
                id: format!("{}.{}", db, t),
                name: t,
                node_type: "table".to_string(),
                children: vec![],
            })
            .collect();

        root_nodes.push(SchemaNode {
            id: db.clone(),
            name: db,
            node_type: "database".to_string(),
            children,
        });
    }

    Ok(root_nodes)
}

fn escape_sql(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "''")
}

fn build_opts(conn: &ConnectionInfo) -> Result<Opts, String> {
    Opts::try_from(
        mysql_async::OptsBuilder::default()
            .ip_or_hostname(conn.host.clone())
            .tcp_port(conn.port)
            .user(Some(conn.user.clone()))
            .pass(Some(conn.password.clone()))
            .db_name(conn.default_db.clone()),
    )
    .map_err(|e| format!("无效的连接参数: {}", e))
}
