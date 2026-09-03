# QueryLab Design System — 资源与公共参数（assets）

> 版本：v1.0（2026-09-02）
> 内联 SVG 关键图标（16px 网格，stroke 1.5，`currentColor` 继承文本色）——替代旧项目 emoji 图标（📁📊👁️🔑🗑️📋🔍💾⚡🔒⟡🕒等，跨平台渲染不一致）。语义与旧项目图标一一对应，不新增商业含义。
> 附：公共参数（17 个 Tauri 命令清单 / 连接配置字段 / 分页参数 / 数据类型枚举）——真实值 + 来源，供 V1 原型与 P7 契约直接引用。

---

## 1. 内联 SVG 图标集（V1 使用，组件 C32）

> 规格：viewBox="0 0 16 16"；fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"；尺寸通过 CSS 控制（默认 16px，空态大图标 48px）。下表给出每个图标的 path 数据与替换对象。

| 图标 ID | 替换旧 emoji | 语义 | SVG paths（d 值） |
|---------|--------------|------|-------------------|
| icon-db | 📁 / 库节点 | 数据库（Schema） | `M2 4c0-1.1 2.7-2 6-2s6 .9 6 2-2.7 2-6 2-6-.9-6-2Z` `M2 4v8c0 1.1 2.7 2 6 2s6-.9 6-2V4` `M2 8c0 1.1 2.7 2 6 2s6-.9 6-2` |
| icon-table | 📊 | 表 | `M2.5 3.5h11v9h-11z` `M2.5 6.5h11` `M2.5 9.5h11` `M6 3.5v9` |
| icon-view | 👁️ | 视图 | `M1.5 8s2.4-4.2 6.5-4.2S14.5 8 14.5 8 12.1 12.2 8 12.2 1.5 8 1.5 8Z` `M9.8 8a1.8 1.8 0 1 1-3.6 0 1.8 1.8 0 0 1 3.6 0Z` |
| icon-column | （列头/差异列） | 列 | `M4 2.5v11` `M12 2.5v11` `M2.5 4h3` `M10.5 4h3` `M2.5 12h3` `M10.5 12h3` |
| icon-index | （索引只读区） | 索引 | `M3 13V6l3-1.5L9 6l4-2v7l-4 2-3-1.5L3 13Z`（简化山形） |
| icon-key | 🔑 | 主键 | `M10.5 3a3 3 0 0 0-2.8 4.1L2.5 12.3V14h1.7l.8-.8v-1.2h1.2l.8-.8h1.2l1-1A3 3 0 1 0 10.5 3Z` `M11.5 6.2a.7.7 0 1 1-1.4 0 .7.7 0 0 1 1.4 0Z` |
| icon-connection | （连接区标题） | 连接/插头 | `M5.5 2.5v3M10.5 2.5v3` `M3.5 5.5h9v2a4.5 4.5 0 0 1-9 0v-2Z` `M8 12v2.5` |
| icon-run | ▶ | 运行 | `M4.5 3l8 5-8 5V3Z` |
| icon-batch | ⚡ | 批量模式 | `M9 1.5 3.5 9H7l-.8 5.5L12.5 7H9l0-5.5Z` |
| icon-transaction | 🔒 | 事务 | `M4 7.5V5.5a4 4 0 0 1 8 0v2` `M2.5 7.5h11v6h-11z` |
| icon-format | ⟡ | 格式化 | `M3 3l4 5-4 5` `M13 3 9 8l4 5` |
| icon-snippet | 📋 | 片段 | `M5 2.5h6V14H5z` `M5 4.5h6M7 2.5v0`（含两横线 `M6.8 7h2.4M6.8 9.5h2.4`） |
| icon-history | 🕒 | 历史 | `M8 3.5a4.5 4.5 0 1 1-4.4 5.5` `M8 5.5V8l2 1.5` `M2.5 6.5 3.6 9l1.4-2.4` |
| icon-search | 🔍 | 搜索/结构对比 | `M7 2.5a4.5 4.5 0 1 1 0 9 4.5 4.5 0 0 1 0-9Z` `M10.3 10.3 14 14` |
| icon-design | 📋（设计表） | 表设计器 | `M2.5 2.5h11v3h-11z` `M2.5 8h4.3v5.5H2.5z` `M9.2 8h4.3v5.5H9.2z` |
| icon-backup | 💾 | 备份还原 | `M2.5 2.5h9L13.5 5v8.5h-11z` `M5 2.5V6h5V2.5` `M5 13.5v-4h6v4` |
| icon-export | （导出按钮） | 导出 | `M8 2v8` `M5 7l3 3 3-3` `M2.5 13.5h11` |
| icon-refresh | 🔄 / ↻ | 刷新 | `M13.5 8a5.5 5.5 0 1 1-1.6-3.9` `M13.5 2.5V6H10` |
| icon-rename | ✏️ | 重命名 | `M11 2.5 13.5 5 5.5 13H3v-2.5l8-8Z` |
| icon-trash | 🗑️ / ❌ | 删除 | `M2.5 4h11` `M5 4V2.5h6V4` `M4 4l.7 9.5h6.6L12 4` `M6.7 6.5v5M9.3 6.5v5` |
| icon-plus | ＋ | 新建/添加 | `M8 3v10M3 8h10` |
| icon-close | ✕ / × | 关闭 | `M3.5 3.5l9 9M12.5 3.5l-9 9` |
| icon-edit | ✎ | 编辑 | `M11 2.5 13.5 5 5.5 13H3v-2.5l8-8Z` |
| icon-test | （测试连接） | 闪电测试 | `M9 1.5 3.5 9H7l-.8 5.5L12.5 7H9l0-5.5Z`（同 icon-batch，语境区分） |
| icon-warning | ⚠️ | 错误/危险 | `M8 2 14.5 13.5h-13L8 2Z` `M8 6.5v3.5` `M8 11.8v.2` |
| icon-check | ✓ | 保存/成功 | `M3 8.5l3.2 3L13 4.5` |
| icon-info | （提示） | 信息 | `M8 2.5a5.5 5.5 0 1 1 0 11 5.5 5.5 0 0 1 0-11Z` `M8 7.5v3.5` `M8 5.2v.2` |
| icon-file-sql | （.sql 文件） | SQL 文件 | `M4 1.5h5.5L12.5 4.5V14.5H4z` `M9.5 1.5v3h3` `M6.2 9c.4-1.5 3.2-1.5 3.2.3 0 1.3-3 1.4-3 3.2h3` |
| icon-empty-inbox | 📭 | 空表 | `M2.5 6.5h11v7h-11z` `M2.5 6.5 4.8 2.5h6.4l2.3 4` `M6.5 10h3` |

使用规则：
1. 单文件原型中 `<svg>` 直接内联，禁止外链图标 CDN。
2. 图标颜色一律 `currentColor`（跟随后景文本 token）；危险语境配 `--ql-danger`。
3. 每个图标 `aria-hidden="true"`，语义由相邻文本承担（无障碍）。

---

## 2. 公共参数（真实值 + 来源）

### 2.1 Tauri 命令清单（实际注册 16 个，`src-tauri/src/main.rs` generate_handler 逐项清点）

| # | 命令 | 入参（JSON 名） | 出参 | 错误形态 | 来源 |
|---|------|-----------------|------|----------|------|
| 1 | `app_get_info` | 无 | `{version, platform, build}` | 无（不返回 Result） | commands/app.rs |
| 2 | `conn_list` | 无 | `ConnectionInfo[]`（无密码） | `Result<_, String>` | commands/connection.rs |
| 3 | `conn_upsert` | `connection: ConnectionInfo` | `id: String` | String | connection.rs |
| 4 | `conn_delete` | `id: String` | `bool` | String | connection.rs |
| 5 | `conn_test` | `connection: ConnectionInfo` | `{latency_ms, server_version, user, default_db}`（5s 超时） | String（「连接超时（5秒）」/「连接失败: …」） | connection.rs |
| 6 | `db_export` | `params: {connection, database, tables[], export_type, format, file_path}` | `{success, size, tables, message}` | String | backup.rs |
| 7 | `db_import` | `params: {connection, database, file_path, drop_existing}` | `{success, tables, rows, message}` | String | backup.rs |
| 8 | `fs_write_file` | `path, contents` | `bool` | String（自动建目录） | app.rs |
| 9 | `meta_list_databases` | `connection` | `string[]` | String | metadata.rs |
| 10 | `meta_list_tables` | `connection, database, includeViews` | `TableInfo[]` | String | metadata.rs |
| 11 | `meta_get_table_schema` | `connection, database, table` | `TableSchema`（foreign_keys 恒空） | String | metadata.rs |
| 12 | `meta_get_schema_tree` | `connection` | `SchemaNode[]` | String | metadata.rs（**无前端调用方，C8 待决策**） |
| 13 | `meta_create_database` | `params: {connection, name, charset, collation}` | `String` 消息 | String | metadata.rs |
| 14 | `meta_create_table` | `params: {connection, database, table, columns[], engine, charset, collation, comment}` | `String` 消息 | String | metadata.rs |
| 15 | `query_execute` | `connection, sql, max_rows` | `{queryId, sets[], elapsedMs}` | String（「连接失败: …」/「SQL 错误: …」） | query.rs |
| 16 | `query_update_cell` | `params: {connection, table, column, primary_key, primary_key_value, new_value, is_null}` | `{success, message, affectedRows}` | String | query.rs |

> **勘误（A3，2026-09-02 P4 核查）**：逆向报告原文称「17 个已注册 command」，经逐项清点 `src-tauri/src/main.rs` `generate_handler!` 列表实为 **16 个命令名**（上表 #1-#16）。昨晚口径将 app.rs 内的两个命令（app_get_info、fs_write_file）与文件统计混淆所致。已同步勘误 `docs/01_reverse/REVERSE_ANALYSIS.md` 两处（1.2 架构图与 2.3 表）；记录见 `docs/06_review/PRODUCT_REVIEW.md` 第七节 A3。

### 2.2 连接配置字段（ConnectionInfo，`src-tauri/src/db/driver.rs` L56-70）

| JSON 字段 | Rust 字段 | 类型 | 默认 | 说明 |
|-----------|-----------|------|------|------|
| `id` | id | String | 空则后端生成 UUID | 主键 |
| `name` | name | String | — | 连接名称（缺省显示 host） |
| `driver` | driver_type | String | 'mysql'（表单默认） | 驱动/方言（B5 修复依据字段） |
| `host` | host | String | — | 主机 |
| `port` | port | u16 | 3306 | 端口 |
| `user` | user | String | — | 用户 |
| `password` | password | String | — | **不出现在序列化输出**（skip_serializing），存钥匙串；B1 修复语义：编辑时空=保持原密码 |
| `defaultDb` | default_db | Option<String> | None | 默认库 |

持久化：`{config_dir}/querylab/connections.json`（无密码）；钥匙串 service=`com.i2kai.querylab.connection`，account=连接 id。

### 2.3 分页与数量参数

| 参数 | 值 | 来源 |
|------|-----|------|
| 网格分页 pageSize | 50 | DataGrid.svelte |
| 查询截断 maxRows | 1000 | App.svelte executeQuery |
| 历史上限 MAX_HISTORY | 100 | SqlEditor.svelte |
| 连接测试超时 | 5 秒 | connection.rs |
| 备份每表行上限 | 10000 | backup.rs |
| 编辑 UPDATE/DELETE | …LIMIT 1 | query.rs / DataGrid.svelte |

### 2.4 数据类型与枚举（前端枚举源码值）

| 枚举 | 值 | 来源 |
|------|-----|------|
| 视图/表类型 | `BASE TABLE` / `VIEW`（TableInfo.type） | db/types.rs |
| 字符集下拉 | utf8mb4 / utf8 / latin1 / gbk / big5（6 项含联动排序） | SchemaTree.svelte |
| 排序规则 | utf8mb4_unicode_ci 等 7 项（联动） | SchemaTree.svelte |
| 存储引擎 | InnoDB / MyISAM / MEMORY / ARCHIVE / CSV | TableDesigner.svelte |
| 列类型分组 | 整数 / 浮点数 / 字符串 / 二进制 / 日期时间 / 其他（6 组） | TableDesigner.svelte |
| 长度适用类型 | CHAR / VARCHAR / BINARY / VARBINARY / DECIMAL / NUMERIC | TableDesigner.svelte |
| 导出类型 export_type | structure / data / both（前端仅 both） | backup.rs |
| 导出格式 format | sql / json / csv（前端仅 sql） | backup.rs |
| RowValue | Null / Bool / Number / Float / String / Bytes（untagged） | db/types.rs |
| 系统库过滤 | information_schema / mysql / performance_schema / sys | App.svelte |
| 敏感词 | password / secret / token / api[_-]?key / access[_-]?key / private[_-]?key / credential | SqlEditor.svelte L33-41 |
| localStorage | querylab_sql_history / querylab_sql_history_enabled | SqlEditor.svelte L31-32 |
| 快捷键 | Ctrl+Enter / Ctrl+S / Ctrl+H / Ctrl+K / F1 / Ctrl+F / Tab | SqlEditor.svelte keymap |

---

## 3. 品牌资产

- 产品名：QueryLab（lys-query-lab）；identifier `com.i2kai.querylab`（tauri.conf.json）。
- Logo：顶栏文字「QueryLab」（16px，`--ql-primary` 色，App.svelte L571-573）——无图形 logo 资产（如实记录）。
- 窗口：1400×900（默认）/ 1000×600（最小）（tauri.conf.json）。
- 旧项目残留：`index.html` 标题「querylab-ui」、public/vite.svg 模板残留（逆向报告⑨.11，成品化清理项，非 DS 资产）。
