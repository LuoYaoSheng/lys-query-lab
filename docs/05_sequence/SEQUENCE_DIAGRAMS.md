# QueryLab 时序图集（SEQUENCE_DIAGRAMS）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 事实来源：`src-tauri/src/commands/{connection,query,metadata,backup,app}.rs`（逐文件核实）、`src/App.svelte`、`src/components/*.svelte`、`docs/04_architecture/SYSTEM_ARCH.md` §3 关键数据流、`docs/product-review/PRODUCT_LOGIC_REVIEW.md`（PL-01/PL-02）、`docs/product-review/USER_FLOW_REVIEW.md`。现状（As-Is）与已知断裂均标注；未实测行为标【未知】。

---

## 图 1：新建连接 + 测试连接（现状）

```mermaid
sequenceDiagram
    participant U as 用户
    participant CM as ConnectionManager.svelte
    participant FE as conn_test / conn_upsert
    participant ST as ConnectionStorage
    participant KR as 系统钥匙串
    participant MY as MySQL
    U->>CM: 打开新建表单，填 5 字段
    U->>CM: 点「测试连接」
    CM->>FE: invoke conn_test(ConnectionInfo 含手输密码)
    FE->>MY: OptsBuilder.pass(手输密码) + 5s 超时
    MY-->>FE: VERSION()/USER()/DATABASE()
    FE-->>CM: ConnectionTestResult{latency_ms, server_version, user, default_db}
    CM-->>U: 绿色结果块（延迟/版本/用户）
    U->>CM: 保存
    CM->>FE: invoke conn_upsert(ConnectionInfo)
    FE->>FE: id 为空则 Uuid::new_v4
    FE->>ST: upsert（密码非空 → set_connection_password）
    ST->>KR: 写入密码（service=com.i2kai.querylab.connection, account=id）
    ST->>ST: save_all（password.clear() 后序列化落盘 connections.json）
    FE-->>CM: 返回 id
    CM->>FE: invoke conn_list 刷新
    CM-->>U: 列表更新 + toast
    Note over CM,KR: ⚠️ 已知断裂 PL-01：后续执行链路从不读钥匙串<br/>带密码账户「测试成功→使用失败」
```

## 图 2：执行查询（现状，query_execute）

```mermaid
sequenceDiagram
    participant U as 用户
    participant E as SqlEditor.svelte
    participant A as App.svelte
    participant Q as query_execute
    participant MY as MySQL
    participant R as ResultsPanel.svelte
    U->>E: 编写 SQL，Ctrl+Enter（选中优先）
    E->>A: dispatch execute(sql)
    A->>A: 检查 selectedConnection（未选 → toast 请先选择连接）
    A->>Q: invoke query_execute({connection, sql})
    Q->>Q: 按 ; 裸分句 ⚠️ PF-02（拆坏含分号字符串/注释）
    loop 每条语句
        Q->>MY: 新建连接（无池化；密码为空串 ⚠️ PL-01）执行
        MY-->>Q: 列元数据 + 行流 / 错误
    end
    Q-->>A: QueryResultSet{setIndex, columns, meta{elapsedMs,affectedRows}, chunks, paging} / Err(String)
    A->>A: 状态栏 Query completed in Xms / Query failed
    A->>R: queryResult / queryError（五态渲染，多 Tab）
    R->>R: 历史入库（SqlEditor：去重置顶，敏感词过滤）
    Note over Q,MY: ⚠️ 无超时无取消（UF-02）：慢查询时「执行中」态无终点
```

## 图 3：浏览表结构（meta_get_table_schema，点表进网格）

```mermaid
sequenceDiagram
    participant U as 用户
    participant T as SchemaTree.svelte
    participant A as App.svelte
    participant G as DataGrid.svelte
    participant M1 as meta_list_tables
    participant M2 as meta_get_table_schema
    participant MY as MySQL(information_schema)
    U->>T: 展开库
    T->>M1: invoke meta_list_tables(connection, database)
    M1->>MY: SELECT ... FROM information_schema.TABLES
    MY-->>M1: TableInfo[]{name, type(BASE TABLE/VIEW), comment, engine, rowsEst}
    M1-->>T: 表+视图列表（图标区分，缓存）
    U->>T: 点击表
    T->>A: dispatch selectTable(database, table)
    A->>A: setViewMode('grid')
    A->>G: 挂载 DataGrid
    G->>M2: invoke meta_get_table_schema(connection, database, table)
    M2->>MY: information_schema.COLUMNS / STATISTICS（/ KEY_COLUMN_USAGE）
    MY-->>M2: Column[] / Index[]（⚠️ 外键恒空数组 PF-13）
    M2-->>G: TableSchema{columns, indexes, foreign_keys, create_sql}
    G->>G: 单列 PRIMARY 门禁判定（只读横幅或开放编辑）
    G->>Q2: query_execute(COUNT + LIMIT 50 OFFSET)
    Q2-->>G: 数据分页渲染
```

## 图 4：备份导出（db_export，现状含断裂）

```mermaid
sequenceDiagram
    participant U as 用户
    participant B as DatabaseBackup.svelte
    participant DLG as @tauri-apps/plugin-dialog save()
    participant DE as db_export
    participant MY as MySQL
    participant FS as 本地文件系统
    U->>B: 备份还原视图 → 📤 导出备份 Tab
    U->>B: 选库（加载表，默认全选）→ 勾选
    U->>B: 开始导出
    B->>DLG: save()（选保存路径）
    DLG--xB: ⚠️ 必然 reject：Rust 侧未注册 tauri-plugin-dialog（PL-02）
    B->>B: catch → 静默降级 browserDownload（URL.createObjectURL）
    Note over B,FS: 降级下载落点不可控（UF-03），无 toast 区分主/降级路径
    B->>DE: invoke db_export(ExportParams{connection, database, tables})
    DE->>MY: SHOW CREATE TABLE + SELECT（每表数据上限 10000 行）
    MY-->>DE: 结构 DDL + 数据行
    DE->>DE: 拼 SQL 文本（结构+数据）
    DE-->>B: ExportResult{file_path 或 sql 文本}
    B-->>U: 成功绿色结果块（位置/表数）+ toast / 取消则「已取消导出」
```

> 对照——导入侧（db_import）：`open()` 同样必然 reject 且无降级，仅 console.error，importFile 恒空，「开始导入」永久禁用（UF-06/PL-02）。

## 图 5：表设计器保存（新建 meta_create_table / 编辑 diff→ALTER）

```mermaid
sequenceDiagram
    participant U as 用户
    participant D as TableDesigner.svelte
    participant MC as meta_create_table / query_execute(ALTER)
    participant MY as MySQL
    participant T as SchemaTree.svelte
    U->>D: 新建模式（+ 新建表）或编辑模式（📋 设计表，加载 originalColumns 快照）
    U->>D: 增删列/改属性（联动：主键→NOT NULL·自增→主键）/改表选项
    D->>D: hasChanges=true → 头部「有未保存的更改」
    Note over D: ⚠️ 关闭按钮直接丢弃，无确认（PL-03/UF-05）
    U->>D: 保存
    D->>D: 结构校验（至少一列/命名规则/自增须主键/仅单列主键）
    alt 校验失败
        D-->>U: 错误横幅阻止提交
    else 新建模式
        D->>MC: invoke meta_create_table(CreateTableParams{connection, database, table, columns, options})
        MC->>MY: CREATE TABLE ...
        MY-->>MC: ok
        MC-->>D: 「表 '{db}.{table}' 创建成功」
    else 编辑模式
        D->>D: originalColumns 与当前 columns 做 diff
        D->>MC: 生成 ALTER 序列（CHANGE COLUMN/MODIFY/DROP/ADD/表选项）逐句执行
        MC->>MY: ALTER TABLE ...
        MY-->>MC: ok
        MC-->>D: 保存成功
    end
    D->>T: dispatch 完成事件 → Schema 树局部刷新
    D-->>U: toast + 切换网格视图
```

## 附：图间共性的已知断裂索引

| 断裂点 | 影响的图 | 编号 |
|--------|----------|------|
| 凭据链路（密码 skip_serializing + 执行链路不读钥匙串） | 图 1/2/3/4/5 全部执行类步骤 | PL-01 |
| plugin-dialog 未注册（save/open 必然 reject） | 图 4（及导出类流程） | PL-02 |
| 按 `;` 裸分句 | 图 2（批量/多语句场景） | PF-02 |
| 无超时无取消 | 图 2 | UF-02 |
| 外键恒空 | 图 3 | PF-13 |
| 关闭无确认 | 图 5 | PL-03 |
