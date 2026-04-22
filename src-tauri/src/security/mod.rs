// Security - Keychain integration for connection secrets

use anyhow::{Context, Result};

const CONNECTION_PASSWORD_SERVICE: &str = "com.i2kai.querylab.connection";

fn entry_for(connection_id: &str) -> keyring::Entry {
    keyring::Entry::new(CONNECTION_PASSWORD_SERVICE, connection_id)
}

pub fn set_connection_password(connection_id: &str, password: &str) -> Result<()> {
    let entry = entry_for(connection_id);
    entry
        .set_password(password)
        .context("无法将连接密码写入系统钥匙串")
}

pub fn get_connection_password(connection_id: &str) -> Result<Option<String>> {
    let entry = entry_for(connection_id);
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(anyhow::Error::new(err).context("无法从系统钥匙串读取连接密码")),
    }
}

pub fn delete_connection_password(connection_id: &str) -> Result<()> {
    let entry = entry_for(connection_id);
    match entry.delete_password() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(anyhow::Error::new(err).context("无法从系统钥匙串删除连接密码")),
    }
}
