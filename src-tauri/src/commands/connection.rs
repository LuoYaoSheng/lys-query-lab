// Connection management commands

use crate::db::ConnectionInfo;
use crate::storage::ConnectionStorage;
use anyhow::Result;
use mysql_async::prelude::*;
use mysql_async::Opts;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

/// 连接列表
#[tauri::command]
pub fn conn_list() -> Result<Vec<ConnectionInfo>, String> {
    let storage = ConnectionStorage::default();
    storage.load_all()
        .map_err(|e| e.to_string())
}

/// 新建或更新连接
#[tauri::command]
pub fn conn_upsert(mut connection: ConnectionInfo) -> Result<String, String> {
    let storage = ConnectionStorage::default();

    // 如果没有 ID，生成新 ID
    if connection.id.is_empty() {
        connection.id = Uuid::new_v4().to_string();
    }

    storage.upsert(&connection)
        .map_err(|e| e.to_string())?;
    Ok(connection.id)
}

/// 删除连接
#[tauri::command]
pub fn conn_delete(id: String) -> Result<bool, String> {
    let storage = ConnectionStorage::default();
    storage.delete(&id)
        .map_err(|e| e.to_string())?;
    Ok(true)
}

/// 测试连接结果
#[derive(Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub latency_ms: u64,
    pub server_version: String,
    pub user: String,
    pub default_db: Option<String>,
}

/// 测试连接
#[tauri::command]
pub async fn conn_test(connection: ConnectionInfo) -> Result<ConnectionTestResult, String> {
    let opts = build_mysql_opts(&connection)?;

    let start = Instant::now();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        test_mysql_connection(opts),
    )
    .await
    .map_err(|_| anyhow::anyhow!("连接超时（5秒）").to_string())?
    .map_err(|e| anyhow::anyhow!("连接失败: {}", e).to_string())?;

    let latency = start.elapsed().as_millis() as u64;

    Ok(ConnectionTestResult {
        latency_ms: latency,
        server_version: result.0,
        user: result.1,
        default_db: result.2,
    })
}

fn build_mysql_opts(conn: &ConnectionInfo) -> Result<Opts, String> {
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

async fn test_mysql_connection(opts: Opts) -> Result<(String, String, Option<String>), mysql_async::Error> {
    let mut conn = mysql_async::Conn::new(opts).await?;

    // 获取版本
    let version: String = conn
        .query("SELECT VERSION() AS v")
        .await?
        .iter()
        .next()
        .and_then(|r: &mysql_async::Row| r.get("v"))
        .unwrap_or_default();

    // 获取当前用户
    let user: String = conn
        .query("SELECT USER() AS u")
        .await?
        .iter()
        .next()
        .and_then(|r: &mysql_async::Row| r.get("u"))
        .unwrap_or_default();
    // 清理用户名格式 (root@% -> root)
    let user = user.split('@').next().unwrap_or(&user).to_string();

    // 获取当前数据库
    let db: Option<String> = conn
        .query("SELECT DATABASE() AS d")
        .await?
        .iter()
        .next()
        .and_then(|r: &mysql_async::Row| r.get::<Option<String>, _>("d"))
        .flatten();

    conn.disconnect().await?;

    Ok((version, user, db))
}
