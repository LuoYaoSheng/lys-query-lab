# QueryLab 状态模型评审（ST）

> 评审依据：《AI 产品重构逻辑评审规范 v1.0》· 2026-09-03
> 输入文档清单：`docs/01_reverse/REVERSE_ANALYSIS.md` §④（各页状态字段）、`docs/02_product/PAGE_SPEC.md` §一.2/§二（状态列表）、`docs/06_review/PRODUCT_REVIEW.md`（P4 交叉）、源码核实 `src/App.svelte`（工作区状态全集）、`src/components/ResultsPanel.svelte`、`src/components/DataGrid.svelte`、`src-tauri/src/commands/*.rs`（连接模型）。
> 评审口径：必要性（状态是否承载用户可感知差异）/ 重复（多份等价状态）/ 冲突（状态间可矛盾）/ 缺失（用户需要但无状态表达）。

---

## 一、状态机文本图

### 1.1 连接态（应用级）

```text
[未连接] --点击连接项--> [已选中(乐观 Connected)] --每次命令--> (每命令独立新建 MySQL 连接, 命令毕即断)
    ^                                                          |
    └------------------ 删除当前连接 onConnect(null) -----------+
缺失转移：
  [已选中] --服务器宕机/网络断/密码失效--> 无「连接中断」态（UI 仍显示 Connected to X，
                                         直到下一次命令失败才以错误态间接暴露）
说明：selectedConnection 只是前端持有的配置对象；statusMessage='Connected to X' 在点击瞬间写入
      （App.svelte L104），不含任何真实连通性校验（ST-01）。
```

### 1.2 查询态（App 级，queryLoading/queryResult/queryError 三元组）

```text
[idle] --executeQuery--> [executing(Executing...)] --成功--> [idle+结果]（statusMessage=Query completed in Xms）
                                      |--失败--> [idle+错误]（Query failed）
                                      |--无超时无取消--> [executing] 可无限期停留（UF-02）
批量执行复用同一三元组：statusMessage 换文案（Batch executing.../with transaction.../completed/failed）
缺失：无 per-statement 状态（PAGE012 未接线，ST-04）；无「结果过期(stale)」标记（切连接时 reset 清空，尚可）
```

### 1.3 结果态（ResultsPanel，props 驱动五态）

```text
① loading「执行中...」+ spinner
② error   ⚠️ + 错误 pre 全文
③ empty   「执行 SQL 后结果显示在这里」
④ message 无列无块「执行成功，影响 N 行」
⑤ data    数据表（+ 结果 Tab N 组、可编辑提示）
次级状态：activeSetIndex（Tab）、editingCell（编辑器开合）、updateMessage（更新结果条）、
          tableSchema/schemaKey（编辑门禁输入）、lastQueryId（新结果重置编辑态）
死状态：exportLoading 恒 false（ST-03）
```

### 1.4 视图态（viewMode × 渲染条件）

```text
viewMode ∈ {query, grid, design, sync, backup}
渲染规则：query 视图当 viewMode==='query' || (!currentTableName && !isCreatingNewTable)（App.svelte L342）
隐式优先级：query 兜底 —— viewMode 与实际渲染可解耦（ST-02，当前无触发路径的防御性问题）
条件可见按钮：数据网格/📋 设计表 仅 currentTableName 非空时出现
```

### 1.5 编辑态（网格/结果面板单元格）

```text
[idle] --双击(门禁通过)--> [editing] --Enter/✓--> [saving(后端 UPDATE)] --> [idle+刷新+反馈]
                                |--Esc/✗--> [idle]
网格特有：Tab 跳列（PF-09 丢值）；新行 isNew 走 INSERT 分支
```

### 1.6 设计器态

```text
[loading(编辑模式)] → [clean] --任何修改--> [dirty(hasChanges 指示)] --保存--> [clean+刷新]
[dirty] --点关闭--> 直接丢弃（无确认转移，PL-03）
```

---

## 二、状态必要性 / 重复 / 冲突 / 缺失分析

| 维度 | 分析 |
|------|------|
| **必要** | 五态结果模型、危险操作 tableOperating（按钮禁用+进行中文案）、expandedDbs/tablesData 缓存、lastQueryId 重置机制（防止旧 Tab 编辑残留）均必要且实现正确。statusMessage 作为全局进度线（Ready/Connected/Executing/耗时/失败）信息密度合理 |
| **重复** | ① 编辑门禁双实现：ResultsPanel.isEditable 与 DataGrid.supportsRowMutation 各自判定（P4 PF-10 交叉），状态输入不同（editableTableName vs tableSchema）但语义相近——重复且口径可漂移；② 更新反馈双套：ResultsPanel.updateMessage 与 DataGrid.updateMessage 同名同形异实现；③ 连接信息三处展示（编辑器 connection-info、状态栏、连接列表高亮）——可接受的冗余，不立问题 |
| **冲突** | ① statusMessage（乐观 Connected）与每命令实连的现实冲突（ST-01）；② viewMode 与渲染条件表达式的潜在不一致（ST-02）；③ 查询态三字段由单次执行原子更新，无冲突；ResultsPanel 新旧结果切换以 queryId 为锚，正确 |
| **缺失** | ① 连接中断/失效态（ST-01 一体）；② 批量 per-statement 态（ST-04）；③ 结果 stale 态（当前以切连接全清空替代，够用）；④ 应用级「 busy/操作互斥」概念——导出与查询可并发触发同一状态栏文案相互覆盖【未知——并发场景未实测，源码无互斥锁】 |

---

## 三、问题清单

### ST-01 「Connected to X」为乐观断言，无连通性语义【B·新】
- 当前设计：点击连接项瞬间 `statusMessage = 'Connected to ${conn.name}'`（App.svelte L104），未做任何连通校验；后端每命令独立新建连接（mysql_async Conn::new per command，query.rs L116、metadata.rs L226 等），无持久连接可探测。
- 问题：状态栏宣称的「已连接」与系统事实（下一次命令才知道能不能连上）脱节；断网/服务器宕机后 UI 长期停留在 Connected，用户对后续失败的归因被误导（以为是 SQL 写错）。
- 影响：状态可信度受损；与 UF-07（连接类异常无指引）叠加放大排障成本。
- 建议方向：选中连接时执行一次轻量探测（复用 conn_test）再置 Connected；或文案改为「已选中 X」并在首次命令成功后升级为 Connected；断连错误时回退状态。

### ST-02 viewMode 与渲染条件解耦，query 隐式兜底【C·新】
- 当前设计：渲染表达式 `{#if viewMode === 'query' || (!currentTableName && !isCreatingNewTable)}`（App.svelte L342）——当 viewMode 为 grid/design 但 currentTableName 被清空时，激活按钮显示 grid/design 而实际渲染查询视图。
- 问题：当前代码所有清空 currentTableName 的路径（resetWorkspaceState）都同步把 viewMode 归位 query，故**现阶段不可触发**；但该表达式将不变式隐式埋在多个调用点的同步纪律里，任何新增清空路径都会破坏按钮-视图一致。
- 影响：防御性缺陷，重构时易引入「按钮亮着 A 视图却是 B」。
- 建议方向：重构时以单一派生函数收敛「实际视图」计算，或断言不变式；V1 原型无需处理。

### ST-03 exportLoading 恒 false，死状态【B·新（逆向 9.4.7 已记小缺陷，P4 未单列）】
- 当前设计：ResultsPanel 声明 exportLoading（L18）但从未置真（逆向报告⑨.4.7；本评审复核确认无赋值点，仅 L340/L348 disabled 绑定）。
- 问题：导出（含降级下载大结果集）无 loading 反馈，用户可重复点击；状态声明与行为脱节属死代码级状态。
- 影响：小；与 PL-02（导出主链路）修复时一并处理成本最低。
- 建议方向：随 PL-02 落地时接线导出 loading；或删除该状态。

### ST-04 批量执行缺 per-statement 状态【B·P4 PF-03 交叉】
- 当前设计：批量执行复用单查询三元组（queryLoading/queryResult/queryError），唯一外显是状态栏一行汇总（App.svelte L202）；BatchProgressPanel 的 current/success/failed/pending 四态模型（组件内定义完整）无数据源。
- 问题：多语句执行的中间态对用户不可见；失败时无法定位第几条（错误态只含整体错误）。
- 影响：P4 已定 B（接线）；本报告从状态模型角度确认该组件状态设计合理，接线即可用。
- 建议方向：维持 P4 B3 方案。

### ST-05 结果五态覆盖完整【D·正面】
- 当前设计：loading/error/empty/message/data 五态覆盖查询生命周期全部输出形态（page-spec PAGE005；源码核实一致），空结果集（0 行 Tab）与「执行成功影响 N 行」区分正确。
- 问题：无。
- 建议方向：不动，V1 延续（P4 PP-07 的「状态栏概要+面板详情」分层维持 D）。

### ST-06 statusMessage 单字符串无级别，错误可被后续操作覆盖【C·新】
- 当前设计：statusMessage 是无级别的单一字符串（App.svelte L25），Ready/Executing/失败共用一个槽位；错误详情仅在结果面板。若查询失败（Query failed）后用户切视图或触发其他操作，失败提示即被覆盖（如 setViewMode 自动重查会改写）。
- 问题：全局状态线缺少「最近一次错误」的保持机制；与 P4 PP-07（D：分层合理）不完全冲突——分层文案合理，但**覆盖丢失**是新角度。
- 影响：低频场景信息丢失；对长会话排障略有影响。
- 建议方向：观察为主；若 V1 改造状态栏，可加「最近错误」粘性提示（有未查看错误时状态栏图标化标记）。

---

## 四、小结

状态模型骨架健康（五态结果、危险操作进行态、缓存与重置纪律），共 6 项（B3、C2、D1）。最实质的是 ST-01：连接态的「乐观断言」是全应用唯一承诺了事实但无法兑现的状态，其修复（选中即探测）成本低于收益；ST-03 随 PL-02 顺手处理；ST-02/06 为观察项。
