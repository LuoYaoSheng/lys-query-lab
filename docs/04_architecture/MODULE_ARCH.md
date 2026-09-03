# QueryLab V1 模块拆分建议（module-split）

> 版本：v1.0（2026-09-02，P7）
> 基线：`docs/01_reverse/REVERSE_ANALYSIS.md` §2（现结构）+ P4 §4 公共能力识别 + STATE_MACHINE.md 与 API_SPEC.md（docs/08_development/）。
> 目标：在不推倒重写的前提下，把「组件即模块、逻辑内聚于组件」的旧结构拆为「UI 组件 / 业务模块 / 服务层 / 契约层」四层，消除三处重复实现与两处未接线组件。

---

## 1. 目标目录结构（V1）

```text
QueryLab/
├── src/
│   ├── App.svelte                  # 应用壳（瘦身：仅布局+视图路由+全局层挂载）
│   ├── main.js / app.css
│   ├── lib/
│   │   ├── api/                    # ★新增：Tauri invoke 统一层（api-design §3）
│   │   ├── stores/                 # ★新增：connection/editor/result/notification + 视图内 store（state-management）
│   │   ├── sqlUtils/               # ★新增：parseStatements/formatSql/escapeSqlString/isSensitive
│   │   ├── exporter/               # ★新增：CSV/JSON/SQL INSERT 生成 + 文件名规则（B4/B10）
│   │   ├── editGuard/              # ★新增：结果/网格共用编辑门禁（PF-10）
│   │   └── notifications.js        # 平移并入 stores/notification（保留对外 API 兼容）
│   ├── components/                 # UI 组件（9 个 + 接线 BatchProgressPanel）
│   │   ├── ConnectionManager.svelte
│   │   ├── SchemaTree.svelte
│   │   ├── SqlEditor.svelte
│   │   ├── ResultsPanel.svelte
│   │   ├── DataGrid.svelte
│   │   ├── TableDesigner.svelte
│   │   ├── DataSync.svelte
│   │   ├── DatabaseBackup.svelte
│   │   ├── NotificationCenter.svelte
│   │   └── BatchProgressPanel.svelte   # B3：由 App 挂载接线
│   └── test/                       # vitest 组件测试（扩：sqlUtils/exporter/editGuard 单测）
├── src-tauri/src/
│   ├── main.rs                     # 16 命令注册（meta_get_schema_tree 待 C8 决策）
│   ├── commands/                   # app/connection/query/metadata/backup（语义按 api-design 修正）
│   ├── db/                         # types.rs 共享模型 + driver.rs（占位处置见 §4）
│   ├── storage/                    # connections.rs（B1 密码保留语义 + 单测扩展）
│   ├── security/                   # keyring 封装（不变）
│   ├── core/                       # state.rs/errors.rs 占位处置见 §4
│   └── util/
├── docs/（含 00_context-09_test 编号体系与 07_design_system）· prototype/{v0-old,v1-new}/
└── （src-ui/ → 建议删除，见 §5）
```

---

## 2. 业务模块划分（11 模块，对齐 P4 §4.2）

| 模块 | 归属组件 | Store | 服务依赖 | 边界说明 |
|------|----------|-------|----------|----------|
| M1 连接管理 | ConnectionManager | connectionStore | api/connection | CRUD/测试/钥匙串语义（B1） |
| M2 Schema 浏览 | SchemaTree | connectionStore(schemaCache) | api/metadata | 树/右键/新建库表入口/DDL（RENAME/TRUNCATE/DROP） |
| M3 SQL 编辑 | SqlEditor | editorStore | sqlUtils, api/query | 历史/片段/格式化/补全（B7/B8）/批量开关 |
| M4 查询执行调度 | App（调度逻辑移入 editorStore.execute） | editorStore | api/query | 分句（B2）/事务包裹/批量逐条 |
| M5 结果展示与导出 | ResultsPanel | resultStore | exporter, editGuard, api/query | 多 Tab/五态/受限编辑（B11）/导出（B4） |
| M6 表数据 CRUD | DataGrid | gridStore | editGuard, exporter, api/query, api/metadata | 分页/筛选/行级 CRUD/导出 |
| M7 表设计 | TableDesigner | designerStore | api/metadata, api/query | 新建/diff→ALTER（预览确认） |
| M8 结构对比 | DataSync | syncStore | api/metadata, api/query | 差异分析/SQL 生成（B6）/危险执行 |
| M9 备份还原 | DatabaseBackup | backupStore | api/backup | 导出/导入/进度 |
| M10 通知与确认 | NotificationCenter | notificationStore | — | Toast/confirm（Promise） |
| M11 批量进度 | BatchProgressPanel | resultStore.batchProgress | — | **B3 接线**：M4 事件驱动 |

**跨模块事件（保留 Svelte 事件或转 store action）**：syncComplete → M2 刷新（refreshAll+loadDatabases）；设计器保存成功 → M2 刷新对应库 + 切 M6；单元格更新成功 → resultStore.refresh（500ms 延迟刷新语义保留）。

---

## 3. 重复实现合并清单（P4 PF-10/PF-11 落地）

| 旧实现 | 合并到 | 验收点 |
|--------|--------|--------|
| ResultsPanel.isEditable + DataGrid.supportsRowMutation | lib/editGuard | 同一判定两种入口（result: 单表直查+单列主键+主键列在结果；grid: 单列主键在列），单测覆盖 |
| ResultsPanel 单元格编辑 + DataGrid 单元格编辑（两套 DOM/键盘逻辑） | 统一「EditableCell」行为（DS C12） | Enter/Esc/NULL/Tab（B9 保留值）一致 |
| ResultsPanel.exportCSV/JSON + DataGrid.exportCSV/JSON/SQL | lib/exporter | 文件名规则统一（B4）；CSV 转义一致 |
| escapeSqlString（DataGrid）与 quote_value（Rust） | 双侧保留但测试对齐（转义 \ 与 '） | 同用例集 |

---

## 4. 死代码与占位处置

| 项 | 现状 | V1 处置 |
|----|------|---------|
| src/lib/Counter.svelte | 模板残留，无引用 | 删除 |
| core/state.rs（AppState/ConnManager/SessionManager/TaskManager） | 全 TODO dead_code | 删除；待 C10/C3 增强确认后按新设计重建 |
| core/errors.rs（AppError） | 未被 commands 使用 | 保留文件但标注「C14 决策后启用或删除」；默认删除 |
| util/mod.rs | 空 | 删除 |
| db/driver.rs（Driver/DbConnection/MySQLDriver 空壳） | 占位 | **保留 types 部分（ConnectionInfo 等在用）**；trait/空壳删除或随 C12 扩库决策保留——默认删除空壳仅留数据模型（迁 db/types.rs） |
| meta_get_schema_tree | 无调用方 | C8 决策（默认建议剔除） |
| index.html 标题 querylab-ui / public/vite.svg | 模板残留 | 改为 QueryLab / 删除（成品化，tech-architecture §4） |

> 以上删除均不改变任何已实现功能面；如用户偏好保守可仅删 Counter.svelte，其余挂 `#[allow(dead_code)]` 并留 TODO。

---

## 5. src-ui 清理决策留档【C9，待用户确认】

- 事实（逆向 §2.4）：src-ui 为并行旧副本——缺 NotificationCenter、notifications.js、全部测试；其余组件均落后于根工作区；README 称「用于 UI / Tauri 联调整理」，但 RELEASE_CHECKLIST 要求「角色说明没有冲突」，双工作区已构成口径冲突源；且 node_modules 实际只装在 src-ui 下（本机状态）。
- **建议**：整目录删除（含其 src-tauri 副本与 node_modules），README 同步改为单一工作区说明。
- 备选：若仍有联调依赖，则冻结为 `archive/src-ui-legacy/`（移出构建脚本、README 注明只读）。
- **处置**：默认不动（禁止改既有源码），本档留待用户确认后执行。

---

## 6. 迁移步骤建议（开发执行序）

1. 建骨架：lib/api + lib/stores + lib/sqlUtils/exporter/editGuard（纯新增，不破坏现状）。
2. 平移通知层到 stores/notification（保持 notifications.js API 兼容再切换引用）。
3. 逐模块接管状态：connection → editor → result → grid/designer/sync/backup（每步跑既有 vitest + 新增单测）。
4. App.svelte 瘦身为壳 + 视图路由；BatchProgressPanel 接线（B3）。
5. 后端按 api-design 修正 conn_upsert（B1）与 query_execute 入参（B2，兼容期双字段）+ cargo test 扩展。
6. 死代码清理（§4）与 src-ui 决策执行（§5）。
7. 每步以 `docs/09_test/V1_ACCEPTANCE.md` 的断言集（浏览器实测口径）回归。

---

## 7. 测试基线（随拆分同步）

| 层 | 既有 | 新增 |
|----|------|------|
| 前端组件 | ConnectionManager/DataGrid/DatabaseBackup 3 件 | BatchProgressPanel 接线测试；编辑门禁双入口一致性 |
| 前端纯逻辑 | — | sqlUtils.parseStatements（含字符串内分号/注释用例）、exporter 文件名规则、editGuard |
| Rust | connections 存储 2 例 + format_table_ident 1 例 | conn_upsert 密码保留语义（B1）用例；分句 Rust 版对齐用例（C 兼容期后） |
