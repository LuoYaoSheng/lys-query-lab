// Metadata commands - Schema 浏览

use crate::db::{Column, Index, TableInfo, TableSchema};
use crate::db::ConnectionInfo;
use mysql_async::prelude::*;
use mysql_async::{Opts, Row};
use serde::{Deserialize, Serialize};

/// Schema 树节点
#[derive(Serialize, Deserialize, Clone)]
pub struct SchemaNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub children: Vec<SchemaNode>,
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
    _include_views: bool,
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

    let tables: Vec<TableInfo> = conn
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
