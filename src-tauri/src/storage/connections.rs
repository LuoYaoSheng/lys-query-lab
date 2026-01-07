// 连接配置存储

use crate::db::ConnectionInfo;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct ConnectionStorage {
    file_path: PathBuf,
}

impl ConnectionStorage {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
            .join("querylab");

        fs::create_dir_all(&config_dir)?;

        let file_path = config_dir.join("connections.json");
        Ok(Self { file_path })
    }

    /// 读取所有连接
    pub fn load_all(&self) -> Result<Vec<ConnectionInfo>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)?;
        let connections: Vec<ConnectionInfo> = serde_json::from_str(&content)
            .unwrap_or_default();

        Ok(connections)
    }

    /// 保存所有连接
    pub fn save_all(&self, connections: &[ConnectionInfo]) -> Result<()> {
        let content = serde_json::to_string_pretty(connections)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// 添加或更新单个连接
    pub fn upsert(&self, connection: &ConnectionInfo) -> Result<()> {
        let mut connections = self.load_all()?;
        if let Some(pos) = connections.iter().position(|c| c.id == connection.id) {
            connections[pos] = connection.clone();
        } else {
            connections.push(connection.clone());
        }
        self.save_all(&connections)?;
        Ok(())
    }

    /// 删除连接
    pub fn delete(&self, id: &str) -> Result<()> {
        let mut connections = self.load_all()?;
        connections.retain(|c| c.id != id);
        self.save_all(&connections)?;
        Ok(())
    }
}

impl Default for ConnectionStorage {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                file_path: PathBuf::from("connections.json"),
            }
        })
    }
}
