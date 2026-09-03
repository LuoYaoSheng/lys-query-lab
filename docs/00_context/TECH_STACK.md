# QueryLab 技术栈清单（TECH_STACK）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 权威来源：根目录 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`vite.config.js`、`src/main.js`、`src-tauri/src/main.rs`。版本号为声明值（^ 语义），实际锁定以 `package-lock.json` / `src-tauri/Cargo.lock` 为准【未逐项核对锁定版本】。

---

## 1. 总览

| 层 | 技术 | 声明版本 | 证据 |
|----|------|----------|------|
| 桌面壳 | Tauri 2 | CLI ^2.9.6（npm）/ 2（Rust crate） | `package.json` devDependencies、`src-tauri/Cargo.toml` dependencies |
| 前端框架 | Svelte（传统模式 + mount API） | ^5.43.8 | `package.json`、`src/main.js` |
| 构建工具 | Vite | ^7.2.4 | `package.json`、`vite.config.js` |
| 前端测试 | Vitest + @testing-library/svelte + jsdom | ^3.2.4 / ^5.3.1 / ^26.1.0 | `package.json` devDependencies、`src/test/setup.js` |
| SQL 编辑器 | CodeMirror 6（全家桶） | ^6.x（7 个子包） | `package.json` dependencies、`src/components/SqlEditor.svelte` |
| 后端语言 | Rust（edition 2021） | — | `src-tauri/Cargo.toml` |
| 异步运行时 | tokio（features = full） | 1 | `src-tauri/Cargo.toml` |
| 数据库驱动 | mysql_async（仅 MySQL） | 0.34 | `src-tauri/Cargo.toml` |
| 密钥存储 | keyring（系统钥匙串） | 1.1.0 | `src-tauri/Cargo.toml`、`src-tauri/src/security/mod.rs` |
| 序列化 | serde + serde_json | 1.0 | `src-tauri/Cargo.toml` |
| 本地配置目录 | dirs | 5 | `src-tauri/src/storage/connections.rs` |
| 标识/时间 | uuid（v4+serde）/ chrono | 1.11 / 0.4 | `src-tauri/Cargo.toml` |
| 错误处理 | thiserror 2 + anyhow 1 + async-trait 0.1 | — | `src-tauri/Cargo.toml`、`src-tauri/src/db/driver.rs` |
| Tauri 插件（Rust 侧已注册） | tauri-plugin-shell | 2 | `src-tauri/Cargo.toml`、`src-tauri/src/main.rs` L20、`tauri.conf.json` plugins.shell.open=true |

## 2. 前端依赖明细（源：`package.json`）

dependencies（运行时）：

| 包 | 版本 | 用途 | 消费处 |
|----|------|------|--------|
| @codemirror/autocomplete | ^6.20.0 | SQL 自动补全（F020） | `src/components/SqlEditor.svelte` |
| @codemirror/commands | ^6.10.1 | 编辑器命令（默认键位） | 同上 |
| @codemirror/lang-sql | ^6.10.0 | SQL 语法/方言 | 同上 |
| @codemirror/search | ^6.5.11 | 编辑器内搜索（F021） | 同上 |
| @codemirror/state | ^6.5.3 | 编辑器状态 | 同上 |
| @codemirror/theme-one-dark | ^6.1.3 | 深色主题 | 同上 |
| @codemirror/view | ^6.39.9 | 编辑器视图 | 同上 |
| codemirror | ^6.0.2 | CM6 元包 | 同上 |
| @tauri-apps/api | ^2.10.1 | invoke IPC | `src/App.svelte` 及 8 个组件 |
| @tauri-apps/plugin-dialog | ^2.4.2 | 原生保存/打开对话框 | `DatabaseBackup.svelte`（save+open）、`ResultsPanel.svelte`（save）、`DataGrid.svelte`（save）——⚠️ 见 §4 断裂说明 |
| — | — | — | — |

devDependencies（构建/测试）：@testing-library/jest-dom ^6.9.1、@testing-library/svelte ^5.3.1、@sveltejs/vite-plugin-svelte ^6.2.1、@tauri-apps/cli ^2.9.6、jsdom ^26.1.0、svelte ^5.43.8、vite ^7.2.4、vitest ^3.2.4。

## 3. Rust 依赖明细（源：`src-tauri/Cargo.toml`）

| crate | 版本 | 用途 | 消费处 |
|-------|------|------|--------|
| tauri | 2 | 桌面壳/命令框架 | 全后端 |
| tauri-plugin-shell | 2 | open 等系统能力 | `main.rs` 注册；**前端零消费**（PERMISSION_REVIEW PM-03） |
| serde / serde_json | 1.0 | 命令参数/返回序列化 | 全后端 |
| tokio | 1（full） | 异步运行时 | `commands/connection.rs`（conn_test 5s 超时）等 |
| async-trait | 0.1 | Driver/DbConnection trait 异步方法 | `db/driver.rs` |
| thiserror | 2 | DbError 错误枚举 | `db/driver.rs` |
| anyhow | 1 | 命令层错误传播 | 全后端 |
| keyring | 1.1.0 | 系统钥匙串密码存取 | `security/mod.rs` |
| mysql_async | 0.34 | MySQL 连接/查询 | `commands/connection.rs`、`commands/query.rs`、`commands/metadata.rs`、`commands/backup.rs` |
| uuid | 1.11（v4+serde） | 连接 id 生成 | `commands/connection.rs` conn_upsert |
| chrono | 0.4 | 时间处理 | 声明于 Cargo.toml；直接消费点【未知——grep 未见显式 use，可能经 mysql_async 间接使用】 |
| dirs | 5 | 跨平台配置目录 | `storage/connections.rs` |

build-dependencies：tauri-build 2（`src-tauri/build.rs`）。

## 4. 已知断裂与缺陷（技术栈视角）

1. **plugin-dialog 前后端断裂（PL-02，B 级）**：前端 `package.json` 有 `@tauri-apps/plugin-dialog` ^2.4.2 且 3 个组件 import save/open；但 Rust 侧 `src-tauri/Cargo.toml` **无 tauri-plugin-dialog crate**、`src-tauri/src/main.rs` 仅注册 `tauri_plugin_shell`、`src-tauri/` 下**无 capabilities/ 目录**（仅 gen/schemas 生成物）。后果：打包态 `save()`/`open()` 必然 reject——导出全部落入浏览器降级下载（落点不可控），备份导入文件选择完全不可用（「开始导入」按钮永久禁用）。来源：`docs/product-review/PRODUCT_LOGIC_REVIEW.md` PL-02、`docs/product-review/PERMISSION_REVIEW.md` §1.2。
2. **tauri-plugin-shell 注册但零消费（PM-03，C 级）**：`main.rs` L20 注册 + `tauri.conf.json` `shell.open:true`，全局 grep 无前端调用。冗余能力面，处置待用户决策。
3. **未启用 Tauri 2 capabilities 显式授权模型（PM-07）**：无 `src-tauri/capabilities/` 目录，插件/命令授权无显式载体。
4. **无 CSP（PM-01 关联）**：`tauri.conf.json` 无 `security` 节。
5. **Driver 抽象空壳（PL-05/IA-05）**：`db/driver.rs` 定义 Driver/Capabilities trait（supports_explain/procedures/ssh_tunnel/returning/limit_offset），无任何实现方；方言逻辑实际散落在视图组件与 meta 命令中（硬编码 information_schema、MySQL 引擎/字符集、LIMIT 语法）。
6. `withGlobalTauri: false`（`tauri.conf.json` L13）为正面配置，缩小注入面（PERMISSION_REVIEW §1.2）。

## 5. 关联阅读

- 依赖用途与死依赖标注：`docs/00_context/DEPENDENCY_LIST.md`
- 权限/能力面规格：`docs/08_development/PERMISSION.md`
- 架构分层：`docs/04_architecture/SYSTEM_ARCH.md`
