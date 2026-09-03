# QueryLab 页面交互规格说明（page-spec）

> 对应 `docs/01_reverse/REVERSE_ANALYSIS.md`（PAGE001-PAGE012 / F001-F057）与 `docs/02_product/PRD.md`。
> 本文所有交互均来自旧项目源码实测（src/、src-tauri/），未脑补。

---

## 一、全局交互约定

1. **窗口与布局**：桌面单窗口（1400×900，最小 1000×600）。三段式：顶栏（48px）→ 主区（左侧栏 280px + 工作区）→ 底部状态栏（24px，蓝色底）。深色主题（背景 #1e1e1e / 面板 #252526 / #2d2d2d，主色 #007acc，成功 #4ec9b0/#2da042，危险 #f48771/#d73a49）。（src/App.svelte、tauri.conf.json）
2. **状态栏**：左侧 statusMessage（Ready / No connection / Connected to X / Executing... / Query completed in Xms / Query failed / Batch executing with transaction... / Batch executing... / Batch completed: N statements, Xms / Batch execution failed），右侧 `平台: {os}`。（App.svelte L189）
3. **危险操作确认**：一律走全局确认对话框（`confirmAction`，tone=danger 红色确认按钮），遮罩点击与 Esc 均视为取消。适用：删除连接、删除表、清空表、网格删除行、执行结构变更。（notifications.js、各组件）
4. **轻量反馈**：操作成功/失败/信息用 Toast（success 绿 3.2s / error 红 4.5s / info 蓝 3.2s，可 × 关闭）。表单内测试结果允许局部结果块（绿/红）。（NotificationCenter.svelte）
5. **弹窗通用行为**：所有模态弹窗支持 × 关闭、遮罩点击关闭、Esc 关闭（role=dialog + aria-modal；连接表单、新建库、删表/清空/重命名、片段、删除行确认、shell 面板均一致）。
6. **右键菜单**：仅表项有；点击遮罩或选择菜单项后关闭；菜单项：🔄 刷新 / ✏️ 重命名表 / 🗑️ 清空表数据 / 分隔线 / ❌ 删除表（红）。（SchemaTree.svelte）
7. **编辑门禁总则**：
   - 结果面板：仅"单条 SQL 且形如 `SELECT * FROM \`db\`.\`table\`[ LIMIT n]`、结果 sets=1、表存在单列 PRIMARY、主键列出现在结果列"时可编辑，主键列本身不可编辑。
   - 数据网格：仅"表存在单列 PRIMARY 且出现在列中"时开放更新/删除（复选框可用），否则显示只读横幅；插入始终可用（需已加载表结构）。
8. **主键视觉**：主键列头/单元格紫底（#252835/#c586c0），结果面板主键列头带 🔑。
9. **值渲染规则**：NULL → 斜体灰「NULL」；数字 → 绿；字符串 → 橙；二进制 → 灰「[N bytes]」/「[Binary]」；网格经 escapeHtml 安全输出。
10. **键盘**：全局 Esc 关闭最上层弹窗/菜单；编辑器快捷键 Ctrl+Enter 执行、Ctrl+S 格式化、Ctrl+H 历史、Ctrl+K 清空、F1 片段、Tab 缩进；单元格编辑 Enter 保存 / Esc 取消 /（网格）Tab 跳下一列。
11. **加载/空/错误三态文案基准**（取自源码原文）：「加载中...」「执行中...」「无可用数据库」「无表」「暂无历史记录」「此表当前没有数据」「无数据」「执行 SQL 后结果显示在这里」「执行成功，影响 N 行」「请先选择连接」。
12. **系统库过滤**：information_schema / mysql / performance_schema / sys 在数据库列表中一律隐藏。

---

## 二、每页 11 维度规格

### PAGE001 主工作台（应用壳）

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE001〔F053/F054/F055/F050〕 |
| 页面目标 | 承载侧栏与五种工作视图，管理工作区全局状态 |
| 进入条件 | 应用启动即入；无前置条件 |
| 页面结构 | 顶栏（logo / 设置 / 帮助 / 关于 / 版本号）＋ 侧栏（连接区≤50% 高 + Schema 区弹性）＋ 工作区（视图切换器 + 视图容器）＋ 状态栏 |
| 组件列表 | NotificationCenter、ConnectionManager、SchemaTree、SqlEditor、ResultsPanel、DataGrid、TableDesigner、DataSync、DatabaseBackup、shellPanel |
| 按钮列表 | 设置、帮助、关于；视图按钮：SQL 查询 / 数据网格（条件）/ 📋 设计表（条件）/ 🔍 结构对比（预览）/ 💾 备份还原 |
| 按钮行为 | 导航按钮 → 打开对应侧滑面板；「SQL 查询」→ setViewMode('query')，若存在 currentTableName 则自动执行 `SELECT * FROM \`db\`.\`table\` LIMIT 1000;` 并 setSql 回填编辑器；其余 → 切换对应视图（设计按钮紫 / 对比蓝 / 备份红 active 样式） |
| 状态列表 | viewMode: query/grid/design/sync/backup；未连接（No connection）；已连接（Connected to X）；执行中；执行完成/失败；批量执行中/完成；isCreatingNewTable 时显示「📝 新建表: {db}」绿色指示条 |
| 跳转关系 | 连接区选中 → PAGE002/003 激活；点表 → PAGE006；新建表 → PAGE007（新建模式）；各视图按钮 → PAGE004/006/007/008/009 |
| 异常处理 | app_get_info/conn_list 失败 console.error（界面仍可用）；未连接执行 SQL → queryError「请先选择连接」+ 状态栏 No connection；loadDatabases 失败静默（console） |
| 数据展示规则 | 版本号 `v{version} ({build})`；状态栏平台为 OS 名（macos/windows/linux） |

### PAGE002 连接管理

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE002〔F001-F005〕 |
| 页面目标 | 连接的增删改查、测试与选择 |
| 进入条件 | 应用启动即渲染于侧栏上区 |
| 页面结构 | 连接头（标题 + ＋）→ 测试结果块（列表态）→ 连接列表（项：名称 + host:port + ✎ ⚡ ✕）→ 空状态；弹窗：表单（连接名称/主机/端口/用户/密码）+ 表单内测试结果块 + 操作行（取消/测试/保存） |
| 组件列表 | 连接项按钮、btn-icon×3、connection-form 弹窗、test-result 块 |
| 按钮列表 | ＋（新建）；✎（编辑）；⚡（测试，测试中显示 ...）；✕（删除，红）；取消；测试（测试中…）；保存（保存中…）；表单 × |
| 按钮行为 | ＋ → 重置表单（driver='mysql'、port=3306）开弹窗；✎ → 回填（密码恒空）开弹窗；⚡ → conn_test，成功绿块「连接成功!\n延迟: Xms\n版本: X\n用户: X」，失败红块「连接失败: err」；✕ → 全局危险确认「确定要删除连接 "X" 吗？」→ conn_delete → toast「连接已删除」，若删当前连接 → onConnect(null) 清空工作区；保存 → conn_upsert → conn_list 刷新 → 关弹窗；表单内测试 → 以表单数据（id=__test__）conn_test |
| 状态列表 | 列表空（「暂无连接，点击 + 新建」）；测试中（按钮禁用 + ...）；保存中；当前选中项（左侧 3px 蓝条 + 高亮底）；testResult 成功/失败 |
| 跳转关系 | 选中连接 → 激活 Schema 区并重置工作区；删除当前连接 → 全部视图回初始态 |
| 异常处理 | 保存失败「保存失败: err」（红块）；测试失败含 5s 超时「连接超时（5秒）」；删除失败 toast；弹窗遮罩/Esc 关闭 |
| 数据展示规则 | 名称缺省显示 host；副行 host:port；编辑回填兼容 driver/driver_type、defaultDb/default_db 字段名 |

### PAGE003 Schema 浏览树

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE003〔F006-F013〕 |
| 页面目标 | 库/表/视图浏览与表级操作、新建入口 |
| 进入条件 | 侧栏下区常驻；加载数据需已选连接 |
| 页面结构 | 工具栏（+ 数据库）→ 树（库节点 → 展开子区：新建表行 + 表项列表/加载中/无表）→ 弹窗×4（新建库 / 右键菜单 / 删除表确认 / 重命名 / 清空确认） |
| 组件列表 | 树节点（▼/▶ 📁 库名）、add-table-btn（+ 新建表 虚线）、table-item（📊/👁️ + 名称 + 注释）、dialog 系列、context-menu（4 项 + 分隔线） |
| 按钮列表 | + 数据库；+ 新建表；右键：刷新/重命名表/清空表数据/删除表；弹窗：取消/创建、取消/确认删除、取消/确认（重命名）、取消/确认清空 |
| 按钮行为 | + 数据库 → 弹窗（字符集 6 选联动排序规则）→ meta_create_database → toast「数据库创建成功」→ 刷新并展开新库；+ 新建表 → dispatch createTable → App 切 design（新建模式）；刷新 → 清缓存重载表；重命名（视图 → notifyInfo「视图不支持重命名」；表 → 弹窗 → RENAME TABLE → toast 成功刷新）；清空（视图拦截；表 → TRUNCATE → toast「表数据已清空」）；删除 → DROP → toast「表删除成功」→ 刷新表列表 |
| 状态列表 | 未连接「请先选择连接」；加载中...；错误文案；库展开/收起；表加载中/无表；tableOperating（按钮禁用 + 「删除中.../重命名中.../清空中...」） |
| 跳转关系 | 点表 → PAGE006；+ 新建表 → PAGE007；操作完成原地刷新树 |
| 异常处理 | 加载库失败显示 error 文本；DDL 失败 toast「XX失败: err」；空库名 notifyError「请输入数据库名称」；未连接建库 notifyError「请先连接数据库」 |
| 数据展示规则 | 表/视图图标区分；视图名称黄色（#dcdcaa）；注释 span 截断 100px + title tooltip；系统库过滤 |

### PAGE004 SQL 查询视图（编辑器）

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE004〔F014-F022〕 |
| 页面目标 | SQL 编写、执行（普通/批量/事务）与辅助工具 |
| 进入条件 | 默认视图；或从其他视图点「SQL 查询」 |
| 页面结构 | 工具栏 → 主区（左：历史侧栏 320px（可开合）｜右：CodeMirror 编辑器）；片段弹窗（500px 双列） |
| 组件列表 | btn-run（▶ 运行/批量运行，绿/紫）、btn-batch（⚡ 批量模式）、btn-transaction（🔒 事务，批量时显示）、btn-format（⟡ 格式化）、btn-snippet（📋 片段）、btn-history（🕒 历史（会话/本地）(N)）、btn-clear（清空）、connection-info、shortcuts-hint、history-panel（header/actions/notice/search/list）、snippet-dialog |
| 按钮列表 | 运行；批量模式（toggle active）；事务（toggle active）；格式化；片段；历史；清空；历史面板：开启/关闭本地保存、清空历史、×；片段项 24 个；弹窗 × |
| 按钮行为 | 运行/Ctrl+Enter → 取选中 SQL 优先 → 空则忽略 → 未连接 notifyError → 存历史 → 按 batchMode 分流 onExecute/onBatchExecute；批量+事务 → App 拼 `START TRANSACTION;\n{sql}\nCOMMIT;`；格式化/Ctrl+S → 关键字大写换行缩进；片段/F1 → 弹窗点选插入光标处（+\n）；历史/Ctrl+H → 侧栏开合，点条目回填；本地保存开关 → 写 localStorage 偏好 + 提示文案；清空历史 → 清数组与 localStorage；清空/Ctrl+K → setSql('') |
| 状态列表 | 编辑器空（占位符「-- 输入 SQL 语句...」）；批量模式（运行按钮变「批量运行」紫底）；事务开启（蓝底 active）；历史开/合；历史空「暂无历史记录」；historyNotice 提示（会话级/敏感拦截/开关反馈）；persistHistory 徽标（本地/会话） |
| 跳转关系 | 执行结果 → PAGE005；历史/片段原地操作 |
| 异常处理 | 未连接 notifyError；执行错误交由结果面板；历史解析失败 console.error；敏感 SQL（password/secret/token/api_key/access_key/private_key/credential）不落本地并提示 |
| 数据展示规则 | 历史 SQL 显示前 150 字符 + 省略号 + 本地时间；片段显示名称 + 前 50 字符预览；连接信息「{name/host} ({dialect/SQL})」或「未连接」 |

### PAGE005 查询结果面板

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE005〔F023-F027、F056〕 |
| 页面目标 | 结果展示（多集）、导出、受限单元格编辑 |
| 进入条件 | 位于 SQL 查询视图下半区；内容随执行变化 |
| 页面结构 | 头部（结果 Tab 组 + 导出按钮组｜信息条）→ 更新消息条 → 内容区（五态之一） |
| 组件列表 | result-tab×N、btn-export×2、results-info（耗时/查询ID/编辑提示）、update-message、数据表（th 列名+类型、td 单元格、cell-editor 输入 + ✓/✗/NULL） |
| 按钮列表 | 结果 Tab；CSV；JSON；单元格编辑：✓ 保存、✗ 取消、NULL 开关 |
| 按钮行为 | Tab → 切 activeSetIndex（重置编辑态）；CSV/JSON → save 对话框 → fs_write_file（失败降级浏览器下载）；双击可编辑单元格 → 编辑器；✓/Enter → query_update_cell（is_null 随 NULL 开关）→ 成功消息 500ms 后 onRefresh；✗/Esc → 取消；NULL → 切换 isNull（输入框半透明斜体） |
| 状态列表 | ① 加载「执行中...」+ spinner ② 错误 ⚠️ + 错误 pre ③ 空态「执行 SQL 后结果显示在这里」④ 无列无块「执行成功，影响 N 行」⑤ 数据表；可编辑提示「双击单元格编辑」；更新成功/失败消息条 |
| 跳转关系 | 编辑保存后触发 App onRefresh（重查当前表 SQL） |
| 异常处理 | 查询错误全文显示（可滚动）；更新失败「更新失败: err」；schema 加载失败静默降级只读；新结果（queryId 变化）重置 Tab/编辑/消息 |
| 数据展示规则 | 列头两行（名称 + 类型尾巴）；主键列头 🔑 + 特殊底色；耗时 Xms；查询 ID 取前 8 位；行数取 meta.affectedRows |

### PAGE006 数据网格视图

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE006〔F028-F037〕 |
| 页面目标 | 表数据分页浏览/筛选/CRUD/导出 |
| 进入条件 | SchemaTree 点表，或选中表后点「数据网格」 |
| 页面结构 | 工具栏（左：刷新/＋新增/－删除(N)/分隔/CSV/JSON/SQL；右：筛选输入 + 列选择 + 筛选 + 清除）→ 消息横幅 → 只读横幅（条件）→ 表格（# / ☑ / 数据列）→ 分页条 → 删除确认弹窗 |
| 组件列表 | btn-toolbar×7、filter-input、filter-column（select）、btn-filter/btn-clear、message-banner、readonly-banner、table（row-num/checkbox/单元格 + cell-input）、pagination（信息 + </页码输入/>）、confirm-dialog |
| 按钮列表 | ↻ 刷新；+ 新增；- 删除 (N)（选中 0 或无主键或加载中禁用）；CSV/JSON/SQL（无列禁用）；筛选；清除（有筛选词才显示）；分页 < > 与页码输入；弹窗 取消/删除 |
| 按钮行为 | 刷新 → 重新加载（清选择与临时行）；新增 → 追加临时空行（主键列置 ''）并进入首列编辑，逐列 Enter/Tab，末列保存触发 INSERT（排除自增列）→「插入成功」+刷新；删除 → 确认弹窗「确定要删除选中的 N 行数据吗？此操作不可撤销！」→ 逐行 DELETE LIMIT 1 →「成功删除 N 行」；筛选 → WHERE 拼接（指定列或全列 OR，LIKE %词%，转义 \ 与 '）重查计数与数据并回第 1 页；导出 → 生成内容 + save 对话框落盘；双击非主键单元格（需单列主键）→ 编辑 → Enter 保存 UPDATE…LIMIT 1 |
| 状态列表 | 加载中...；错误块；无列「无数据」；空表（列头 + 📭 此表当前没有数据 + 表名提示）；行态：悬停/选中（绿底）/编辑行（绿底）/新行（蓝底）；分页信息「第 X / Y 页，共 N 行」或「空表」 |
| 跳转关系 | 由 PAGE003 进入；关闭本视图经视图切换器 |
| 异常处理 | 加载失败错误块；增删改失败红色横幅（含 err）；无单列主键：readonly-banner + 编辑/删除按钮禁用 + 双击提示「当前表缺少单列主键，暂不支持网格编辑/删除」 |
| 数据展示规则 | 列头名称 + 类型；行号列；NULL 斜体灰；HTML 转义输出；主键列紫底 |

### PAGE007 表设计器

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE007〔F038-F042〕 |
| 页面目标 | 表结构可视化新建/编辑 |
| 进入条件 | 新建模式：SchemaTree「+ 新建表」；编辑模式：选中表后「📋 设计表」 |
| 页面结构 | 头部（图标+标题（新建含表名输入）/未保存指示/关闭/保存）→ 错误横幅 → 列表区（section 头 + ＋添加列 + 9 列表格）→ 表选项区（2×2 网格） |
| 组件列表 | table-name-input、unsaved-indicator、btn（关闭/保存）、error-banner、columns-table（主键☑/自增☑/列名/类型 select（6 分组）/长度/NULL☑/默认值/注释/删除🗑️）、options-grid（引擎/字符集/排序/注释） |
| 按钮列表 | 关闭；保存（保存中...）；+ 添加列；每行 🗑️ 删除列 |
| 按钮行为 | 添加列 → 追加默认行（VARCHAR/255/可空）并置 hasChanges；删除列 → 移除并置 hasChanges；勾主键 → 自动 NOT NULL；勾自增 → 自动主键+NOT NULL；主键勾选时 NULL 复选框禁用、非主键时自增复选框禁用；保存（新建）→ 校验 → meta_create_table → toast 后端消息 → onRefresh；保存（编辑）→ diff 生成 ALTER 序列逐条执行 → toast「表结构保存成功」；关闭 → onClose（App 复位新建态并回网格） |
| 状态列表 | 新建模式（📝 新建表 - {db}. + 输入框）/编辑模式（📋 表设计 - db.table）；加载中...（编辑模式）；保存中；hasChanges 指示；新列行绿底（非 original-col）；错误横幅 |
| 跳转关系 | 新建成功 → App 刷新对应库并切回网格查看新表；编辑保存 → 刷新网格数据 |
| 异常处理 | 校验失败文案：「至少需要一列」「列名不能为空」「列名 "X" 不符合命名规则」「列 "X" 设置了自增必须是主键」「目前只支持单列主键」「请输入表名」；执行失败「保存失败: err」 |
| 数据展示规则 | 类型按分组下拉（整数/浮点/字符串/二进制/日期时间/其他）；长度输入仅 CHAR/VARCHAR/BINARY/VARBINARY/DECIMAL/NUMERIC 显示；编辑模式回填列属性与表选项（从 create_sql 解析 ENGINE/CHARSET/COLLATE/COMMENT） |

### PAGE008 结构对比（预览）

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE008〔F043-F047〕 |
| 页面目标 | 两库结构差异分析 + 结构 SQL 生成/执行（预览形态） |
| 进入条件 | 点「🔍 结构对比（预览）」 |
| 页面结构 | 头部（标题 + ×）→ 配置卡（mode-note + 源库 select → 目标库 select + 模式 select（禁用）+ 开始比较）→ 错误块 → 结果区（统计条 6 项 → 差异列表（全选 + 行）→ 详情面板 → 操作按钮） |
| 组件列表 | 三个 select、btn-compare、summary-item×6、diff-item（checkbox + 名称 + 徽标 + ±~列/索引标签）、detail-panel（列差异 6 列 grid + 索引提示 + SQL 预览 pre）、btn-copy/btn-export/btn-sync |
| 按钮列表 | 开始比较（比较中...）；×；全选；差异行复选框；行点击（详情）；📋 复制 SQL；💾 导出 SQL；▶ 执行同步（同步中...，选中 0 禁用）；详情 × |
| 按钮行为 | 开始比较 → 校验（必选/不同库）→ 并行拉两库表列表 → 共同表逐对拉 schema 比较列与索引 → 汇总差异并默认全选；复制 → navigator.clipboard + toast；导出 → 下载 sync_{src}_to_{dst}_{ts}.sql；执行同步 → 全局 danger 确认（不可撤销/建议先备份）→ 逐句 query_execute → toast「结构变更已执行完成」+ dispatch syncComplete（刷新树与库列表） |
| 状态列表 | 比较中；比较前（仅配置区）；syncError；比较后（统计+列表）；无差异「没有发现结构差异，两个数据库结构相同」；行选中态；详情开/合；同步中 |
| 跳转关系 | × → 返回 SQL 查询视图；syncComplete → 侧栏 Schema 刷新 |
| 异常处理 | 未选两库 notifyError「请选择源数据库和目标数据库」；同库 notifyError「源数据库和目标数据库不能相同」；比较失败错误块；单表比较失败 console（不中断）；执行失败 toast「结构变更执行失败: err」 |
| 数据展示规则 | 徽标：新增（绿）/删除（红）/有差异（黄）；行左侧色条同色；详情列差异行按状态着色；新增表在 SQL 中仅注释「需要手动创建」 |

### PAGE009 备份还原

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE009〔F048-F049〕 |
| 页面目标 | SQL 备份导出与导入还原 |
| 进入条件 | 点「💾 备份还原」 |
| 页面结构 | 头部（📤 导出备份 / 📥 导入还原 Tab + ×）→ 导出面板（库 select → 表多选网格（全选链接）→ 导出类型 radio → 格式静态说明 → 开始导出 + 进度条 + 状态 + 结果块）／导入面板（mode-note → 目标库 select → 文件选择（readonly + 浏览 SQL）→ 先删复选 → 开始导入 + 进度 + 结果块） |
| 组件列表 | mode-tab×2、select×2、tables-grid（checkbox 网格）、radio、file-selector、checkbox-item、btn-export/btn-import、progress-bar/status、result-message |
| 按钮列表 | Tab×2；全选/取消全选；📤 开始导出（导出中...，未选表禁用）；浏览 SQL；📥 开始导入（导入中...，无文件/未选库禁用）；× |
| 按钮行为 | 选库 → meta_list_tables（不含视图）默认全选；导出 → save 对话框（{db}_backup_{ts}.sql）→ 取消 notifyInfo「已取消导出」/ 确认 db_export → 进度 100% + 绿色结果块（保存位置/表数量）+ toast；导入 → db_import（drop_existing 可选）→ 结果块（导入表数/行数）+ toast |
| 状态列表 | 导出/导入 Tab；未选库（仅库选择）；表已加载；导出中（进度条+「正在导出...」）；成功/失败结果块；导入中 |
| 跳转关系 | × → 返回 SQL 查询视图 |
| 异常处理 | 未选库 notifyError「请选择数据库」；未选表「请选择至少一个表」；未选文件「请选择导入文件」；导出/导入失败红色结果块 + toast「导出失败/导入失败: err」 |
| 数据展示规则 | 格式说明固定「仅支持 SQL (.sql)」；导出类型仅「结构 + 数据（SQL）」；导入面板顶部固定提示「当前导入仅支持 SQL 备份文件。」 |

### PAGE010 通知中心

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE010〔F051-F052〕 |
| 页面目标 | 全局 Toast 与危险确认 |
| 进入条件 | 全局挂载；业务事件触发 |
| 页面结构 | 右上 Toast 栈（固定 z-2000）+ 确认遮罩（z-2100）与对话框（标题/正文/取消/确认） |
| 组件列表 | toast（success/error/info + ×）、confirm-dialog（btn-cancel/btn-confirm tone 类） |
| 按钮列表 | 每 Toast ×；确认 取消/确认（danger 红 / info 蓝） |
| 按钮行为 | × → dismissToast；取消/遮罩/Esc → resolveConfirm(false)；确认 → resolveConfirm(true)；Toast 超时自动消失 |
| 状态列表 | Toast 栈（可叠加）；确认对话框显/隐 |
| 跳转关系 | 无（阻断层） |
| 异常处理 | 并发 confirm：旧请求自动 resolve(false) |
| 数据展示规则 | success 3.2s / error 4.5s / info 3.2s 默认超时；确认文案可自定义 title/message/confirmLabel/cancelLabel/tone |

### PAGE011 设置/帮助/关于（侧滑面板）

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE011〔F050〕 |
| 页面目标 | 轻量信息面板 |
| 进入条件 | 顶栏三按钮 |
| 页面结构 | 右侧 420px 滑出（eyebrow「QueryLab」+ 标题 + ×）→ 内容分节卡片 |
| 组件列表 | shell-section 卡片（h3 + ul/ol/p） |
| 按钮列表 | ×（关闭） |
| 按钮行为 | 关闭面板（×/遮罩/Esc） |
| 状态列表 | settings / help / about 三态 |
| 跳转关系 | 无 |
| 异常处理 | 无 |
| 数据展示规则 | 设置：安全与存储 3 条 + 当前限制 2 条；帮助：快速开始 4 步 + 能力边界 3 条 + 快捷键 5 条；关于：版本/平台/构建 + 定位 + 发布阶段 |

### PAGE012 批量执行进度面板（未接线，评审演示态）

| 维度 | 规格 |
|------|------|
| PAGE-ID | PAGE012〔部分实现〕 |
| 页面目标 | 批量语句逐条进度展示 |
| 进入条件 | 【未知】旧项目无入口（组件未接线）；重制原型中由评审面板主动唤起演示 |
| 页面结构 | 全屏遮罩 → 面板（头部 ⚡ 批量执行进度 + × → 进度条 + 计数 → 统计（总计/成功/失败）→ 语句列表（序号/类型图标/预览/状态）→ 错误详情块） |
| 组件列表 | progress-track/fill、stat-item×3、statement-item（current/success/failed/pending 态）、error 块 |
| 按钮列表 | ×（关闭） |
| 按钮行为 | 关闭面板 |
| 状态列表 | 执行中（current 高亮蓝）/成功（绿）/失败（红）/待执行（半透明）；「N / M 语句执行完成」+「有错误」徽标 |
| 跳转关系 | 无 |
| 异常处理 | 错误详情 pre 展示 |
| 数据展示规则 | 语句类型图标：SELECT🔍 INSERT➕ UPDATE✏️ DELETE🗑️ CREATE🆕 ALTER🔧 DROP💣 事务🔒 其他📄；预览 60 字符截断 |

---

## 三、六项特检矩阵（逐页）

> 检查项：①空状态 ②加载 ③错误（连接失败 / SQL 报错）④权限 ⑤网络或连接异常 ⑥用户取消。
> 「—」= 该页无此场景（桌面本地应用，无账号权限体系；"权限"统一映射为数据库权限/编辑门禁错误）。

| 页面 | ①空状态 | ②加载 | ③错误（连接失败/SQL 报错） | ④权限（库权限/编辑门禁） | ⑤连接异常 | ⑥用户取消 |
|------|---------|-------|---------------------------|--------------------------|-----------|-----------|
| PAGE001 壳 | 未连接时视图仍可切，SQL 区显示连接信息「未连接」 | 启动加载 app_info/conn_list（失败仅 console） | 未连接执行 SQL → 「请先选择连接」+ No connection | — | 状态栏 No connection | — |
| PAGE002 连接 | 「暂无连接，点击 + 新建」 | 测试中（...）/保存中（保存中...） | 测试失败红块（连接失败/超时 5 秒）；保存/删除失败提示 | 无库权限时 conn_test 报错透传 | 测试连接超时 5s 明确文案 | 弹窗取消/Esc；删除确认取消 |
| PAGE003 Schema | 「请先选择连接」/「无可用数据库」/「无表」 | 「加载中...」（库与表两级） | 加载库错误文案；DDL 失败 toast | 无权限库不可见/表加载失败 console | 连接断开时 DDL 报错 toast | 各弹窗取消；右键菜单点遮罩关闭 |
| PAGE004 编辑器 | 编辑器占位符；历史「暂无历史记录」 | （编辑器本地无加载态；执行态见 PAGE005） | 未连接 notifyError；SQL 错误在结果面板 | 无权限 SQL 由结果面板报错透传 | 连接断开执行报错 | 片段/历史弹窗关闭 |
| PAGE005 结果 | 「执行 SQL 后结果显示在这里」/空结果集（0 行 Tab） | 「执行中...」+ spinner | 错误态 ⚠️+错误 pre（SQL 语法/连接失败: err） | 复杂 SQL 只读（门禁不显示编辑 UI）；无权限报错透传 | 「连接失败: err」错误态 | 单元格编辑 Esc/✗ 取消；导出 save 取消 |
| PAGE006 网格 | 「无数据」（无列）/空表「📭 此表当前没有数据」 | 「加载中...」 | 加载错误块；写失败红色横幅 | 无单列主键 → 只读横幅+按钮禁用+双击提示 | 连接失败错误块 | 删除确认取消；导出取消 |
| PAGE007 设计器 | 新建模式空列（需 ≥1 列才能保存） | 编辑模式「加载中...」 | 校验错误横幅；「保存失败: err」 | 无权限 ALTER 报错透传 | 同左 | 关闭按钮放弃更改 |
| PAGE008 对比 | 「没有发现结构差异，两个数据库结构相同」 | 「比较中...」 | syncError 错误块；执行失败 toast | 无权限表比较失败（console 跳过）；执行报错 toast | 比较报错错误块 | 执行同步 danger 确认取消 |
| PAGE009 备份 | 未选库仅显示库选择 | 「正在导出/导入...」+进度条 | 红色结果块 + toast（导出失败/导入失败: err） | 无权限导出/导入报错 | 同左 | save 对话框取消 →「已取消导出」 |
| PAGE010 通知 | 无 Toast 时不可见 | — | — | — | — | Toast ×；确认取消/Esc/遮罩 |
| PAGE011 面板 | 固定内容 | — | — | — | — | ×/遮罩/Esc 关闭 |
| PAGE012 进度（演示） | 待执行语句半透明 | 执行中高亮 | 「有错误」徽标 + 错误详情块 | — | — | × 关闭 |

---

## 四、原型映射约定（供 HTML 原型与验收）

- 旧版原型（P1-P3 产物，现位于 `prototype/v0-old/app-prototype.html`）中每页顶部标注「PAGE 编号 + PRD 章节」，与本文一一对应；V1 新版原型见 `prototype/v1-new/app-prototype.html`（P6 产物，含 P4 B 类优化落地）。
- 原型「场景库」面板提供五态切换：加载 / 成功 / 失败（连接失败、SQL 报错两类）/ 空数据 / 异常（只读门禁、敏感历史、超时等），对应上表矩阵。
- 所有 toast/确认在原型中按第 一.3/一.4 条约定渲染。
