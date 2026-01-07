// 统一错误码/错误映射

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl AppError {
    // 数据库相关错误
    pub fn db_connection_failed(msg: impl Into<String>) -> Self {
        Self {
            code: "DB_CONN_FAILED".to_string(),
            message: msg.into(),
            detail: None,
        }
    }

    pub fn db_query_failed(msg: impl Into<String>) -> Self {
        Self {
            code: "DB_QUERY_FAILED".to_string(),
            message: msg.into(),
            detail: None,
        }
    }

    // 连接管理错误
    pub fn conn_not_found(id: impl Into<String>) -> Self {
        Self {
            code: "CONN_NOT_FOUND".to_string(),
            message: format!("连接不存在: {}", id.into()),
            detail: None,
        }
    }

    // 参数错误
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: "INVALID_PARAMS".to_string(),
            message: msg.into(),
            detail: None,
        }
    }

    // 未授权/VIP 功能
    pub fn feature_required(feature: impl Into<String>) -> Self {
        Self {
            code: "FEATURE_REQUIRED".to_string(),
            message: format!("需要 VIP 权限: {}", feature.into()),
            detail: None,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

// 从其他错误类型转换
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: err.to_string(),
            detail: None,
        }
    }
}
