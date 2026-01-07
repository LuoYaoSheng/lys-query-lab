// App commands - 版本、环境、健康检查

use serde::Serialize;

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
    pub platform: String,
    #[serde(rename = "build")]
    pub build_type: String,
}

#[tauri::command]
pub fn app_get_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        build_type: if cfg!(debug_assertions) {
            "dev".to_string()
        } else {
            "prod".to_string()
        },
    }
}

/// 统一返回结构
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(code: &str, message: &str) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                detail: None,
            }),
        }
    }
}
