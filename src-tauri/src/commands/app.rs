// App commands - 版本、环境、健康检查

use serde::Serialize;
use std::fs;
use std::path::Path;

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

#[tauri::command]
pub fn fs_write_file(path: String, contents: String) -> Result<bool, String> {
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    fs::write(&path, contents).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(true)
}
