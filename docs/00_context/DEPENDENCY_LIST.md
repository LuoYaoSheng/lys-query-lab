# QueryLab 依赖清单（DEPENDENCY_LIST）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 来源：`package.json`、`src-tauri/Cargo.toml`、全局 grep 消费点核实、`docs/product-review/PERMISSION_REVIEW.md` §1、`docs/product-review/PRODUCT_LOGIC_REVIEW.md`。
> 「消费状态」口径：活跃（有调用方）/ 死依赖（无调用方）/ 断裂（声明但链路不可用）。

---

## 1. 前端 dependencies（10 项，源 `package.json`）

| 包 | 版本 | 用途 | 消费状态 | 消费点 |
|----|------|------|----------|--------|
| @tauri-apps/api | ^2.10.1 | invoke IPC 通道 | 活跃 | `src/App.svelte` L3 及 8 个组件（invoke 调用共 30+ 处，grep 计数） |
| @tauri-apps/plugin-dialog | ^2.4.2 | 原生保存/打开对话框 | **断裂**（详见 §3） | `DatabaseBackup.svelte` L3（save+open）、`ResultsPanel.svelte` L3（save）、`DataGrid.svelte` L3（save）及 2 个 .test.js 的 vi.mock |
| codemirror | ^6.0.2 | CM6 元包 | 活跃 | SqlEditor.svelte |
| @codemirror/lang-sql | ^6.10.0 | SQL 语法/方言 | 活跃 | SqlEditor.svelte（方言检测字段错配缺陷见 PRODUCT_REVIEW PF-05） |
| @codemirror/autocomplete | ^6.20.0 | 补全（F020） | 活跃 | SqlEditor.svelte（表名源未接线/列名缺失，PF-08） |
| @codemirror/commands | ^6.10.1 | 默认键位命令 | 活跃 | SqlEditor.svelte |
| @codemirror/search | ^6.5.11 | 编辑器搜索（F021） | 活跃 | SqlEditor.svelte |
| @codemirror/state | ^6.5.3 | 编辑器状态 | 活跃 | SqlEditor.svelte |
| @codemirror/view | ^6.39.9 | 编辑器视图 | 活跃 | SqlEditor.svelte |
| @codemirror/theme-one-dark | ^6.1.3 | 深色主题 | 活跃 | SqlEditor.svelte |

## 2. 前端 devDependencies（8 项，源 `package.json`）

| 包 | 版本 | 用途 | 消费状态 |
|----|------|------|----------|
| svelte | ^5.43.8 | UI 框架 | 活跃（全部 .svelte） |
| vite | ^7.2.4 | 构建 | 活跃（`vite.config.js`） |
| @sveltejs/vite-plugin-svelte | ^6.2.1 | Vite 的 Svelte 插件 | 活跃（`vite.config.js`） |
| vitest | ^3.2.4 | 测试运行器 | 活跃（3 个 .test.js + `src/test/setup.js`） |
| @testing-library/svelte | ^5.3.1 | 组件测试 | 活跃（.test.js） |
| @testing-library/jest-dom | ^6.9.1 | DOM 断言 | 活跃（`src/test/setup.js` 引入） |
| jsdom | ^26.1.0 | 测试 DOM 环境 | 活跃（vitest environment） |
| @tauri-apps/cli | ^2.9.6 | tauri 命令行 | 活跃（scripts: tauri:dev/tauri:build） |

## 3. Rust dependencies（12 项，源 `src-tauri/Cargo.toml`）

| crate | 版本 | 用途 | 消费状态 | 消费点 |
|-------|------|------|----------|--------|
| tauri | 2 | 桌面壳/命令 | 活跃 | main.rs、全部 commands |
| tauri-plugin-shell | 2 | shell.open 系统能力 | **死依赖（Rust 侧注册但前端零消费）** | main.rs L20 注册；前端全局 grep 无调用（PERMISSION_REVIEW PM-03） |
| serde | 1.0（derive） | 序列化派生 | 活跃 | db/types.rs、db/driver.rs、commands/* |
| serde_json | 1.0 | JSON 读写 | 活跃 | storage/connections.rs 等 |
| tokio | 1（full） | 异步运行时/超时 | 活跃 | commands/connection.rs（conn_test 5s 超时）、main 隐含 |
| async-trait | 0.1 | trait 异步方法 | 活跃（但仅服务于空壳抽象） | db/driver.rs Driver/DbConnection |
| thiserror | 2 | 错误派生 | 活跃 | db/driver.rs DbError |
| anyhow | 1 | 错误传播 | 活跃 | storage、security、commands |
| keyring | 1.1.0 | 系统钥匙串 | 活跃 | security/mod.rs（set/get/delete_connection_password） |
| mysql_async | 0.34 | MySQL 驱动 | 活跃 | commands/{connection,query,metadata,backup}.rs |
| uuid | 1.11（v4+serde） | 连接 id | 活跃 | commands/connection.rs conn_upsert |
| chrono | 0.4 | 时间 | **疑似死依赖** | Cargo.toml 声明；直接 `use chrono` 消费点未检出【未知——可能经 mysql_async 间接使用，需 cargo-udeps/机器验证】 |
| dirs | 5 | 配置目录 | 活跃 | storage/connections.rs |

build-dependencies：tauri-build 2（活跃，`src-tauri/build.rs`）。

## 4. 缺失依赖（前端有、Rust 侧缺）

| 缺失项 | 证据 | 后果 | 关联问题编号 |
|--------|------|------|--------------|
| tauri-plugin-dialog（Rust crate） | 前端 package.json 有 @tauri-apps/plugin-dialog；Cargo.toml 无对应 crate；main.rs 未注册；无 capabilities/ 目录 | 打包态 save()/open() 必然 reject：导出走浏览器降级下载（落点不可控）；备份导入文件选择完全不可用 | PL-02（B 级，重构必办）；PERMISSION_REVIEW §1.2 |
| capabilities/ 声明（ACL） | src-tauri/ 下仅 gen/schemas 生成物 | 未启用显式能力授权模型；引入 dialog/fs 插件时需同步建立 | PM-07（C 级） |
| CSP 配置 | tauri.conf.json 无 security 节 | 默认无内容安全策略 | PM-01 关联 |

## 5. 死代码资产（非 npm/cargo 依赖，随依赖盘点一并登记）

| 项 | 位置 | 状态 | 处置（`docs/04_architecture/MODULE_ARCH.md` §4） |
|----|------|------|------|
| Counter.svelte | `src/lib/Counter.svelte` | 模板残留，无引用 | 建议删除 |
| core/state.rs、core/errors.rs | `src-tauri/src/core/` | `#![allow(dead_code)]` 占位（errors.rs 定义 AppError 但命令层实际返回 String） | 启用（错误码规范见 `docs/08_development/ERROR_CODE.md`）或删除 |
| util/mod.rs | `src-tauri/src/util/` | 占位 | 同上 |
| db/driver.rs Driver/Capabilities | `src-tauri/src/db/driver.rs` | trait 空壳无实现方 | 启用（多库扩展）或删除（PL-05/IA-05） |
| meta_get_schema_tree | `src-tauri/src/commands/metadata.rs` L375 | 后端命令已注册，前端零调用（前端 4 处调用为 meta_list_tables 等其他命令） | C8 待用户确认剔除 |
| src-ui/ 整目录 | `src-ui/` | 旧副本（含独立 .git） | C9 待用户确认删除；注意其预填密码 'root123456' 是 PL-01 被掩盖的环境原因（留档） |

## 6. 关联阅读

- 技术栈版本明细：`docs/00_context/TECH_STACK.md`
- 权限/能力面目标态：`docs/08_development/PERMISSION.md`
- 模块处置清单：`docs/04_architecture/MODULE_ARCH.md`
