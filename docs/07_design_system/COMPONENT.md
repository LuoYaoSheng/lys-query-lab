# QueryLab Design System — 组件库（components）

> 版本：v1.0（2026-09-02）
> 原则：**业务组件优先**（连接卡/Schema 树/数据网格/结果面板/表设计器/diff 预览/危险确认等），通用原子组件为辅。
> 每个组件记录：标识、来源（旧项目源码出处）、构成、状态、交互、DS 令牌引用、V1 变化（对应 P4 B 类优化编号）。
> 组件状态标注：✅ V1 沿用 ｜ 🔧 V1 修复/优化（对应 B1-B13）｜ ➕ V1 新增呈现（不新增商业功能，仅修复性呈现）。

---

## 0. 组件记录总表

| # | 组件 | 类别 | 旧项目来源 | V1 状态 |
|---|------|------|-----------|---------|
| C01 | 连接卡片 ConnectionCard | 业务 | ConnectionManager.svelte | ✅ |
| C02 | 连接表单弹窗 ConnectionFormDialog | 业务 | ConnectionManager.svelte | 🔧 B1 |
| C03 | 测试结果块 TestResultBlock | 业务 | ConnectionManager.svelte | 🔧 B13 |
| C04 | Schema 树 SchemaTree | 业务 | SchemaTree.svelte | 🔧 B12（SVG 图标） |
| C05 | 库表右键菜单 TableContextMenu | 业务 | SchemaTree.svelte | 🔧 B12 |
| C06 | SQL 编辑器 SqlEditor | 业务 | SqlEditor.svelte（CodeMirror 6） | 🔧 B2/B5/B7/B8 |
| C07 | 语句分句预览条 StatementPreviewBar | 业务 | —（B2 新增呈现） | ➕ |
| C08 | 历史侧栏 HistoryPanel | 业务 | SqlEditor.svelte | 🔧 B7 |
| C09 | 片段弹窗 SnippetDialog | 业务 | SqlEditor.svelte | ✅ |
| C10 | 结果面板 ResultsPanel | 业务 | ResultsPanel.svelte | 🔧 B4/B10/B11 |
| C11 | 数据网格 DataGrid | 业务 | DataGrid.svelte | 🔧 B9/B10/B12 |
| C12 | 可编辑单元格 EditableCell | 通用（原子） | ResultsPanel + DataGrid 双实现 | 🔧 B10 统一 |
| C13 | 分页条 Pagination | 通用（原子） | DataGrid.svelte | ✅ |
| C14 | 筛选工具栏 FilterToolbar | 业务 | DataGrid.svelte | 🔧 B12 |
| C15 | 空态占位 EmptyState | 通用（原子） | 各组件散落 | ✅ 规范化 |
| C16 | 加载态 LoadingState | 通用（原子） | 各组件散落 | ✅ 规范化 |
| C17 | 错误块 ErrorBlock | 通用（原子） | 各组件散落 | ✅ 规范化 |
| C18 | 危险确认对话框 DangerConfirm | 业务 | NotificationCenter.svelte | ✅ |
| C19 | Toast 通知 Toast | 通用（原子） | NotificationCenter.svelte | ✅ |
| C20 | 侧滑信息面板 ShellPanel | 业务 | App.svelte shellPanel | ✅ |
| C21 | 表设计器列表格 ColumnsEditor | 业务 | TableDesigner.svelte | ✅ |
| C22 | 表选项区 TableOptionsForm | 业务 | TableDesigner.svelte | ✅ |
| C23 | diff 列差异表 DiffColumnsTable | 业务 | DataSync.svelte | ✅ |
| C24 | SQL 预览块 SqlPreview | 业务 | DataSync.svelte / TableDesigner ALTER 预览 | 🔧 B6 |
| C25 | 差异列表行 DiffListItem | 业务 | DataSync.svelte | ✅ |
| C26 | 批量进度面板 BatchProgressPanel | 业务 | BatchProgressPanel.svelte（未接线） | 🔧 B3 接线 |
| C27 | 文件选择器 FileSelector | 业务 | DatabaseBackup.svelte | ✅ |
| C28 | 表多选网格 TablesPickGrid | 业务 | DatabaseBackup.svelte | ✅ |
| C29 | 进度条 ProgressBar | 通用（原子） | DatabaseBackup / BatchProgressPanel | ✅ |
| C30 | 视图切换器 ViewSwitcher | 业务 | App.svelte | 🔧 B12 |
| C31 | 状态栏 StatusBar | 业务 | App.svelte | ✅ |
| C32 | 图标 Icon（内联 SVG 集） | 通用（原子） | —（B12 新增，替代 emoji） | ➕ |
| C33 | 模态弹窗容器 ModalDialog | 通用（原子） | 各弹窗共用行为 | ✅ 规范化 |

> 业务组件 19 个 + 通用原子组件 14 个，合计 33 项记录。

---

## 1. 业务组件详述

### C01 连接卡片（ConnectionCard）
- 来源：`src/components/ConnectionManager.svelte`。
- 构成：主行（名称，缺省显示 host）+ 副行（host:port）+ 悬停操作组（✎ 编辑 / ⚡ 测试 / ✕ 删除）。
- 状态：默认 / 悬停（操作组显现）/ 选中（左侧 3px `--ql-primary` 条 + 高亮底）/ 测试中（按钮显示 `...`）。
- 令牌：选中条 `--ql-primary`；操作按钮 4px 圆角。
- V1：结构不变；操作图标换内联 SVG（C32）。

### C02 连接表单弹窗（ConnectionFormDialog）🔧B1
- 来源：ConnectionManager.svelte（connection-form）。
- 构成：5 字段（连接名称/主机/端口/用户/密码）+ 操作行（取消/测试/保存）+ 表单内测试结果块；弹窗行为见 C33。
- V1 修复呈现：密码框 placeholder「**留空保持原密码不变**」（编辑态），下方帮助文案「密码仅存系统钥匙串，不落盘」；保存后不清空钥匙串密码（修复 PF-01 静默清空）。字段默认值：driver='mysql'、port=3306（源码 openNewForm）。

### C03 测试结果块（TestResultBlock）🔧B13
- 来源：ConnectionManager.svelte。
- 成功：绿块「连接成功! / 延迟: Xms / 版本: X / 用户: X」；失败：红块「连接失败: {err}」（超时 5 秒文案「连接超时（5秒）」）。
- V1：失败块新增「编辑连接」直达按钮（表单态聚焦主机输入框）——FL-01。

### C04 Schema 树（SchemaTree）
- 来源：`src/components/SchemaTree.svelte`。
- 构成：工具栏（+ 数据库）→ 库节点（▼/▶ + 库名，SVG 化）→ 展开区（+ 新建表虚线行 + 表项列表）。
- 表项：图标（表/视图）+ 名称 + 注释（截断 + title tooltip）；视图名 `--ql-warning` 色。
- 状态：未连接（「请先选择连接」）/ 加载中 / 错误 / 无库（「无可用数据库」）/ 无表（「无表」）/ 展开缓存。
- V1：全部 emoji → C32 SVG 图标；系统库过滤逻辑不变（information_schema/mysql/performance_schema/sys）。

### C05 库表右键菜单（TableContextMenu）
- 来源：SchemaTree.svelte。菜单项：刷新 / 重命名表 / 清空表数据 / 分隔线 / 删除表（红）。点击遮罩或选项后关闭。
- 配套弹窗：新建库（库名 + 字符集 6 项下拉 + 排序规则联动）、重命名（原表名只读 + 新表名）、清空确认、删除确认（不可撤销红文案）。
- V1：图标 SVG 化，菜单结构不变。

### C06 SQL 编辑器（SqlEditor）🔧B2/B5/B7/B8
- 来源：`src/components/SqlEditor.svelte`（CodeMirror 6：oneDark 主题、行号、SQL 方言、折行、searchKeymap）。
- 工具栏：▶ 运行（批量模式变「批量运行」紫底）/ ⚡ 批量模式开关 / 🔒 事务（批量时显示）/ ⟡ 格式化 / 📋 片段 / 🕒 历史（会话/本地）(N) / 清空 / 连接信息 / 快捷键提示。
- 快捷键：Ctrl+Enter 执行、Ctrl+S 格式化、Ctrl+H 历史、Ctrl+K 清空、F1 片段、Ctrl+F 搜索、Tab 缩进。
- V1 修复呈现：① 方言按 `driver` 字段正确映射 MySQL（B5，连接信息显示「MySQL」方言徽标）；② FROM/JOIN 后表名补全接 Schema 树数据（B8，列名补全维持受限说明）；③ 语法色板 = tokens §1.10。

### C07 语句分句预览条（StatementPreviewBar）➕B2
- 旧项目无此组件（parseStatements 未接线，query_execute 按 `;` 裸拆）。
- V1 呈现：执行前在编辑器与结果面板之间显示「将执行 N 条语句 · 首条：SELECT ...（60 字符截断）」；含分号字符串示例正确计为 1 条（识别字符串/注释）。标注「修复：旧项目按 ; 简单分句」。

### C08 历史侧栏（HistoryPanel）🔧B7
- 来源：SqlEditor.svelte（320px 左侧栏）。
- 构成：头（历史记录 + ×）/ 操作行（开启·关闭本地保存 / 清空历史）/ 提示条 historyNotice / 搜索框 / 条目列表（SQL 前 150 字符 + 本地时间）。
- V1 修复呈现：搜索框生效（输入即过滤，大小写不敏感子串；无匹配显示「无匹配历史」）。敏感词拦截提示、上限 100、双模式（会话/本地）不变。

### C09 片段弹窗（SnippetDialog）
- 来源：SqlEditor.svelte（500px 双列，24 个内置片段，点选插入光标处）。V1 结构不变。

### C10 结果面板（ResultsPanel）🔧B4/B10/B11
- 来源：`src/components/ResultsPanel.svelte`。
- 构成：头部（结果 Tab 组「结果 N (行数)」+ CSV/JSON 导出）→ 信息条（耗时 Xms / 查询ID 前 8 位 / 编辑提示）→ 更新消息条 → 五态内容区。
- 值渲染：`--ql-value-*` 四色 + 主键列 🔑（`--ql-pk-*`）。
- V1 修复呈现：① 导出文件名兜底 `{表名|query_前8位}_{时间戳}.csv/json`（B4）；② 只读原因提示「非单表直查或无单列主键，结果只读」（B11）；③ 可编辑单元格与网格统一（B10）；④ 导出按钮有 loading 态（PF-10 附带）。

### C11 数据网格（DataGrid）🔧B9/B10/B12
- 来源：`src/components/DataGrid.svelte`。
- 构成：工具栏（↻ 刷新 / + 新增 / - 删除(N) / CSV / JSON / SQL / 筛选输入 + 列选择 + 筛选 + 清除）→ 消息横幅 → 只读横幅（条件）→ 表格（# 行号 + ☑ 复选 + 数据列）→ 分页条 → 删除确认弹窗。
- 行态四色：`--ql-row-*`；主键列 `--ql-pk-*`；pageSize=50。
- V1 修复呈现：Tab 键保留当前值跳下一列（B9，修复 DataGrid L487 丢值）；图标 SVG 化。

### C12 可编辑单元格（EditableCell）🔧B10 统一
- 来源：ResultsPanel 与 DataGrid 双实现（P4 PF-10）。
- 统一交互：双击进入 → 输入框（+ ✓ 保存 / ✗ 取消 / NULL 开关）→ Enter/Esc 键；保存走 UPDATE…LIMIT 1（后端 query_update_cell 或拼 SQL），成功提示 + 自动刷新。
- 门禁：结果面板=单表直查+单列主键；网格=单列主键；主键列本身不可编辑。
- V1：两处使用同一组件规格与同一门禁文案。

### C13 分页条（Pagination）
- 来源：DataGrid.svelte。构成：「第 X / Y 页，共 N 行」（空表显示「空表」）+ < 上一页 + 页码输入 + > 下一页；跳页校验 1..totalPages。V1 不变。

### C14 筛选工具栏（FilterToolbar）
- 来源：DataGrid.svelte。构成：筛选输入（回车触发）+ 列选择（所有列/具体列）+ 筛选按钮 + 清除按钮（有词才显示）。语义：指定列 LIKE 或全列 OR，转义 `\` 与 `'`。V1 不变（图标化）。

### C15 空态占位（EmptyState）
- 散落来源：各组件。V1 统一规格：SVG 图标（大号 48px `--ql-text-secondary`）+ 主文案（13px `--ql-text-primary`）+ 可选副文案（12px `--ql-text-secondary`）。
- 文案基准沿用源码原文：「暂无连接，点击 + 新建」「无可用数据库」「无表」「暂无历史记录」「此表当前没有数据」「无数据」「执行 SQL 后结果显示在这里」「没有发现结构差异，两个数据库结构相同」；V1 新增「无匹配历史」（B7）。

### C16 加载态（LoadingState）
- 散落来源：各组件。V1 统一规格：spinner（`--ql-primary` 描边旋转）+ 文案（「加载中...」「执行中...」「比较中...」「正在导出...」「导入中...」「保存中...」「测试中...」均源码原文）。

### C17 错误块（ErrorBlock）
- 散落来源：各组件。V1 统一规格：⚠ 图标 + 标题行（`--ql-danger`）+ 详情（pre，可滚动，monospace 11px）；用于 SQL 报错详情、加载失败、校验失败横幅。

### C18 危险确认对话框（DangerConfirm）
- 来源：`src/components/NotificationCenter.svelte`（confirmStore，Promise 化）。
- 构成：遮罩（`rgba(0,0,0,.5)`，点击=取消）+ 对话框（标题/正文/取消/确认）。
- tone：danger（确认钮 `--ql-danger-strong` 底）/ info（`--ql-primary` 底）；Esc=取消；并发确认旧的自动 resolve(false)。
- 适用六处（源码事实）：删除连接、删除表、清空表、网格删除行、执行结构变更、导入删表复选（提示性）。

### C19 Toast（Toast）
- 来源：NotificationCenter + `src/lib/notifications.js`。右上栈（16px 偏移，gap 10px），三级配色见 tokens §1.5；success/info 3.2s、error 4.5s 自动消失；每条 × 手动关闭。

### C20 侧滑信息面板（ShellPanel）
- 来源：App.svelte。右侧 420px 滑出；三分节（设置/帮助/关于）；卡片（`--ql-bg-panel` + `--ql-border-card` + 10px 圆角）；遮罩/Esc/× 关闭。V1 不变。

### C21 表设计器列表格（ColumnsEditor）
- 来源：`src/components/TableDesigner.svelte`。9 列：主键☑/自增☑/列名/类型（6 分组下拉）/长度（按类型显隐）/NULL☑/默认值/注释/删除🗑️。
- 联动：主键→自动 NOT NULL；自增→自动主键+NOT NULL；主键勾选时 NULL 禁用、非主键时自增禁用。
- 校验文案（源码原文）：「至少需要一列」「列名不能为空」「列名 "X" 不符合命名规则」「列 "X" 设置了自增必须是主键」「目前只支持单列主键」「请输入表名」。
- V1：索引区只读标注「索引（只读，编辑能力规划中）」——PF-14 C 类如实呈现，不虚构编辑入口。

### C22 表选项区（TableOptionsForm）
- 来源：TableDesigner.svelte。2×2 网格：引擎（InnoDB/MyISAM/MEMORY/ARCHIVE/CSV）/ 字符集（utf8mb4/utf8/latin1/gbk/big5 等）/ 排序规则（联动）/ 表注释。V1 不变。

### C23 diff 列差异表（DiffColumnsTable）
- 来源：DataSync.svelte。六列：列名/状态（新增·绿/删除·红/修改·黄）/源类型/目标类型/可空/默认值；行按状态着色。V1 不变。

### C24 SQL 预览块（SqlPreview）🔧B6
- 来源：DataSync 详情面板、TableDesigner ALTER 预览。
- 规格：pre + monospace + 语法色（`--ql-code-*`）；可滚动。
- V1 修复呈现：「修改列类型」预览正确插值目标库表名（B6，修复 `{targetDatabase}` 字面量 bug，标注修复）。

### C25 差异列表行（DiffListItem）
- 来源：DataSync.svelte。构成：复选框 + 表名 + 状态徽标（新增·绿 / 删除·红 / 有差异·黄，左色条同色）+ 列变化标签（+N 列/-N 列/~N 列/索引差异）；点击行开详情。V1 不变。

### C26 批量进度面板（BatchProgressPanel）🔧B3
- 来源：`src/components/BatchProgressPanel.svelte`（组件完整、未接线）。
- 构成：进度条 + 计数「N / M 语句执行完成」+ 统计（总计/成功/失败）+ 语句列表（类型图标：SELECT🔍 INSERT➕ UPDATE✏️ DELETE🗑️ CREATE🆕 ALTER🔧 DROP💣 事务🔒 其他📄；状态 current 蓝/success 绿/failed 红/pending 半透明）+ 错误详情块。
- V1 修复呈现：批量模式执行时自动唤起该面板逐条推进——标注「PAGE012 · 修复：旧项目组件未接线」。

### C27 文件选择器（FileSelector）
- 来源：DatabaseBackup.svelte。readonly 输入框 + 「浏览 SQL」按钮（系统 open 对话框）。V1 不变。

### C28 表多选网格（TablesPickGrid）
- 来源：DatabaseBackup.svelte。checkbox 网格 + 全选/取消全选；选库后默认全选。V1 不变。

### C29 进度条（ProgressBar）
- 来源：DatabaseBackup / BatchProgressPanel。轨道 `--ql-bg-hover` + 填充 `--ql-primary`（批量面板用语义色）；配状态文本。V1 不变。

### C30 视图切换器（ViewSwitcher）🔧B12
- 来源：App.svelte。按钮：SQL 查询（默认）/ 数据网格（选中表后）/ 设计表（选中表后）/ 结构对比（预览）/ 备份还原；active 态按视图主色（tokens §1.6）。
- V1：emoji 全部替换为 C32 SVG 图标，语义与条件显隐不变。

### C31 状态栏（StatusBar）
- 来源：App.svelte `.app-footer`（24px，`--ql-primary` 底）。左 statusMessage（Ready / No connection / Connected to X / Executing... / Query completed in Xms / Query failed / Batch executing [with transaction]... / Batch completed: N statements, Xms / Batch execution failed），右「平台: {os}」。V1 不变。

### C33 模态弹窗容器（ModalDialog）
- 行为基准（page-spec 一.5）：× 关闭、遮罩点击关闭、Esc 关闭，role=dialog + aria-modal；适用连接表单、新建库、删表/清空/重命名确认、片段、删除行确认、右键菜单遮罩。
- 尺寸：确认框 420px；表单弹窗 ~500px（ConnectionManager）；10px 圆角。

---

## 2. 通用原子组件（摘要）

- C12/C13/C15/C16/C17/C19/C29/C32/C33 见上文对应条目；原子组件只定义行为与令牌，不绑定业务数据。
- C32 图标集规格见 `assets.md`（内联 SVG 关键图标清单）。

---

## 3. 组件-页面映射（快速索引）

| 页面 | 使用组件 |
|------|----------|
| PAGE001 壳 | C30 视图切换器、C31 状态栏、C20 侧滑面板、C18/C19 全局层、新建表指示条 |
| PAGE002 连接 | C01、C02、C03、C18、C19 |
| PAGE003 Schema | C04、C05、C15、C16、C17、C18 |
| PAGE004 编辑器 | C06、C07、C08、C09 |
| PAGE005 结果 | C10、C12、C15、C16、C17 |
| PAGE006 网格 | C11、C12、C13、C14、C15、C16、C17、C18 |
| PAGE007 设计器 | C21、C22、C16、C17、C24（ALTER 预览） |
| PAGE008 对比 | C23、C24、C25、C16、C17、C18 |
| PAGE009 备份 | C27、C28、C29、C16、C17 |
| PAGE010 通知 | C18、C19 |
| PAGE011 面板 | C20 |
| PAGE012 批量进度 | C26、C29 |
