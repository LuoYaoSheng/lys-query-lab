# QueryLab 产品体验审查报告（P4）

> 审查阶段：《旧 App AI 重构 SOP v1.0》P4 产品体验审查
> 审查日期：2026-09-02
> 审查对象：旧项目源码（`src/` + `src-tauri/`，权威工作区）与 P1-P3 产物（`docs/01_reverse/REVERSE_ANALYSIS.md`、`docs/02_product/PRD.md`、`docs/02_product/PAGE_SPEC.md`、`prototype/app-prototype.html`（现位于 `prototype/v0-old/`）、`docs/09_test/COVERAGE_CHECKLIST.md`、`docs/09_test/HTML_V0_ACCEPTANCE.md`）。
> 审查方式：逐文件源码核对 + 文档交叉比对，所有结论可追溯到文件路径与行号；未实测数据库连接（无可用 MySQL 实例，连接类行为以源码逻辑推断，涉及处已注明）。
> 本报告是 P6（V1 新版原型）B 类优化落地与 P7（开发架构）公共能力划分的直接输入。

---

## 〇、问题分级定义与统计

| 级别 | 定义 | 处置 | 数量 |
|------|------|------|------|
| **A** | 文档勘误 | 直接修改 P1-P3 产物文档并在本文第七节记录 | 3 项（含 P6 归位引发的路径勘误 1 项） |
| **B** | 体验优化 | V1 新版原型落地（呈现修复后规格，不复制 bug 行为） | 13 项 |
| **C** | 需用户决策 | 默认不做、留档待确认（多为 PRD/API 文档规划未实现项的范围决策） | 14 项 |
| **D** | 观察不动 | 现状合理或为旧项目有意设计，如实保留 | 8 项 |

---

## 一、功能问题（重复 / 缺失入口 / 不合理实现）

### PF-01 编辑连接不重输密码会清空钥匙串密码【B】
- 现象：`conn_list` 返回的连接不携带密码（`src-tauri/src/db/driver.rs` L64-65 `#[serde(default, skip_serializing)] password`），编辑表单密码恒空；此时直接保存，`ConnectionStorage::upsert` 走「密码为空 → delete_connection_password」分支（`src-tauri/src/storage/connections.rs` L76-80），钥匙串密码被删除，后续连接全部失败且无任何提示。
- 体验影响：数据丢失级（用户必须重新输入密码才能恢复），且发生过程完全静默。
- 处置：**B**。V1 呈现修复后语义——编辑弹窗密码框 placeholder「留空保持原密码不变」，且保存时不再清空；P7 `api-design.md` 给出 `conn_upsert` 密码保留语义契约（`password` 为空且 `id` 非空 = 保持原密码）。

### PF-02 query_execute 按 `;` 简单分句，含分号的字符串/注释会被拆坏【B】
- 现象：后端 `src-tauri/src/commands/query.rs` L120-124 仅 `split(';')`；`SELECT * FROM t WHERE name = 'a;b'` 会被拆成两条非法语句。前端 `parseStatements`（`src/components/SqlEditor.svelte` L371-467，支持字符串/块注释/行注释/DELIMITER 检测）已实现并 export（L545）但**从未被调用**。
- 处置：**B**。V1 落地两件事：① 执行前显示「分句预览」（识别出的语句条数与首条预览，正确处理字符串内分号）；② P7 将分句逻辑归属定为「前端解析、后端按语句数组执行」的契约修正（`query_execute` 入参增加 `statements: string[]` 或后端复用同一算法，详见 api-design）。

### PF-03 批量执行进度面板（PAGE012）未接线【B】
- 现象：`src/components/BatchProgressPanel.svelte`（382 行）组件完整（进度条、总计/成功/失败、逐条语句状态、错误详情），但 `src/App.svelte` 未 import；批量执行时用户只能看状态栏一行文字，多语句执行无逐条反馈。
- 处置：**B**。V1 中将批量执行与进度面板接线：批量模式执行时唤起进度面板，逐条推进（原型以模拟数据驱动）；标注「PAGE012 · 修复：旧项目组件未接线」。

### PF-04 结果导出文件名可为空（`_export.csv`）【B】
- 现象：`src/components/ResultsPanel.svelte` L225/L245 文件名取 `tableName`（仅单表直查时有值），复杂查询导出文件名为 `_export.csv` / `_export.json`。
- 处置：**B**。V1 导出默认文件名兜底：`{表名或query_前8位}_{时间戳}.csv`，不得为空。

### PF-05 方言检测字段错配，MySQL 高亮退化为 Standard SQL【B】
- 现象：`src/components/SqlEditor.svelte` L249 读 `connection.dialect`，实际序列化字段为 `driver`（`driver.rs` L59-60 `#[serde(rename = "driver")]`），MySQL 连接的编辑器高亮恒为 Standard SQL 方言（MySQL 特有函数/关键字不高亮）。
- 处置：**B**。V1 呈现修复后规格：按 `driver` 字段正确映射 MySQL 方言，工具栏连接信息同时显示方言徽标「MySQL」；标注「F014 · 修复：方言检测字段错配」。

### PF-06 结构对比详情面板 SQL 预览不插值（`{targetDatabase}` 字面量）【B】
- 现象：`src/components/DataSync.svelte` L660-665「修改列类型」SQL 预览写在 JS 模板字符串内但未加 `${}`，输出 `ALTER TABLE \`{targetDatabase}\`.\`{detailData.table}\`` 字面量；同面板「添加/删除列」段（Svelte 模板）正常插值，行为不一致。
- 处置：**B**。V1 按修复后规格正确插值（v0 原型已如此，V1 延续并保留标注）。

### PF-07 历史面板搜索框无过滤逻辑【B】
- 现象：`src/components/SqlEditor.svelte` L618-624 搜索输入框存在（placeholder「搜索历史...」）但未绑定任何状态与过滤，历史列表恒全量。
- 处置：**B**。V1 实现过滤：输入即过滤（大小写不敏感子串匹配 SQL 文本），无匹配显示「无匹配历史」空态。

### PF-08 SQL 自动补全表名源未接线、列名补全缺失【B】
- 现象：`updateTableNames`（SqlEditor L533）无调用方，`tableNames` 恒空数组——FROM/JOIN 后的表名补全实际不可用；列名补全未实现（L533-537 注释自认"可选"）。
- 处置：**B**（限定范围）：V1 将表名补全接线到 Schema 树数据（当前连接的库/表列表），列名补全仍以受限说明标注（列名需要按表拉取 schema，属实现成本项，超 V1 原型范围的部分在 P7 中列为接线任务）。不虚构列名补全。

### PF-09 网格单元格编辑 Tab 键丢值【B】
- 现象：`src/components/DataGrid.svelte` `handleKeydown` 中 Tab 跳到下一列时携带 `value: ''`（L487），编辑已有单元格时按 Tab 会清空当前输入显示。
- 处置：**B**。V1 按修复后规格呈现：Tab 保留当前输入值并跳到下一列（v0 原型已如此，V1 延续并保留标注）。

### PF-10 单元格编辑逻辑双实现（结果面板 vs 数据网格）【B-公共能力】
- 现象：`ResultsPanel.svelte` 与 `DataGrid.svelte` 各自实现一套「双击编辑 → Enter 保存 / Esc 取消 → UPDATE…LIMIT 1」逻辑，门禁判定也各自实现（`isEditable` L196-205 vs `supportsRowMutation`），规则相近但口径不同（结果面板要求单表直查，网格要求单列主键）。
- 处置：**B**（公共能力提取，非 UI 变化）：P5 组件库沉淀「可编辑单元格/EditableCell」组件与「编辑门禁」判定；P7 模块拆分将两处合并到共享模块。V1 原型层面表现为一致的编辑交互。

### PF-11 导出能力三处重复实现【B-公共能力】
- 现象：CSV/JSON 导出逻辑在 `ResultsPanel.svelte`（L225/L245）与 `DataGrid.svelte`（exportCSV/exportJSON）各写一遍，SQL INSERT 导出在 DataGrid 内，备份导出在后端 `db_export`——四处生成逻辑互不复用，文件命名/转义规则不一致风险。
- 处置：**B**（公共能力提取）：P5/P7 沉淀统一「导出服务」（文件名规则、CSV 转义、JSON 结构、SQL INSERT 生成），V1 原型体现统一的导出交互与命名规则。

### PF-12 meta_get_schema_tree 后端命令无调用方【C】
- 现象：`src-tauri/src/main.rs` 已注册，前端全局无 invoke（逆向报告⑨已证）。
- 处置：**C**。P7 `api-design.md` 建议从 V1 命令清单剔除或接线（SchemaTree 已用 meta_list_databases + meta_list_tables 组合实现同一目标），剔除属契约变更，需用户确认。

### PF-13 外键定义存在但恒返回空数组【C】
- 现象：`ForeignKey` 类型已定义（`db/types.rs` L30-36），`meta_get_table_schema` 恒返回 `[]`（`metadata.rs` L368）。
- 处置：**C**。表设计器/Schema 树是否需要展示外键属范围决策；V1 不虚构外键展示，PRD 维持「未实现」口径。

### PF-14 表设计器索引无可视化编辑【C】
- 现象：`TableDesigner.svelte` L144-145 加载并保存 `indexes` 变量，但无任何编辑 UI。
- 处置：**C**。索引管理产品化属新功能范围决策；V1 在设计器中以「索引（只读展示，编辑能力规划中）」如实标注，不虚构编辑入口。

---

## 二、页面问题（信息层级 / 操作路径 / 页面职责）

### PP-01 连接编辑弹窗密码框无「留空后果」提示【B】
- 现象：编辑连接时密码恒空且无占位提示（PF-01 的 UI 侧）；用户无从知道「不填会怎样」。
- 处置：**B**。V1 密码框 placeholder「留空保持原密码不变」+ 下方帮助文案「密码仅存系统钥匙串，不落盘」。

### PP-02 结果面板编辑门禁无解释（静默只读）【B】
- 现象：复杂 SQL / 无单列主键时结果面板直接不出现编辑 UI（page-spec PAGE005「门禁不满足时只读（无提示类按钮不出现）」），用户不知道为什么不能编辑。
- 处置：**B**。V1 在结果信息条增加只读原因提示：「非单表直查或无单列主键，结果只读」（可编辑时显示编辑提示，与旧项目一致）。

### PP-03 视图切换器图标不统一（emoji 混排）【B-DS】
- 现象：「📋 设计表」「🔍 结构对比（预览）」「💾 备份还原」用 emoji，「SQL 查询」「数据网格」纯文字；侧栏/树/工具栏大量 emoji 图标（📁📊👁️🔑🗑️等），跨平台渲染不一致。
- 处置：**B**。P5 Design System 沉淀内联 SVG 图标集（数据库/表/视图/列/索引/连接/导出等），V1 全面替换 emoji；语义不变。

### PP-04 历史侧栏与编辑器主次关系（320px 固定侧栏）【D】
- 现象：历史侧栏固定 320px 从左侧挤入，编辑区被动变窄，无拖拽调节。
- 处置：**D**。可用性可接受，V1 保持开合交互不变（布局细节留给开发实现）。

### PP-05 shell 面板三合一（设置/帮助/关于）【D】
- 现象：三个入口共用一个 420px 侧滑面板容器，内容为静态卡片。
- 处置：**D**。`docs/RELEASE_VERIFICATION_2026-04-22.md` 明确「轻量版面板，不是完整独立页面」为有意设计，V1 如实保留。

### PP-06 备份页「导出类型」radio 与「仅支持 SQL」说明冗余【D】
- 现象：导出类型单选只有一项可选（结构+数据 SQL），格式说明固定「仅支持 SQL (.sql)」。
- 处置：**D**。对外承诺口径（RELEASE_CHECKLIST）如此，冗余但诚实；V1 保留。

### PP-07 状态栏错误信息只有一行（`Query failed`）【D】
- 现象：错误详情在结果面板 pre 中，状态栏只显示结论。信息分层合理（状态栏=概要，面板=详情）。
- 处置：**D**。V1 延续该分层，仅统一配色与图标。

### PP-08 连接列表操作按钮悬停才现（✎⚡✕）【D】
- 现象：hover 显示操作按钮（桌面惯例，Navicat/DBeaver 同款）。
- 处置：**D**。桌面端合理，V1 延续并保证焦点可达（键盘可操作）。

---

## 三、流程问题（跳转 / 路径 / 异常处理）

### FL-01 连接测试失败无「去编辑」直达路径【B】
- 现象：测试失败红色块只显示错误（连接失败/超时 5 秒），用户需自行关闭弹窗（列表态）再找 ✎ 才能修正参数。
- 处置：**B**。V1 在失败结果块内增加「编辑连接」快捷按钮（表单态直接聚焦主机输入框）。

### FL-02 切回查询视图自动重查并静默覆盖编辑器内容【B】
- 现象：`App.svelte` `setViewMode('query')` 且有 currentTableName 时自动执行 `SELECT * FROM … LIMIT 1000;` 并 `setSql` 回填（L233-246）——用户正在编写的 SQL 会被无提示替换。
- 处置：**B**。V1 保留自动重查功能（F055 不得丢失），但增加「已自动回填当前表查询」提示条（可关闭），消除静默覆盖感。

### FL-03 切换/删除连接后编辑器 SQL 残留【B】
- 现象：`resetWorkspaceState()`（App.svelte L62-72）重置结果/视图/当前表，但**不清空编辑器 SQL**（SqlEditor 自持状态）；切换到新连接后旧 SQL 仍在，误执行风险。
- 处置：**B**。V1 呈现修复后语义：切换连接时编辑器保留内容但顶部出现「当前 SQL 来自上一个连接会话」警示条；删除当前连接则清空编辑器（工作区整体重置的一致性）。

### FL-04 SQL 报错无错误定位【C】
- 现象：错误以全文 pre 展示（含 errno/位置信息原文），但无编辑器行内定位/标记（旧 PRD 5.3 规划「错误定位」未实现）。
- 处置：**C**。错误定位属规划未实现项的范围决策；V1 保持全文展示 + 统一错误样式。

### FL-05 执行中无法取消查询【C】
- 现象：无 query_cancel（API 文档 A.5 规划；core/state.rs TaskManager TODO）；长查询只能等待。
- 处置：**C**。需后端任务管理架构支撑，P7 tech-architecture 列为候选增强，范围待用户决策。

### FL-06 批量执行无逐条进度（同 PF-03）【B】
- 处置：并入 PF-03。

### FL-07 导出失败降级浏览器下载在桌面壳内行为不明【C】
- 现象：`fs_write_file` 失败时降级 `URL.createObjectURL` 浏览器下载（ResultsPanel L225 等），Tauri webview 内下载落点不可控。
- 处置：**C**。降级策略属技术实现决策；V1 原型以「导出成功 toast + 失败错误提示」呈现，不模拟降级细节。

### FL-08 结构对比「新增表」仅生成注释提示【D】
- 现象：新增表在同步 SQL 中仅注释「需要手动创建」（DataSync L219-224），为发布边界的有意设计。
- 处置：**D**。对外承诺口径（不宣称全自动），V1 如实呈现并在详情面板明示。

### FL-09 危险操作确认链路完整【D-正面确认】
- 现象：删连接/删表/清空/网格删行/结构变更/导入删表六处危险操作全部走全局 danger 确认，遮罩/Esc 均取消——无遗漏。
- 处置：**D**。V1 延续该模式并沉淀为 P5「危险操作确认」模式。

### FL-10 备份导出取消路径反馈【D-正面确认】
- 现象：save 对话框取消 → notifyInfo「已取消导出」，无错误噪音。
- 处置：**D**。V1 延续。

---

## 四、公共能力识别（Component / Module / Service / Config）

> 该清单为 P5 Design System 与 P7 模块拆分的直接输入。标注「已有」「重复实现」「缺失」三种现状。

### 4.1 Component（UI 组件）

| 组件 | 现状 | 来源 | V1 处理 |
|------|------|------|---------|
| 连接卡片（列表项：名称/host:port/操作组） | 已有 | ConnectionManager.svelte | DS 沉淀（含选中态/悬停态规范） |
| 连接表单弹窗（5 字段 + 测试结果块） | 已有 | ConnectionManager.svelte | DS 沉淀（密码留空语义见 PF-01） |
| Schema 树（库/表/视图三级 + 展开缓存） | 已有 | SchemaTree.svelte | DS 沉淀（SVG 图标化） |
| 数据网格（列头+类型副行/行号/主键列/值渲染） | 已有·**双实现**（ResultsPanel/DataGrid） | 两组件 | DS 统一为「数据网格 DataGrid」 |
| 可编辑单元格（双击/Enter/Esc/NULL 开关） | 已有·**双实现** | 两组件 | DS 统一「可编辑单元格」 |
| 分页条（上下页/页码输入/总数） | 已有 | DataGrid.svelte | DS 沉淀 |
| 筛选工具栏（输入+列选择+筛选/清除） | 已有 | DataGrid.svelte | DS 沉淀 |
| 表设计器列表格（9 列 + 联动） | 已有 | TableDesigner.svelte | DS 沉淀 |
| diff 预览（列差异六列表 + SQL 预览） | 已有 | DataSync.svelte | DS 沉淀 |
| 危险确认对话框 | 已有 | NotificationCenter.svelte | DS 沉淀（P5 模式） |
| Toast 栈 | 已有 | NotificationCenter.svelte | DS 沉淀 |
| 侧滑信息面板 | 已有 | App.svelte shellPanel | DS 沉淀 |
| 批量进度面板 | 已有**未接线** | BatchProgressPanel.svelte | V1 接线（PF-03） |
| 空态占位（多种文案） | 已有·散落 | 各组件 | DS 统一「空态」规范 |
| 结果 Tab 组（多结果集） | 已有 | ResultsPanel.svelte | DS 沉淀 |
| 文件选择器（readonly 输入+浏览） | 已有 | DatabaseBackup.svelte | DS 沉淀 |

### 4.2 Module（业务模块）

连接管理 / Schema 浏览 / SQL 编辑（含历史、片段、格式化、补全）/ 查询执行调度 / 结果展示与导出 / 表数据 CRUD（网格）/ 表设计（新建+diff ALTER）/ 结构对比 / 备份还原 / 通知与确认 / 应用信息与面板。共 11 个模块，与 `src/components/` 一一对应（执行调度在 App.svelte，P7 建议独立）。

### 4.3 Service（前端服务层）

| 服务 | 现状 | 说明 |
|------|------|------|
| 通知服务 | 已有 | `src/lib/notifications.js`（toast/confirm store，被 8 组件复用） |
| Tauri invoke 封装 | **缺失** | 各组件直接 `invoke('<cmd>', {...})`，无统一 API 层、无错误归一（P7 建议提取 `src/lib/api/`） |
| SQL 工具（分句/格式化/转义） | 散落 | parseStatements+format 在 SqlEditor 内；escapeSqlString 在 DataGrid 内——建议提取 `sqlUtils` |
| 导出服务（CSV/JSON/SQL 生成+文件名） | **重复实现×3** | 见 PF-11，建议提取 `exporter` |
| 编辑门禁判定 | **重复实现×2** | isEditable / supportsRowMutation，建议提取 `editGuard` |
| 状态栏消息 | 内联 | App.svelte 内联字符串，建议集中定义 |

### 4.4 Config（配置与常量）

| 配置 | 真实值 | 来源 |
|------|--------|------|
| 系统库过滤 | information_schema / mysql / performance_schema / sys | App.svelte loadDatabases |
| 分页大小 | pageSize = 50 | DataGrid.svelte |
| 查询截断 | maxRows = 1000 | App.svelte executeQuery |
| 历史上限 | MAX_HISTORY = 100 | SqlEditor.svelte |
| 连接测试超时 | 5 秒 | connection.rs conn_test |
| 备份每表行数上限 | 10000 | backup.rs |
| 字符集下拉 | utf8mb4/utf8/latin1/gbk/big5 等 6 项+联动排序规则 | SchemaTree.svelte |
| 类型分组 | 整数/浮点数/字符串/二进制/日期时间/其他（6 组） | TableDesigner.svelte |
| 存储引擎 | InnoDB/MyISAM/MEMORY/ARCHIVE/CSV | TableDesigner.svelte |
| 敏感词正则 | password/secret/token/api[_-]?key/access[_-]?key/private[_-]?key/credential | SqlEditor.svelte L33-41 |
| localStorage 键 | querylab_sql_history / querylab_sql_history_enabled | SqlEditor.svelte L31-32 |
| 17 个 Tauri 命令 | app_get_info、conn_list、conn_upsert、conn_delete、conn_test、db_export、db_import、fs_write_file、meta_list_databases、meta_list_tables、meta_get_table_schema、meta_get_schema_tree、meta_create_database、meta_create_table、query_execute、query_update_cell（+fs_write_file 计 17） | src-tauri/src/main.rs |
| 快捷键 | Ctrl+Enter 执行 / Ctrl+S 格式化 / Ctrl+H 历史 / Ctrl+K 清空 / F1 片段 / Ctrl+F 搜索 | SqlEditor.svelte keymap |

---

## 五、分级处置汇总

### 5.1 B 类（V1 落地清单——V1 原型必须体现）

| # | 来源 | 优化点 | V1 呈现 |
|---|------|--------|---------|
| B1 | PF-01/PP-01 | 连接编辑密码不清空 | 密码框「留空保持原密码不变」+ 帮助文案 |
| B2 | PF-02 | 分句预览 / 正确分句 | 执行前分句预览条（N 条语句·首条预览，字符串内分号正确识别） |
| B3 | PF-03/FL-06 | 批量进度面板接线 | 批量执行唤起 PAGE012 面板逐条推进 |
| B4 | PF-04 | 导出文件名默认值 | `{表名或query_XXXX}_{时间戳}` 兜底 |
| B5 | PF-05 | 方言高亮修正呈现 | 按 driver 映射 MySQL 方言 + 连接信息方言徽标 |
| B6 | PF-06 | diff SQL 预览正确插值 | 修改列 SQL 预览插值目标库表名 |
| B7 | PF-07 | 历史搜索过滤生效 | 输入即过滤 + 「无匹配历史」空态 |
| B8 | PF-08 | 表名补全接线 | FROM/JOIN 后表名候选来自 Schema 树（列名仍受限说明） |
| B9 | PF-09 | 网格 Tab 不丢值 | Tab 保留值跳列 |
| B10 | PF-10/11 | 编辑/导出能力统一 | 统一可编辑单元格交互 + 统一导出命名与交互 |
| B11 | PP-02 | 结果只读原因提示 | 「非单表直查或无单列主键，结果只读」 |
| B12 | PP-03 | 图标系统化 | 内联 SVG 替换 emoji（P5 assets） |
| B13 | FL-01/02/03 | 流程补正 | 测试失败「编辑连接」直达；自动回填提示条；切换连接 SQL 残留警示 |

### 5.2 C 类（默认不做，留档待用户决策）

| # | 事项 | 来源 | 决策问题 |
|---|------|------|----------|
| C1 | SSL/SSH 连接参数 | PRD 5.1 扩展位 | V1 是否纳入连接表单 |
| C2 | SQL 编辑器多 Tab / 会话管理 | PRD 5.3 / SessionManager TODO | 是否进入 V1 范围 |
| C3 | 查询取消 query_cancel / 分页拉取 query_fetch_more | API 文档 A.5 | 后端任务架构是否重做 |
| C4 | 结果列头排序 | PRD 5.4 | V1 是否支持 |
| C5 | 收藏 SQL / 模板管理 / 后端 history_* | PRD 5.7 / API A.8 | 历史能力是否升级 |
| C6 | CSV/TSV 数据导入 | PRD 5.6 | 导入范围 |
| C7 | License/VIP 体系 | API A.9 / errors.rs feature_required | 是否保留规划 |
| C8 | meta_get_schema_tree 剔除 | PF-12 / main.rs | 契约变更确认 |
| C9 | src-ui 旧副本删除 | README / 逆向 2.4 | 仓库清理确认 |
| C10 | 连接池 / 运行时连接句柄 | API A.3 | 性能架构是否重做 |
| C11 | 符号搜索 meta_search_symbols | API A.4 | 是否实现 |
| C12 | PostgreSQL / SQLite 支持 | driver 占位 | V1 是否扩库 |
| C13 | 外键展示 | PF-13 | meta_get_table_schema 是否补实现 |
| C14 | 统一返回结构 {ok,data,error,trace_id} | API A.0 | P7 契约采用何种错误模型 |

### 5.3 D 类（观察不动）

D1 状态栏/结果面板错误信息分层；D2 shell 面板轻量三合一；D3 备份页格式说明冗余（诚实口径）；D4 结构对比新增表仅注释（承诺边界）；D5 系统库过滤；D6 连接列表 hover 操作（桌面惯例）；D7 历史侧栏固定宽度；D8 危险确认链路（正面基线，延续）。

---

## 六、C 类待用户决策清单（汇总）

见 5.2 表格 C1-C14。其中 **C8/C9/C14 同时是 P7 架构文档的强制留档项**。

---

## 七、A 类勘误执行记录

> A 类修改仅限昨晚新建的 docs 文件，逐项记录如下。

| # | 文件 | 勘误内容 | 性质 |
|---|------|----------|------|
| A1 | `docs/02_product/PAGE_SPEC.md` 一.1 | 状态栏消息「Batch executing(with transaction)...」更正为「Batch executing with transaction...」（源码原文无括号，App.svelte L189） | 事实勘误 |
| A2 | `docs/01_reverse/REVERSE_ANALYSIS.md` 附、`docs/02_product/PRD.md` §8、`docs/02_product/PAGE_SPEC.md` §四、`docs/09_test/COVERAGE_CHECKLIST.md` 头部与§四、`docs/09_test/HTML_V0_ACCEPTANCE.md` 头部 | 旧原型路径 `prototype/app-prototype.html` 统一更正为 `prototype/v0-old/app-prototype.html`（P6 归位引发），并补充 V1 原型 `prototype/v1-new/app-prototype.html` 指向 | 路径勘误 |
| A3 | `docs/01_reverse/REVERSE_ANALYSIS.md` 1.2 架构图 + 2.3 main.rs 行 | 「17 个已注册 command」更正为 **16 个**：逐项清点 `src-tauri/src/main.rs` `generate_handler!` 列表恰为 16 个命令名（原文列出的名称清单本身只有 16 项，与「17」计数自相矛盾）；`docs/07_design_system/ASSETS.md` §2.1 同步留档 | 事实勘误（计数） |

---

## 八、审查结论

- 功能/页面/流程三类问题合计 **31 项**（功能 14 + 页面 8 + 流程 10，其中 3 项合并计入），全部完成 A/B/C/D 分级。
- 旧项目主链路（连接→浏览→查询→编辑）体验完整，危险操作防护与空态覆盖是**正面基线**；核心缺陷集中在「静默失败」类（密码清空、分句拆坏、导出空文件名、门禁静默只读）。
- B 类 13 项全部纳入 V1 原型落地清单（5.1），作为 P6 的直接输入；C 类 14 项留档，其中 3 项为 P7 强制留档。
- 本报告不改动任何源码；A 类勘误仅修改 P1-P3 产物文档并已在第七节留痕。
