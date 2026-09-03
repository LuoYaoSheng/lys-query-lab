# QueryLab 项目上下文（PROJECT_CONTEXT）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 内容来源：`README.md`、`DEVELOPMENT.md`、`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/src/main.rs`、`docs/01_reverse/REVERSE_ANALYSIS.md`（原 `docs/reverse-analysis.md`）§①②。所有事实均标注来源路径；未证实处标【未知】。

---

## 1. 项目定位

QueryLab（目标仓库名 `lys-query-lab`）是一个**本地优先（local-first）的数据库工作台桌面应用**，首期聚焦 MySQL / MariaDB，覆盖连接管理、Schema 浏览、SQL 编辑与执行、结果展示与导出、数据网格 CRUD、表设计器、结构对比（预览）、备份还原（仅 SQL 格式）。

- 来源：`README.md`「这是什么」；`docs/01_reverse/REVERSE_ANALYSIS.md` §1.1。
- 产品边界口径（对外承诺，源 `docs/RELEASE_CHECKLIST.md`，经 `docs/product-review/PRODUCT_LOGIC_REVIEW.md` PL-08 确认三处一致）：
  - 仅支持 MySQL / MariaDB；
  - 结构对比仅提供结构差异分析与结构变更 SQL 执行，不宣称数据同步；
  - 备份/导入仅支持 SQL 格式；
  - 单元格编辑仅在「单表直接查询 + 单列主键」时开放（结果面板），数据网格仅对单列主键表开放更新/删除。
- 形态：Tauri 2 桌面单窗口，默认 1400×900，最小 1000×600，可缩放（`src-tauri/tauri.conf.json` `app.windows[0]`）。
- 标识：productName `QueryLab`，identifier `com.i2kai.querylab`，前端包名 `querylab-ui`（`package.json` name），Rust crate `querylab`（`src-tauri/Cargo.toml` name）。

## 2. 仓库布局（含 src/ 与 src-ui/ 双前端现状说明）

```text
QueryLab/                      # 仓库根（git 分支 main）
├── src/                       # 【权威前端工作区】Svelte 5 UI（README 约定以此为准）
│   ├── App.svelte             # 应用壳：视图路由/状态栏/shellPanel（829 行）
│   ├── main.js                # 前端入口（Svelte mount）
│   ├── components/            # 9 个业务组件 + 3 个 .test.js
│   ├── lib/                   # notifications.js（Toast/确认）+ Counter.svelte（模板残留死代码）
│   └── assets/                # svelte.svg（模板残留）
├── src-tauri/                 # 【权威 Rust/Tauri 后端】
│   ├── src/main.rs            # 后端入口：注册 shell 插件 + 16 个命令
│   ├── src/commands/          # app / connection / query / metadata / backup
│   ├── src/db/                # types.rs（共享数据模型）+ driver.rs（抽象占位）
│   ├── src/storage/           # connections.json 读写
│   ├── src/security/          # 系统钥匙串（keyring）密码读写
│   ├── src/core/              # state.rs / errors.rs（占位，dead_code）
│   ├── src/util/              # mod.rs（占位）
│   ├── tauri.conf.json        # 窗口/构建/插件配置（无 security.csp）
│   ├── icons/                 # 应用图标 6 个（模板默认图标）
│   └── gen/schemas/           # Tauri 生成物（无 capabilities/ 目录）
├── src-ui/                    # ⚠️ 并行旧副本工作区（内含独立的 src/、src-tauri/、独立 .git）
│                              # 评审已标记「整目录删除【C9，待用户确认】」（docs/04_architecture/MODULE_ARCH.md §5）；
│                              # 本会话未删除（C 类决策）。注意：其表单预填密码 'root123456'，
│                              # 是 PL-01 凭据断裂缺陷曾被掩盖的环境性原因。
├── docs/                      # 文档区（SOP v2.0 编号体系 00-09 + 项目自有文档并存）
├── prototype/                 # HTML 原型：v0-old（旧版）/ v1-new（V1 新版），原位不动
├── dist/                      # Vite 构建产物
├── .github/workflows/         # deploy-docs.yml（docs 站点部署）
└── README.md / DEVELOPMENT.md # 项目自述与开发计划
```

- 双前端现状说明：`src/`（权威）与 `src-ui/`（旧副本）并存，`README.md`「当前仓库结构」与「默认以仓库根目录这套工作区为准」为权威约定来源；`src-ui/` 处置属 C 类决策待用户确认，本编号文档体系一律以 `src/` + `src-tauri/` 为事实基准。
- docs/ 目录下并存项目自有文档（`PRD.md`、`README.md`、`RELEASE_CHECKLIST.md`、`RELEASE_VERIFICATION_2026-04-22.md`、`CORE_LOGIC_REVIEW_2026-04-22.md`、`API_SQL_补全策略.md`、`index.md`、`.vitepress/`、`public/`、`package.json`、`node_modules/`），与本编号体系互不干扰，索引统一见 `docs/DOCUMENT_INDEX.md`。

## 3. 构建与运行方式

来源：`README.md`「开发命令」、`package.json` scripts、`src-tauri/tauri.conf.json` build 节。

| 命令 | 作用 | 定义处 |
|------|------|--------|
| `npm install` | 安装前端依赖 | 惯例（README） |
| `npm run dev` | 启动 Vite 开发服务器（纯前端） | `package.json` scripts.dev = `vite` |
| `npm run build` | 前端生产构建（输出 `dist/`） | `package.json` scripts.build = `vite build` |
| `npm run preview` | 预览构建产物 | `package.json` scripts.preview |
| `npm test` | 运行前端测试（vitest run） | `package.json` scripts.test |
| `npm run tauri:dev` | Tauri 桌面应用联调开发 | `package.json` + `tauri.conf.json` beforeDevCommand=`npm run dev`，devUrl=`http://localhost:5173` |
| `npm run tauri:build` | 打包桌面安装包（targets: all） | `package.json` + `tauri.conf.json` beforeBuildCommand=`npm run build`，frontendDist=`../dist` |

- Rust 侧构建入口：`src-tauri/build.rs`（tauri-build）；`cargo test` 可运行 Rust 单测（`src-tauri/src/storage/connections.rs` 2 个 + `src-tauri/src/commands/query.rs` 1 个，grep `#[test]` 计数）。

## 4. 关键入口

| 层 | 入口 | 说明 |
|----|------|------|
| 前端 | `src/main.js` → `src/App.svelte` | Svelte 5 mount；App.svelte 为应用壳（视图切换 viewMode：query/grid/design/sync/backup，状态栏，shellPanel） |
| 前端通信 | `import { invoke } from '@tauri-apps/api/core'` | `src/App.svelte` L3；8 个组件直接 invoke（统计：query_execute 10 处、meta_get_table_schema 5 处、meta_list_tables 4 处等，grep 计数） |
| 后端 | `src-tauri/src/main.rs` `fn main()` | `tauri::Builder` 注册 `tauri_plugin_shell::init()` 与 16 个命令（app_get_info、conn_list/upsert/delete/test、db_export/import、fs_write_file、meta_list_databases/list_tables/get_table_schema/get_schema_tree/create_database/create_table、query_execute/update_cell） |
| 配置 | `src-tauri/tauri.conf.json` | identifier `com.i2kai.querylab`；`withGlobalTauri: false`；无 security.csp（PERMISSION_REVIEW PM-01 关联） |
| 持久化入口 | `src-tauri/src/storage/connections.rs` `ConnectionStorage::new()` | 配置目录 = `dirs::config_dir()/querylab/connections.json` |
| 密钥入口 | `src-tauri/src/security/mod.rs` | keyring service = `com.i2kai.querylab.connection`，按连接 id 分条存取 |

## 5. 本编号体系内的关联阅读

- 技术栈明细：`docs/00_context/TECH_STACK.md`
- 资产清单：`docs/00_context/ASSET_INVENTORY.md`
- 依赖清单：`docs/00_context/DEPENDENCY_LIST.md`
- 逆向事实模型：`docs/01_reverse/REVERSE_ANALYSIS.md`
- 索引：`docs/DOCUMENT_INDEX.md`
