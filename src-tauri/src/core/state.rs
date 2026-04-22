#![allow(dead_code)]

// 全局状态管理 - 连接池、会话、任务管理

use std::sync::Arc;
use tokio::sync::RwLock;

/// 应用全局状态
#[derive(Clone)]
pub struct AppState {
    /// 连接管理器
    pub conn_manager: Arc<RwLock<ConnManager>>,
    /// 会话管理器
    pub session_manager: Arc<RwLock<SessionManager>>,
    /// 任务管理器（长查询、导入导出）
    pub task_manager: Arc<RwLock<TaskManager>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            conn_manager: Arc::new(RwLock::new(ConnManager::new())),
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
            task_manager: Arc::new(RwLock::new(TaskManager::new())),
        }
    }
}

/// 连接管理器
pub struct ConnManager {
    // TODO: 存储连接配置和运行时连接
}

impl ConnManager {
    pub fn new() -> Self {
        Self {}
    }
}

/// 会话管理器（一个连接下多个 tab session）
pub struct SessionManager {
    // TODO: 管理 SQL 编辑器会话
}

impl SessionManager {
    pub fn new() -> Self {
        Self {}
    }
}

/// 任务管理器（异步任务 + cancel token）
pub struct TaskManager {
    // TODO: 管理长任务，支持取消
}

impl TaskManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
