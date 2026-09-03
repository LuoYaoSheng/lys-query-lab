# QueryLab V1 API 设计——Tauri 命令契约（api-design）

> 版本：v1.0（2026-09-02，P7）
> 基线：`src-tauri/src/main.rs` 注册清单（**实际 16 个命令名**；昨晚文档「17 个」为计数勘误，见 product-review A3）+ 各 commands 源码 + P4 缺陷修正。
> 通用约定：
> - 错误形态：`Result<T, String>`（现状）。错误文案前缀规约：`连接失败: ` / `SQL 错误: ` / `更新失败: ` / `保存失败: ` / `写入文件失败: ` / `创建目录失败: ` / `无效的连接参数: ` / `连接超时（5秒）`。前端 `src/lib/api/` 依前缀归类到五态-错误（连接失败/SQL 报错）。
> - **统一返回结构 {ok,data,error,trace_id} 与错误码化【建议，待用户确认】（C14）**：默认维持 String 模型（改动最小），若确认升级则 core/errors.rs AppError（DB_CONN_FAILED 等）启用，本契约错误码列即届时枚举。
> - 序列化：camelCase 以 serde rename 为准（下表 JSON 名均核对源码）。

---

## 1. 命令清单总表（16 个）

| # | 命令 | 模块 | 状态 | V1 变更 |
|---|------|------|------|---------|
| 1 | app_get_info | app | 保留 | 无 |
| 2 | fs_write_file | app | 保留 | 无 |
| 3 | conn_list | connection | 保留 | 无（密码仍不回传） |
| 4 | conn_upsert | connection | **语义修正** | B1：空密码+已有 id=保留钥匙串旧密码 |
| 5 | conn_delete | connection | 保留 | 无 |
| 6 | conn_test | connection | 保留 | 无（5s 超时） |
| 7 | meta_list_databases | metadata | 保留 | 无 |
| 8 | meta_list_tables | metadata | 保留 | 无 |
| 9 | meta_get_table_schema | metadata | 保留 | foreign_keys 恒空如实（C13） |
| 10 | meta_create_database | metadata | 保留 | 无 |
| 11 | meta_create_table | metadata | 保留 | 无 |
| 12 | **meta_get_schema_tree** | metadata | **建议剔除** | 无调用方；**C8【待用户确认】** |
| 13 | query_execute | query | **契约修正** | B2：入参改语句数组（分句归属前端） |
| 14 | query_update_cell | query | 保留 | 无 |
| 15 | db_export | backup | 保留 | 无 |
| 16 | db_import | backup | 保留 | 受益于统一分句算法 |

---

## 2. 逐条契约

### 2.1 app_get_info（commands/app.rs）
- 入参：无。
- 出参：`{ version: String(CARGO_PKG_VERSION), platform: 'macos'|'windows'|'linux', build: 'dev'|'prod' }`。
- 错误：不返回错误（无 IO）。

### 2.2 fs_write_file（commands/app.rs）
- 入参：`{ path: String, contents: String }`。
- 出参：`boolean`。
- 错误：`创建目录失败: {io}`（自动 create_dir_all）/`写入文件失败: {io}`。
- 用途：所有导出链路落盘（B4 文件名规则见 data-model §5.3）。

### 2.3 conn_list（commands/connection.rs）
- 入参：无。
- 出参：`ConnectionInfo[]`（**password 永不出现在序列化输出**，driver.rs skip_serializing）。
- 副作用：明文密码自动迁移入钥匙串（load_all）。
- 错误：`{io/serde}`。

### 2.4 conn_upsert（commands/connection.rs + storage/connections.rs）🔧B1
- 入参：`{ connection: ConnectionInfo }`（id 空 → 后端生成 UUID）。
- 出参：`id: String`。
- **V1 语义修正（密码保留）**：
  - 旧（缺陷）：`password` 为空即 `delete_connection_password(id)`——编辑连接不重输密码会清空钥匙串（P4 PF-01）。
  - 新：`password` 非空 → 写入钥匙串；`password` 为空 **且 id 已存在** → **保留原密码不动作**；`password` 为空且为新连接 → 不写入（未配置密码）。
  - 需要显式清空密码的场景（如改用免密）：**【建议，待用户确认】** 增加入参 `clear_password: Option<bool>` 或前端删除重建；默认方案为前者。
- 错误：`{keyring/io}`。

### 2.5 conn_delete（commands/connection.rs）
- 入参：`{ id: String }`；出参：`boolean`（true）。
- 行为：删除钥匙串密码 + 移除 connections.json 条目。
- 错误：`{keyring/io}`。

### 2.6 conn_test（commands/connection.rs）
- 入参：`{ connection: ConnectionInfo }`（表单数据允许临时 id `__test__`）。
- 出参：`{ latency_ms: u64, server_version: String, user: String（已去 @host 后缀）, default_db: Option<String> }`。
- 错误：`连接超时（5秒）`（tokio timeout） / `连接失败: {mysql err}` / `无效的连接参数: {opts}`。
- UI 联动（B13）：失败块内提供「编辑连接」直达。

### 2.7 meta_list_databases（commands/metadata.rs）
- 入参：`{ connection }`；出参：`string[]`（**前端再过滤 information_schema/mysql/performance_schema/sys**，App.svelte 事实）。
- 错误：`连接失败: …`。

### 2.8 meta_list_tables（commands/metadata.rs）
- 入参：`{ connection, database: String, includeViews: boolean }`。
- 出参：`TableInfo[]`（name/type/comment?/engine?/rowsEst?）。
- 错误：`连接失败: …` / SQL 错误。

### 2.9 meta_get_table_schema（commands/metadata.rs）
- 入参：`{ connection, database, table }`。
- 出参：`TableSchema{ database, table, columns[], indexes[], foreign_keys[](恒空，C13), create_sql? }`。
- 错误：`连接失败: …`。
- 用途：网格加载/设计器编辑/结构对比/编辑门禁判定。

### 2.10 meta_create_database（commands/metadata.rs）
- 入参：`{ params: { connection, name, charset(default utf8mb4), collation(default utf8mb4_unicode_ci) } }`。
- 出参：`String` 消息。
- 错误：`连接失败: …` / SQL 错误 / 空名（前端拦截「请输入数据库名称」）。

### 2.11 meta_create_table（commands/metadata.rs）
- 入参：`{ params: { connection, database, table, columns: ColumnDefinition[]{name,type,length?,nullable,primaryKey(primaryKey alias),autoIncrement,default_value?,comment}, engine, charset, collation, comment } }`。
- 出参：`String`（`表 '{db}.{table}' 创建成功`）。
- 规则（源码事实）：自增/主键列强制 NOT NULL；AUTO_INCREMENT 单列定义 + 表级 PRIMARY KEY；注释单引号转义。
- 错误：`连接失败: …` / SQL 错误 / 校验（前端：至少一列/命名规则/自增须主键/仅单列主键）。

### 2.12 meta_get_schema_tree —— 建议剔除【C8，待用户确认】
- 现状：已注册，前端零调用（逆向⑨.3）；SchemaTree 用 meta_list_databases + meta_list_tables 组合已覆盖同一目标。
- 选项 A（默认建议）：从 generate_handler 剔除并删除实现（减少死面）。
- 选项 B：保留并在 V1 接线（侧栏一次拉全树，减少 N 次 meta_list_tables；需增加前端缓存失效策略）。
- **未经用户确认不做任何一侧改动。**

### 2.13 query_execute（commands/query.rs）🔧B2
- 入参（V1 修正）：
  ```jsonc
  {
    "connection": ConnectionInfo,
    "statements": ["SELECT …;", "UPDATE …;"],   // V1 新增：前端 sqlUtils.parseStatements 分句结果
    "sql": "原始文本（可选，兼容一个版本的过渡字段）",
    "max_rows": 1000
  }
  ```
- **分句逻辑归属（P4 PF-02 结论）**：唯一实现于前端 `src/lib/sqlUtils/parseStatements`（字符串/块注释/行注释感知；DELIMITER 场景维持旧项目「简化处理」如实声明）；后端**只按语句数组顺序执行，不再自行 split(';`）**。后端备份导入 split_sql_statements 同步替换为等价 Rust 实现（同一测试用例集双语言对齐）。
- 执行规则（源码事实保留）：SELECT/SHOW/DESCRIBE/EXPLAIN/WITH 前缀 → `query_iter`（列元数据来自 result.columns，行 take(max_rows)）；其余 → `query_drop`；每语句一个 QueryResultSet；整体 queryId/elapsedMs。
- 出参：`QueryResult{ queryId, sets[], elapsedMs }`（结构见 data-model §4）。
- 错误：`连接失败: {mysql}` / `SQL 错误: {mysql（含 errno）}`——任一语句失败整批返回 Err（现状语义；批量逐条进度由前端逐条调用获得，见 tech-architecture §3.3）。

### 2.14 query_update_cell（commands/query.rs）
- 入参：`{ params: { connection, table("db.table"), column, primary_key, primary_key_value, new_value, is_null } }`。
- 出参：`{ success, message('更新成功，影响 N 行'), affectedRows }`。
- SQL：`UPDATE \`db\`.\`t\` SET \`col\` = NULL|quoted WHERE \`pk\` = quoted LIMIT 1`（quote_value：数字直写，字符串转义 \ 与 ''）。
- 错误：`连接失败: …` / `更新失败: …`。

### 2.15 db_export（commands/backup.rs）
- 入参：`{ params: { connection, database, tables[], export_type:'structure'|'data'|'both', format:'sql'|'json'|'csv', file_path } }`（前端仅 both+sql）。
- 出参：`{ success, size, tables, message }`。
- 限制（如实）：json 为「SQL 包 JSON 字符串」简化实现；csv 仅第一个表（逆向⑨.2）——V1 前端继续仅开放 SQL，后端能力面不动。
- 错误：`连接失败: …` / IO。

### 2.16 db_import（commands/backup.rs）
- 入参：`{ params: { connection, database, file_path, drop_existing } }`。
- 出参：`{ success, tables, rows, message }`。
- 行为：可选 drop_existing → 逐句执行 → 忽略 already exists/duplicate。
- 错误：`连接失败: …` / `读取文件失败`（IO）/ SQL 错误。

---

## 3. 前端 API 层规约（src/lib/api/，新增）

```
src/lib/api/
  index.ts      // invoke 封装：统一错误归一（前缀→ErrorKind: conn|sql|fs|unknown）、超时
  connection.ts // connList/connUpsert/connDelete/connTest
  query.ts      // queryExecute(statements, opts)/queryUpdateCell
  metadata.ts   // metaListDatabases/metaListTables/metaGetTableSchema/metaCreateDatabase/metaCreateTable
  backup.ts     // dbExport/dbImport
  app.ts        // appGetInfo/fsWriteFile
```

- 每函数返回强类型（JSDoc 或 TS【待确认】），组件禁止直接 `invoke()`。
- 并发：同一 connection 的元数据请求按 (connectionId,db) 去重（SchemaTree 展开缓存语义不变）。
- 兼容期：queryExecute 内部优先发 `statements`，若后端未升级（旧版二进制）自动降级发 `sql`（过渡一个版本后移除）。

---

## 4. 错误码草案（供 C14 决策时启用）

| 前缀/场景 | 草案码 | 五态归类 |
|-----------|--------|----------|
| 连接失败/超时/无效参数 | DB_CONN_FAILED | Error-连接失败 |
| SQL 语法/执行错误（errno） | DB_SQL_ERROR(errno) | Error-SQL 报错 |
| 更新失败 | DB_UPDATE_FAILED | Error-SQL 报错 |
| 文件写入/目录创建 | FS_WRITE_FAILED | Error |
| 钥匙串读写 | KEYRING_FAILED | Error |
| 无权限（MySQL access denied） | DB_ACCESS_DENIED | Permission（门禁） |
