// 连接配置存储

use crate::db::ConnectionInfo;
use crate::security::{
    delete_connection_password, get_connection_password, set_connection_password,
};
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
        let mut connections: Vec<ConnectionInfo> = serde_json::from_str(&content)
            .unwrap_or_default();

        let mut migrated_plaintext = false;

        for connection in &mut connections {
            if !connection.password.is_empty() {
                set_connection_password(&connection.id, &connection.password)?;
                connection.password.clear();
                migrated_plaintext = true;
            }
        }

        if migrated_plaintext {
            self.save_all(&connections)?;
        }

        for connection in &mut connections {
            connection.password = get_connection_password(&connection.id)?.unwrap_or_default();
        }

        Ok(connections)
    }

    /// 保存所有连接
    pub fn save_all(&self, connections: &[ConnectionInfo]) -> Result<()> {
        let sanitized: Vec<ConnectionInfo> = connections
            .iter()
            .cloned()
            .map(|mut connection| {
                connection.password.clear();
                connection
            })
            .collect();

        let content = serde_json::to_string_pretty(&sanitized)?;
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// 添加或更新单个连接
    pub fn upsert(&self, connection: &ConnectionInfo) -> Result<()> {
        if connection.password.is_empty() {
            delete_connection_password(&connection.id)?;
        } else {
            set_connection_password(&connection.id, &connection.password)?;
        }

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
        delete_connection_password(id)?;

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

#[cfg(test)]
mod tests {
    use super::ConnectionStorage;
    use crate::db::ConnectionInfo;
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn temp_file_path() -> PathBuf {
        std::env::temp_dir().join(format!("querylab-connections-{}.json", Uuid::new_v4()))
    }

    #[test]
    fn save_all_does_not_persist_password_field() -> Result<()> {
        let path = temp_file_path();
        let storage = ConnectionStorage { file_path: path.clone() };

        let connections = vec![ConnectionInfo {
            id: "conn-1".into(),
            name: "Local".into(),
            driver_type: "mysql".into(),
            host: "localhost".into(),
            port: 3306,
            user: "root".into(),
            password: "secret-password".into(),
            default_db: Some("demo".into()),
        }];

        storage.save_all(&connections)?;

        let saved = fs::read_to_string(&path)?;
        assert!(!saved.contains("secret-password"));
        assert!(saved.contains("\"host\": \"localhost\""));

        let _ = fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn save_and_load_roundtrip_keeps_connection_metadata() -> Result<()> {
        let path = temp_file_path();
        let storage = ConnectionStorage { file_path: path.clone() };

        let connection = ConnectionInfo {
            id: "conn-roundtrip".into(),
            name: "Primary DB".into(),
            driver_type: "mysql".into(),
            host: "db.internal".into(),
            port: 3306,
            user: "admin".into(),
            password: "".into(),
            default_db: Some("app".into()),
        };

        storage.upsert(&connection)?;
        let loaded = storage.load_all()?;

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "conn-roundtrip");
        assert_eq!(loaded[0].name, "Primary DB");
        assert_eq!(loaded[0].host, "db.internal");
        assert_eq!(loaded[0].default_db.as_deref(), Some("app"));

        let _ = fs::remove_file(path);
        Ok(())
    }
}
