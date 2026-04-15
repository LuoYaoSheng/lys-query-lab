> 更新说明（2026-03-31）
>
> 当前 `QueryLab` 本地实现使用的是 `Tauri 2 + Rust + Svelte 5 + CodeMirror 6`。
> 这份文档里的 API 设计和元数据 SQL 仍然有效，但编辑器部分最初是按 `Monaco` 思路写的。
> 现在应将这里的“补全策略”理解为**编辑器能力设计参考**，当前实现落点应优先映射到 `CodeMirror`。

下面继续把你要的 3 块补齐：**Tauri Commands API（入参/出参 JSON）**、**MySQL 元数据 SQL 清单**、**编辑器补全策略（缓存/刷新/性能）**。都按“直接照着写代码”的粒度来。

---

# A) Tauri Commands API 清单（入参/出参）

## A.0 通用返回结构（所有 command 统一）

```json
{
  "ok": true,
  "data": {},
  "error": null,
  "trace_id": "optional-string"
}
```

失败示例：

```json
{
  "ok": false,
  "data": null,
  "error": { "code": "DB_CONN_FAILED", "message": "连接失败", "detail": "..." },
  "trace_id": "..."
}
```

## A.1 通用类型定义（前后端共享）

### ConnectionInfo

```json
{
  "id": "uuid",
  "name": "prod-mysql",
  "driver": "mysql",
  "host": "127.0.0.1",
  "port": 3306,
  "user": "root",
  "password": "encrypted-or-plain-input",
  "default_db": "optional",
  "ssl": {
    "enabled": false,
    "mode": "preferred | required | verify_ca | verify_identity",
    "ca_path": "optional",
    "cert_path": "optional",
    "key_path": "optional"
  },
  "ssh": {
    "enabled": false,
    "host": "1.2.3.4",
    "port": 22,
    "user": "ubuntu",
    "auth": { "type": "password|key", "password": "optional", "key_path": "optional", "passphrase": "optional" },
    "remote_host": "127.0.0.1",
    "remote_port": 3306,
    "local_port": 0
  },
  "tags": ["dev", "prod"],
  "created_at": 0,
  "updated_at": 0
}
```

### QueryOptions

```json
{
  "database": "optional-db",
  "max_rows": 1000,
  "timeout_ms": 30000,
  "paging": { "enabled": true, "page": 1, "page_size": 200 },
  "return_format": "chunked | full",
  "include_total": false
}
```

### QueryResult（chunked）

```json
{
  "query_id": "uuid",
  "sets": [
    {
      "set_index": 0,
      "columns": [
        { "name": "id", "type": "BIGINT", "nullable": false },
        { "name": "name", "type": "VARCHAR", "nullable": true }
      ],
      "meta": { "elapsed_ms": 12, "affected_rows": 0, "warning_count": 0 },
      "chunks": [
        { "chunk_index": 0, "rows": [[1,"a"],[2,"b"]] }
      ],
      "paging": { "page": 1, "page_size": 200, "has_more": true }
    }
  ]
}
```

---

## A.2 App/Health

### `app_get_info()`

**req**：`{}`
**res.data**

```json
{ "version": "1.0.0", "platform": "mac|win|linux", "build": "dev|prod" }
```

---

## A.3 连接管理

### `conn_list()`

**req**：`{}`
**res.data**

```json
{ "connections": [/* ConnectionInfo */] }
```

### `conn_upsert(connection: ConnectionInfo)`

**req**

```json
{ "connection": { /* ConnectionInfo */ } }
```

**res.data**

```json
{ "id": "uuid" }
```

### `conn_delete(id: string)`

**req**

```json
{ "id": "uuid" }
```

**res.data**

```json
{ "deleted": true }
```

### `conn_test(connection: ConnectionInfo)`

**req**

```json
{ "connection": { /* ConnectionInfo */ } }
```

**res.data**

```json
{ "latency_ms": 25, "server_version": "8.0.x", "user": "root", "default_db": "..." }
```

### `conn_open(connection_id: string)`

打开连接池并返回运行态句柄。
**req**

```json
{ "connection_id": "uuid" }
```

**res.data**

```json
{ "runtime_id": "uuid", "server_version": "8.0.x" }
```

### `conn_close(runtime_id: string)`

**req**

```json
{ "runtime_id": "uuid" }
```

**res.data**

```json
{ "closed": true }
```

---

## A.4 Schema / Metadata

### `meta_list_databases(runtime_id)`

**req**

```json
{ "runtime_id": "uuid" }
```

**res.data**

```json
{ "databases": ["db1","db2"] }
```

### `meta_list_tables(runtime_id, database, include_views?)`

**req**

```json
{ "runtime_id": "uuid", "database": "db1", "include_views": true }
```

**res.data**

```json
{
  "tables": [
    { "name": "users", "type": "BASE TABLE", "comment": "", "engine": "InnoDB", "rows_est": 1234 },
    { "name": "v_users", "type": "VIEW", "comment": "" }
  ]
}
```

### `meta_get_table_schema(runtime_id, database, table)`

**res.data**

```json
{
  "database": "db1",
  "table": "users",
  "columns": [
    { "name": "id", "type": "BIGINT", "nullable": false, "default": null, "comment": "", "extra": "auto_increment" }
  ],
  "indexes": [
    { "name": "PRIMARY", "unique": true, "columns": ["id"] }
  ],
  "foreign_keys": [
    { "name": "fk_user_team", "columns": ["team_id"], "ref_table": "teams", "ref_columns": ["id"] }
  ],
  "create_sql": "CREATE TABLE ..."
}
```

### `meta_search_symbols(runtime_id, database, q, limit)`

用于补全/搜索：表、列、函数、过程。
**req**

```json
{ "runtime_id": "uuid", "database": "db1", "q": "us", "limit": 50 }
```

**res.data**

```json
{
  "tables": ["users","user_logs"],
  "columns": [{ "table":"users", "name":"user_id" }],
  "routines": ["user_stats"]
}
```

---

## A.5 SQL 执行与取消

### `query_execute(runtime_id, sql, options)`

**req**

```json
{ "runtime_id": "uuid", "sql": "select * from users;", "options": { /* QueryOptions */ } }
```

**res.data**

```json
{ "query_id": "uuid", "result": { /* QueryResult */ } }
```

### `query_fetch_more(runtime_id, query_id, set_index, next_page?)`

用于分页/继续拉取 chunk。
**req**

```json
{ "runtime_id": "uuid", "query_id": "uuid", "set_index": 0, "next_page": 2 }
```

### `query_cancel(runtime_id, query_id)`

**res.data**

```json
{ "canceled": true }
```

> 实现建议：每次执行生成 `query_id`，Rust 侧保存 cancel token；mysql 驱动层支持中断/超时就走中断，否则至少能“停止继续拉取+丢弃结果”。

---

## A.6 表数据浏览/编辑

### `table_select(runtime_id, database, table, where?, order_by?, page, page_size)`

**req**

```json
{
  "runtime_id":"uuid","database":"db1","table":"users",
  "where":"id > 10","order_by":"id desc","page":1,"page_size":200
}
```

**res.data**

```json
{
  "columns":[{"name":"id","type":"BIGINT"}],
  "rows":[[11,"a"],[12,"b"]],
  "paging":{"page":1,"page_size":200,"has_more":true}
}
```

### `table_update_cells(runtime_id, database, table, key, changes)`

> **强烈建议**：编辑必须基于“主键/唯一键定位”，否则拒绝更新并提示。
> **req**

```json
{
  "runtime_id":"uuid","database":"db1","table":"users",
  "key":{"pk_columns":["id"],"pk_values":[11]},
  "changes":[{"column":"name","value":"new"}]
}
```

**res.data**

```json
{ "affected_rows": 1 }
```

### `table_insert_row(runtime_id, database, table, values)`

**req**

```json
{ "runtime_id":"uuid","database":"db1","table":"users","values":{"name":"x","age":18} }
```

### `table_delete_row(runtime_id, database, table, key)`

**req**

```json
{ "runtime_id":"uuid","database":"db1","table":"users","key":{"pk_columns":["id"],"pk_values":[11]} }
```

---

## A.7 导入导出（任务型）

### `export_start(runtime_id, spec)`

**req**

```json
{
  "runtime_id":"uuid",
  "spec":{
    "source":{"type":"table","database":"db1","table":"users"},
    "format":"csv|sql|xlsx",
    "path":"/abs/path/out.csv",
    "options":{"delimiter":",","with_header":true,"encoding":"utf-8","chunk_rows":5000}
  }
}
```

**res.data**

```json
{ "task_id":"uuid" }
```

### `import_start(runtime_id, spec)`

（略，同理，返回 task_id）

### `task_cancel(task_id)`

### `task_get_status(task_id)`

#### 事件（Rust emit → 前端 listen）

* `task_progress`：`{task_id, percent, message, processed_rows}`
* `task_done`：`{task_id, path, elapsed_ms}`
* `task_error`：`{task_id, code, message}`

---

## A.8 历史/模板

### `history_add(runtime_id, item)`

### `history_list(connection_id, q?, limit?, offset?)`

### `template_upsert(template)`

### `template_list()`

---

## A.9 License/VIP

### `license_get_status()`

**res.data**

```json
{ "plan":"free|pro", "expires_at":0, "features":["excel_export","explain_viz"] }
```

### `license_import(license_text_or_path)`

### `license_clear()`

---

# B) MySQL 元数据 SQL 清单（可直接封装成 db/mysql/metadata.rs）

> 说明：以下 SQL 默认使用参数 `:db`, `:table`，Rust 侧自己替换成 `?` 并 bind。

## B.1 服务器信息

```sql
SELECT VERSION() AS version;
SELECT CURRENT_USER() AS current_user;
SELECT @@character_set_server AS charset_server, @@collation_server AS collation_server;
```

## B.2 数据库列表

```sql
SELECT SCHEMA_NAME
FROM information_schema.SCHEMATA
ORDER BY SCHEMA_NAME;
```

## B.3 表/视图列表（含类型、注释、引擎、行数估计）

```sql
SELECT
  TABLE_NAME,
  TABLE_TYPE,
  IFNULL(TABLE_COMMENT,'') AS TABLE_COMMENT,
  IFNULL(ENGINE,'') AS ENGINE,
  IFNULL(TABLE_ROWS,0) AS ROWS_EST
FROM information_schema.TABLES
WHERE TABLE_SCHEMA = :db
ORDER BY TABLE_NAME;
```

## B.4 列信息（字段、类型、默认、nullable、extra、注释）

```sql
SELECT
  ORDINAL_POSITION,
  COLUMN_NAME,
  COLUMN_TYPE,
  DATA_TYPE,
  IS_NULLABLE,
  COLUMN_DEFAULT,
  EXTRA,
  IFNULL(COLUMN_COMMENT,'') AS COLUMN_COMMENT
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = :db AND TABLE_NAME = :table
ORDER BY ORDINAL_POSITION;
```

## B.5 索引信息

```sql
SELECT
  INDEX_NAME,
  NON_UNIQUE,
  SEQ_IN_INDEX,
  COLUMN_NAME,
  INDEX_TYPE
FROM information_schema.STATISTICS
WHERE TABLE_SCHEMA = :db AND TABLE_NAME = :table
ORDER BY INDEX_NAME, SEQ_IN_INDEX;
```

## B.6 外键信息（列映射）

```sql
SELECT
  kcu.CONSTRAINT_NAME,
  kcu.COLUMN_NAME,
  kcu.REFERENCED_TABLE_NAME,
  kcu.REFERENCED_COLUMN_NAME
FROM information_schema.KEY_COLUMN_USAGE kcu
WHERE kcu.TABLE_SCHEMA = :db
  AND kcu.TABLE_NAME = :table
  AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
ORDER BY kcu.CONSTRAINT_NAME, kcu.ORDINAL_POSITION;
```

## B.7 外键约束（更新/删除规则）

```sql
SELECT
  rc.CONSTRAINT_NAME,
  rc.UPDATE_RULE,
  rc.DELETE_RULE
FROM information_schema.REFERENTIAL_CONSTRAINTS rc
WHERE rc.CONSTRAINT_SCHEMA = :db
  AND rc.TABLE_NAME = :table
ORDER BY rc.CONSTRAINT_NAME;
```

## B.8 建表 SQL

```sql
SHOW CREATE TABLE `:db`.`:table`;
```

> 注意：这里 `:db/:table` 需要用反引号安全包裹并做合法性校验（只允许字母数字下划线等），不要用 bind 参数替换对象名。

## B.9 视图定义（可选）

```sql
SELECT VIEW_DEFINITION
FROM information_schema.VIEWS
WHERE TABLE_SCHEMA = :db AND TABLE_NAME = :table;
```

## B.10 存储过程/函数列表（可选）

```sql
SELECT ROUTINE_NAME, ROUTINE_TYPE
FROM information_schema.ROUTINES
WHERE ROUTINE_SCHEMA = :db
ORDER BY ROUTINE_TYPE, ROUTINE_NAME;
```

## B.11 表主键/唯一键（用于“可编辑”判断）

```sql
SELECT
  tc.CONSTRAINT_NAME,
  tc.CONSTRAINT_TYPE,
  kcu.COLUMN_NAME,
  kcu.ORDINAL_POSITION
FROM information_schema.TABLE_CONSTRAINTS tc
JOIN information_schema.KEY_COLUMN_USAGE kcu
  ON tc.CONSTRAINT_SCHEMA = kcu.CONSTRAINT_SCHEMA
 AND tc.TABLE_NAME = kcu.TABLE_NAME
 AND tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME
WHERE tc.CONSTRAINT_SCHEMA = :db
  AND tc.TABLE_NAME = :table
  AND tc.CONSTRAINT_TYPE IN ('PRIMARY KEY','UNIQUE')
ORDER BY tc.CONSTRAINT_TYPE, tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION;
```

---

# C) 编辑器补全策略（缓存/刷新/性能）

目标：**快、准、不拖 UI**。补全分三层：关键字/内置函数（静态） + Schema（缓存） + 上下文（当前 SQL）。

## C.1 补全内容来源

### 1）静态词库（前端内置）

* MySQL Keywords（SELECT/INSERT/UPDATE…）
* 常用内置函数（COUNT/SUM/NOW/JSON_EXTRACT…）
* 片段模板（`sel*` → `SELECT * FROM ${table} LIMIT 200;`）

### 2）Schema 词库（来自后端 meta）

* databases
* tables/views
* columns（按表）
* routines（可选）

### 3）上下文感知（前端解析）

* 从光标位置往前解析：

  * `FROM <here>` → 补表
  * `JOIN <here>` → 补表
  * `SELECT <here>` → 补列/函数
  * `WHERE <here>` → 补列/函数
* 解析别名：`FROM users u` → `u.` 时补 users 的列

> 上下文解析不要求完整 SQL AST，**用轻量规则**就能很好用。

---

## C.2 缓存模型（建议）

### 前端内存缓存（工作台级）

```ts
type Cache = {
  runtimeId: string
  db?: string
  lastRefreshAt: number
  tables: string[]
  columnsByTable: Record<string, string[]>
  routines: string[]
}
```

### 后端缓存（Rust，按 runtime_id + db）

* `meta_list_tables` 返回后写入缓存
* `meta_get_table_schema` 获取列后写入缓存
* 设置 TTL（例如 10 分钟）+ 手动刷新按钮

---

## C.3 刷新策略（关键：别卡）

### 触发点

1. **连接打开**：只拉 databases（快）
2. **用户展开某个 db**：拉 tables/views（一次）
3. **用户第一次引用某张表**（或展开表节点）：拉 columns（按需）
4. **用户点击刷新**：清掉缓存，重新拉当前 db 的 tables（列按需再拉）

### 预热策略（可选，体验更爽）

* 进入 db 后后台预热：

  * 拉 tables
  * 再“最多预热前 N 张常用表”的 columns（比如最近打开过的 10 张）
* 任何预热都走低优先级任务，能取消。

---

## C.4 Completion Provider 规则（落地建议）

### 1）识别场景

从 `model.getValueInRange()` 取光标前 200~500 字符，做：

* 是否在字符串/注释内：是 → 不给 schema 补全（只给关键字）
* token：最后一个词、是否包含 `.`、是否处于 `FROM/JOIN/UPDATE/INTO` 后

### 2）给建议的优先级

* `alias.` → 列（最高）
* `FROM/JOIN/UPDATE/INTO` → 表（最高）
* `SELECT/WHERE/ON/ORDER BY/GROUP BY` → 列 + 函数
* 其他 → 关键字 + 模板片段

### 3）避免一次性塞太多

* 默认 limit 200~500 条（按 relevance 排序）
* 表/列太多时：按输入前缀过滤（`q`）后再展示
* 必要时调用后端：`meta_search_symbols(q, limit)`（比全量拉取更快）

---

## C.5 处理“多库同名表/列”

* 补全展示 label：

  * 表：`users (db1)`
  * 列：`id (users)`
* 插入文本保持简洁：

  * `users`
  * `id`
* 当用户显式输入 `db.table` 时，允许插入 `db1.users`

---

## C.6 权限/性能/失败兜底

* 后端 meta 查询失败：

  * 前端 fallback 到静态关键字补全
  * Schema 区域显示“无权限/连接异常”
* 大库（上万表）：

  * 禁止全量 columns 预热
  * 强制走 `meta_search_symbols`（按 q 搜索）
  * SchemaTree 支持“输入过滤”而不是滚动找

---

## C.7 建议你们做的 2 个“明显爽点”

1. **智能生成 SQL**：在 SchemaTree 右键 “生成 SELECT/INSERT/UPDATE” 并自动带列清单
2. **结果列 → 反向补全**：查询结果表格里右键列名 “复制字段名/插入到编辑器”

---

