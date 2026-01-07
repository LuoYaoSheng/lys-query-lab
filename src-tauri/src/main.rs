// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod db;
mod storage;
mod security;
mod util;

use commands::{
    app_get_info, conn_delete, conn_list, conn_test, conn_upsert,
    meta_get_schema_tree, meta_get_table_schema, meta_list_databases, meta_list_tables,
    query_execute,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            app_get_info,
            conn_list,
            conn_upsert,
            conn_delete,
            conn_test,
            meta_list_databases,
            meta_list_tables,
            meta_get_table_schema,
            meta_get_schema_tree,
            query_execute,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
