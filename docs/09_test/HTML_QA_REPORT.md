# QueryLab HTML 原型质量验收报告（V1 新版 · Level 3）

> 验收标准：《AI 生成 HTML 原型质量验收标准 v1.0》
> 验收日期：2026-09-03
> 验收角色：产品测试负责人（**纯验收：只记录，不修改任何 HTML/产品文件**；本报告为唯一产出物）
> 主验对象：`prototype/v1-new/app-prototype.html`（V1 新版原型，P6，12 页，Level 3 口径）
> 快检对象：`prototype/v0-old/app-prototype.html`（旧版原型，对照 `docs/09_test/HTML_V0_ACCEPTANCE.md` 口径）
> 对比基线：`docs/02_product/PRD.md`（v2.0，PAGE001-PAGE012 / F001-F057）、`docs/02_product/PAGE_SPEC.md`（11 维度规格 + 六项特检矩阵）、`docs/03_flow/USER_FLOW.md`（F1-F8 旅程）、`docs/02_product/FEATURE_MAP.md`、`docs/07_design_system/TOKEN.md` + `COMPONENT.md`、`docs/01_reverse/REVERSE_ANALYSIS.md` §⑦ 数据模型。

---

## 一、结论摘要

### 1.1 最终定级：**Level 3（达标，可进入开发）**

| 开发准入 8 项 | 结果 | 证据（详见对应章节） |
|---|---|---|
| 1. 页面覆盖 12/12 | ☑ 通过 | §2.1：PAGE001-PAGE012 全部可进入/返回，逐页实测 |
| 2. 功能覆盖（P0 全呈现） | ☑ 通过 | §2.2：57/57 按原型口径覆盖（C 类留档 14 项按 PRD 不呈现，与覆盖说明一致） |
| 3. 所有按钮/交互有效 | ☑ 通过 | §2.3 + §4：静态 96 个 button 零无行为；动态抽查 50+ 控件全部响应 |
| 4. 五态覆盖 | ☑ 通过 | §2.4 + §5：加载/成功/失败(连接/SQL)/空/门禁 六场景全部实测触发 |
| 5. 异常路径覆盖（六项特检） | ☑ 通过 | §2.5：空/加载/错误/权限门禁/连接异常/用户取消逐页抽测 |
| 6. console 无 JS 错误 | ☑ 通过 | §7：全程 pageError=0、consoleError=0（仅 favicon 404 一次，资源类，与功能无关） |
| 7. DS 一致性（Token/组件） | ☑ 通过 | §6.2：核心 Token 抽查 23/23 与 TOKEN.md 一致；组件规格与 COMPONENT.md 对齐 |
| 8. 数据结构与真实模型一致（偏差已记录） | ☑ 通过 | §6.1：关键字段名/语义与 REVERSE §⑦ 一致；简化项如实列入 P3 |

**阻断项（P0/P1）：0 项。** 发现 P2 缺陷 1 项、P3 备注 4 项（§3），均不阻断开发准入。

### 1.2 核心数字

| 指标 | 数值 |
|---|---|
| 动态断言总数（独立浏览器实测） | **124 条**（v1 主验 112 + v0 快检 12） |
| JS 语法检查（node --check） | v1、v0 均 0 错误 |
| 无行为按钮 | **0 个**（96 个 button 全部有处理器：静态 92 个内联 + 4 个运行时绑定） |
| 孤儿事件处理器（定义未用/调用未定义） | 0 个（169 个顶层函数全部有引用；内联事件引用全部可解析） |
| 死链接/外部资源 | 0 个（单文件零外链） |
| console JS 运行时错误 | **0 个**；favicon.ico 404 ×1（http.server 环境产物，与功能无关） |
| 原型实现缺陷 | P2 ×1、P3 ×4（无 P0/P1） |

---

## 二、五项覆盖率矩阵

### 2.1 页面覆盖（12/12 = 100%）

| 页面 | 实测路径与结果 |
|---|---|
| PAGE001 应用壳 | PASS——顶栏三按钮（设置/帮助/关于）+ 版本号 `v0.1.0 (dev)` + 状态栏「Ready → Connected to 本地开发库 → Query completed in Xms → Batch completed: 5 statements」全流转实测；右侧平台标注「平台: macos」 |
| PAGE002 连接管理 | PASS——2 条内置连接；点击选中高亮 + 状态栏 Connected；＋新建/✎编辑/⚡测试/✕删除全链路（§4） |
| PAGE003 Schema 树 | PASS——展开 shop：5 表 + 1 视图（视图样式类区分）+「+ 新建表」入口；系统库（information_schema/mysql/performance_schema/sys）过滤生效 |
| PAGE004 SQL 查询 | PASS——工具栏 7 按钮 + 连接信息条（B5 方言徽标 MYSQL）+ 快捷键提示；Ctrl+Enter/S/H/K、F1 全部实测生效 |
| PAGE005 结果面板 | PASS——执行 SQL → 结果 Tab「结果 1 (20 行)」+ 信息条（耗时 Xms / 查询 ID 前 8 位 / 编辑提示）+ 主键列标记 + CSV/JSON 导出按钮 |
| PAGE006 数据网格 | PASS——users 表 120 行、每页 50 行、「第 1 / 3 页，共 120 行」；翻页 1→2→3→2、页码跳转、筛选/清除、行勾选/全选 |
| PAGE007 表设计器 | PASS——新建模式（📝 新建表 - shop. + 指示条）/编辑模式（users 5 列 + 索引只读区）；关闭返回路径正确 |
| PAGE008 结构对比 | PASS——进入后模式 select 锁定「仅结构（当前支持）」+ 模式说明「不做真实数据同步」承诺边界文案在位 |
| PAGE009 备份还原 | PASS——导出/导入双 Tab 切换 + × 返回查询视图 |
| PAGE010 通知中心 | PASS——demoToasts 三级（success 3.2s / error 4.5s / info 3.2s）类名逐一断言 + 手动 × 关闭；danger 确认链（删除连接/删表/清空/删行/执行同步）全测 |
| PAGE011 侧滑面板 | PASS——设置/帮助/关于三态内容分节（钥匙串安全说明、Ctrl+Enter 快捷键表、版本/平台/构建）+ Esc/× 关闭 |
| PAGE012 批量进度 | PASS——B3 已接线：批量运行自动唤起面板（5 条语句 420ms/条推进至「5 / 5 语句执行完成」）+ 评审面板 demoBatch 入口 + 状态栏 Batch completed |

### 2.2 功能覆盖（57/57 编号功能 = 100%，原型口径）

原型评审面板自述「57/57（100%），C 类留档 14 项不呈现」——与 PRD §5 功能列表 + FEATURE_MAP 统计口径一致（留档项为 PRD 明确不承诺范围）。实测抽样（全部通过）：

- **连接 F001-F005**：表单 5 字段保存（列表 2→3 条）；编辑回填除密码（密码恒空 + placeholder「留空保持原密码不变」= B1）；测试成功绿块（连接成功!/延迟: 12ms/版本: 8.0.36/用户: root）与失败红块（连接失败: 连接超时（5秒）+ B13「编辑连接」直达按钮）；删除当前连接 → No connection + Schema 树「请先选择连接」+ 编辑器清空；密码钥匙串口径（连接对象不含 password 字段，F005）。
- **Schema F006-F013**：系统库过滤；表/视图区分（isview 类 + 视图名黄色）；新建库弹窗字符集 6 项联动排序规则（gbk → gbk_chinese_ci/gbk_bin）；空库名 notifyError「请输入数据库名称」；创建成功自动展开；右键菜单 4 项（刷新/重命名表/清空表数据/删除表）；重命名双层确认（info tone）→「表重命名成功（RENAME TABLE · F011）」+ 树刷新；清空 danger 确认「此操作不可撤销」→「表数据已清空（TRUNCATE · F012）」；删除 danger 确认「⚠️ 此操作不可撤销！表结构和数据将被永久删除！」；视图重命名/清空拦截「视图不支持重命名/清空」。
- **SQL F014-F022**：行号/语法高亮层/编辑器占位符；多语句 → 2 结果 Tab；未连接执行 → toast「请先选择连接」+ 结果面板错误态 + 状态栏 No connection；格式化 Ctrl+S（select→SELECT + 换行缩进）；F1 片段 24 项 + 点选插入；历史入栈/回填/计数徽标；Ctrl+K 清空；Ctrl+F 提示（F021 说明性覆盖）；补全：`sel` 前缀弹候选、FROM 后表名候选 9 项（B8 接 Schema 数据）。
- **结果 F023-F027/F056**：多 Tab 行数标注与实际行数一致；主键列标记；五态；信息条（耗时/查询 ID/编辑提示）；单表直查（\`shop\`.\`users\` LIMIT 20）→ 双击出现编辑器（✓/✗/NULL 三钮 + Enter/Esc）；复杂 JOIN SQL → 只读且信息条给出原因「复杂 SQL 结果只读（门禁：单表直查 + 单列主键）（B11）」；导出 CSV → 保存对话框模拟 + B4 文件名兜底 `users_20260903_033743.csv`。
- **网格 F028-F037**：120 行/50 行分页；翻页/页码输入；筛选 user_1 → 「第 1 / 1 页，共 32 行」计数同步 + 清除恢复 120；双击非主键列编辑 → Enter 保存 →「更新成功」横幅（主键列双击不响应 = 门禁正确）；NULL 开关（输入框 isnull 半透明态）；勾选 1 行 → 删除确认「确定要删除选中的 1 行数据吗？此操作不可撤销！」→ 120→119 + 「成功删除 1 行（逐行 DELETE … LIMIT 1 · F033）」；取消 → 不动；新增行（new-row 样式 + 逐列输入 + 末列 Enter 提交「插入成功（自增主键列已排除 · F032）」）；空表 archive_empty「此表当前没有数据」+ 列头保留；无主键表 log_no_pk 只读横幅「网格视图仅支持浏览、筛选、导出和插入」+ 删除禁用 + 新增仍可用（F037 口径）；表头全选 → selCount=50。
- **设计器 F038-F042**：预置 id 列；添加列/删除列；长度输入按类型显隐（INT→display:none，VARCHAR→显示）；主键勾选时 NULL 复选框禁用；校验文案逐条实测：「请输入表名」「列名不能为空」「列名 "1bad-name" 不符合命名规则」「列 "c1" 设置了自增必须是主键」「目前只支持单列主键」；新建保存 →「表 'shop.qa_table' 创建成功（meta_create_table · F038）」；编辑改名 →「有未保存的更改」指示 → 保存 → diff→ALTER 预览弹窗（`ALTER TABLE \`shop\`.\`users\` DROP COLUMN \`user_name\`; …`）→ 确认执行 →「表结构保存成功（diff → ALTER 逐条执行 · F039）」；无变更保存 →「没有需要保存的结构变更」。
- **对比 F043-F047**：源=目标 →「源数据库和目标数据库不能相同」；shop→analytics → 统计 6 项 + 差异 4 行 + 详情面板 SQL 预览正确插值（无 `{targetDatabase}` 字面量 = B6）；执行同步 danger 确认「此操作会直接修改目标数据库结构，且不可撤销。建议先完成 SQL 备份。」→ 取消不执行。
- **备份 F048-F049**：选库 → 表网格默认全选 5/5；导出 → 保存确认（shop_backup_20260903_033030.sql）→ 成功块（保存位置 ~/Downloads/... + 表数量 5）；导入未选库 →「请选择数据库」；选库+选文件 → 进度条 →「导入完成 表数量: 3 行数: 约 1280」；先删表复选在位。
- **全局 F050-F055**：三侧滑面板；Toast 三级 + 手动关闭；状态栏全生命周期；视图切换器 5 按钮（数据网格/设计表条件显示）；切回查询视图自动回填 `SELECT * FROM \`shop\`.\`users\` LIMIT 1000;`（F055）+ 回填提示条；B2 分句预览条「将执行 2 条语句 · 首条：SELECT ';' AS a;」（含分号字符串正确计 1 条）。

### 2.3 操作覆盖（全部按钮有效）

静态：96 个 `<button>`，其中 92 个内联 onclick；4 个无内联属性者经源码核查全部为运行时绑定（`confirmCancel/confirmOk` JS 挂载 onclick；toast `.tclose` 动态创建后绑定；分页「下一页」为动态拼接 HTML 内含 onclick）。**无行为按钮 = 0**。孤儿处理器 = 0（反向：169 个顶层函数全部被 HTML 或 JS 引用；正向：内联事件引用的 88 个调用名全部有定义）。

### 2.4 状态覆盖（五态）

| 状态 | 触发方式 | 实测结果 |
|---|---|---|
| 加载 | 场景库「加载中」/SQL 执行 700ms 窗口 | PASS——「执行中...」+ spinner，场景锁定（切场景库继续）；网格/树「加载中...」 |
| 成功 | 默认全链路（查询/更新/插入/删除/建库/建表/导入/导出/重命名） | PASS——9 类成功 toast/横幅/结果块逐一断言 |
| 失败·连接失败 | 场景库 errorConn / cfHost=192.168.1.100 测试 | PASS——结果面板「连接失败: Can't connect to MySQL server on 'localhost' (61)」；测试块「连接超时（5秒）」；Schema 错误文案 |
| 失败·SQL 报错 | 场景库 errorSql | PASS——⚠ + errno 1064 完整错误详情（SQL 语法错误原文） |
| 空数据 | 场景库 empty / archive_empty 表 | PASS——空表「此表当前没有数据（shop.archive_empty）」+ 列头保留；分页显示「空表」 |
| 异常·门禁（权限） | 场景库 gate / log_no_pk 表 | PASS——只读横幅 + 删除禁用 + 复选禁用 + 双击无编辑器；结果面板复杂 SQL 只读 + 原因提示（B11） |

### 2.5 异常覆盖（六项特检矩阵）

按 PAGE_SPEC §三逐项抽测：空状态（连接/历史/树/网格/结果 5 处空态文案全命中源码原文）；加载（库/表/结果/导出/导入 5 处）；错误（连接失败/SQL 报错/保存失败文案/校验横幅）；权限→编辑门禁（单列主键双门禁 + 主键列本身不可编辑）；连接异常（未连接执行/测试超时 5s 文案）；用户取消（弹窗 Esc/遮罩/取消按钮 ×7 处、删除确认取消 ×3、执行同步取消、导出保存取消路径「已取消导出」于 v0 同口径实现）。**全覆盖。**

---

## 三、缺失与缺陷列表（P0-P3）

| # | 级别 | 描述 | 定性 | 证据 |
|---|------|------|------|------|
| D1 | **P2** | F040 列编辑联动的 UI 禁用状态不实时刷新：取消「主键」勾选后，同行的「自增」复选框仍可点击、「NULL」复选框状态滞留（`colChange` 仅在 `type` 变化时重渲染 `renderCols`，pk/nullable 变化只改数据不重绘）。**数据层与校验层有兜底**：勾自增会自动回置主键，保存前 `designerValidate` 仍拦截非法组合，不会产出错误 DDL | 原型实现层交互缺陷（非产品逻辑缺陷，不阻断开发，建议开发期修正联动重渲染） | part6 实测：`initNullableDisabled=true, aiDisabledAfterUnpk=false`；F041 校验文案两条均正确拦截 |
| D2 | P3 | SQL 模拟执行器不解析 LIMIT 数值：`SELECT ... LIMIT 10` 返回 20 行（固定 `rows.slice(0,20)`）。Tab 行数标注与实际行数**一致**（标注 (20 行) 显示 20 行），仅与 SQL 语义不符；且 F025 编辑门禁正则 `^select \* from ...(?: limit \d+)?$` 正确识别 LIMIT n | 演示级模拟失真（原型口径允许模拟数据，如实记录） | part1b P1-05：rows=20 标注一致；runSql L795-800 |
| D3 | P3 | 数据模型演示级简化：连接对象未含 `defaultDb`；TableInfo 未含 `engine/rowsEst`；结果集未模拟 `chunks/paging/warningCount`（分页语义由 PAGE006 pageSize=50 独立体现） | 与 REVERSE §⑦ 对齐性偏差（关键字段全部在位，简化项不影响需求评审） | §6.1 对照表 |
| D4 | P3 | 9 个已定义未使用的 CSS Token（`--ql-primary-pressed`、`--ql-value-bytes`、`--ql-code-selection/variable/meta`、`--ql-code-icon-*` ×3、`--ql-space-1`） | 预留 Token（TOKEN.md §6 允许定义表为唯一来源；未使用不算违规，开发期按需清理） | 静态分析：定义 87 / 使用 78 / 死引用 0 |
| D5 | P3 | 初始加载自动执行演示 SQL（约 0.7s 后状态栏 Executing→Query completed），评审初始态非「Ready」静止 | 有意演示行为（非缺陷），记录以免误判 | 调试脚本：初始 2.5s 后 statusMsg='Query completed in 73ms' |

**口径核对（已知缺陷的模拟一致性，非缺陷）：**

- **表设计器关闭丢弃无确认（PL-03/UF-05）**：实测关闭带未保存更改的设计器 → **无确认框直接丢弃**——与 USER_FLOW 旅程四「直接丢弃无确认 PL-03/UF-05」口径**一致**（B1-B13 修复清单不含此项，C 类留档 14 项亦不含 → 原型如实保留旧项目行为，正确）。PAGE_SPEC PAGE007「关闭 → onClose（App 复位新建态并回网格）」行为一致。
- 其余 v0 报告非阻断备注（系统对话框模拟、剪贴板 file:// 受限、PAGE012 演示态、F020 受限说明覆盖）在 v1 同口径延续。

---

## 四、按钮/控件抽查（实测 50+，此处列代表性 24 项）

| # | 控件 | 页面 | 实测行为 |
|---|------|------|----------|
| 1 | ＋ 新建连接 | PAGE002 | 重置表单开弹窗，端口默认 3306 |
| 2 | 表单「测试」 | PAGE002 | 成功绿块（延迟/版本/用户）/ 失败红块 + 编辑直达 |
| 3 | 表单「保存」 | PAGE002 | 列表 +1、弹窗关、toast「连接已创建（密码已存入系统钥匙串 · F005）」 |
| 4 | 连接项 ✕ 删除 | PAGE002 | danger 确认 → 删除 + toast；删当前连接 → 工作区清空 |
| 5 | 顶栏 设置/帮助/关于 | PAGE001/011 | 侧滑面板三分节内容 + Esc 关闭 |
| 6 | 视图切换 ×5 | PAGE001 | 5 视图切换 + 条件显隐 + F055 自动回填 |
| 7 | ▶ 运行 / 批量运行 | PAGE004 | 单条/批量分流；批量变紫标「批量运行」 |
| 8 | ⚡ 批量模式 + 🔒 事务 | PAGE004 | 事务按钮随批量显隐；事务包裹后 5 条语句进 B3 面板 |
| 9 | ⟡ 格式化（Ctrl+S） | PAGE004 | 关键字大写 + 子句换行 |
| 10 | 📋 片段（F1） | PAGE004 | 24 项双列 + 点选插入光标处 |
| 11 | 历史 开/关 + 清空 | PAGE004 | 侧栏开合、条目回填、计数徽标 |
| 12 | 结果 Tab / CSV / JSON | PAGE005 | 多 Tab 切换；导出走保存确认模拟（B4 文件名兜底） |
| 13 | 单元格 ✓ / ✗ / NULL | PAGE005/006 | 保存/取消/NULL 半透明态三钮均有效 |
| 14 | 网格 刷新/新增/删除(N) | PAGE006 | 刷新重载；新增逐列录入；删除确认→119 行 |
| 15 | 筛选 + 清除 | PAGE006 | user_1 → 32 行同步；清除按钮有词才显 |
| 16 | 分页 上一页/下一页/页码输入 | PAGE006 | 1→2→3→2 跳转合法 |
| 17 | 表头全选 | PAGE006 | selCount=50 |
| 18 | 右键菜单 4 项 | PAGE003 | 刷新/重命名/清空/删除 + 视图拦截 |
| 19 | + 数据库（字符集联动） | PAGE003 | gbk → 排序规则联动；空名报错；创建自动展开 |
| 20 | 设计器 添加列/删除列/保存/关闭 | PAGE007 | 联动/校验/ALTER 预览/关闭（无丢弃确认=PL-03 口径） |
| 21 | 对比 开始比较/差异行/复制/导出/执行 | PAGE008 | 统计 6 项、详情、danger 确认取消不执行 |
| 22 | 备份 Tab/全选/开始导出/浏览/开始导入 | PAGE009 | 默认全选、导出结果块、文件回填、导入结果块 |
| 23 | Toast × / 确认 取消/确认/遮罩/Esc | PAGE010 | 三级配色 + 全部关闭路径 |
| 24 | 场景库 6 按钮 + 演示入口 3 | 评审面板 | 五态切换 + demoBatch/demoSensitive/demoToasts |

---

## 五、状态异常触发记录（与 PAGE_SPEC §三矩阵对照）

| 触发场景 | 预期（page-spec） | 实测 | 判定 |
|---|---|---|---|
| 未连接执行 SQL | toast「请先选择连接」+ 状态栏 No connection | toast + **结果面板错误态同步显示**（超出预期的完整） | PASS |
| 测试连接失败/超时 | 红块「连接失败: err」/「连接超时（5秒）」 | 文案逐字一致 + B13 编辑直达 | PASS |
| SQL 语法错误 | ⚠ + 错误 pre 全文 | errno 1064 完整错误详情 | PASS |
| 连接失败场景 | 「连接失败: Can't connect…」 | 文案一致（错误码 61） | PASS |
| 无单列主键表 | 只读横幅 + 编辑/删除禁用 + 插入仍可用 | 三者全部正确（新增禁用=false） | PASS |
| 复杂 SQL 结果 | 门禁不显示编辑 UI | 双击无编辑器 + **只读原因提示（B11 增强）** | PASS |
| 设计器非法结构 | 校验横幅阻止提交 | 5 条校验文案逐字命中源码原文 | PASS |
| 源=目标库比较 | 「源数据库和目标数据库不能相同」 | 文案一致 | PASS |
| 备份未选库/未选文件 | notifyError 提前返回 | 「请选择数据库」实测 | PASS |
| 空表/空历史/无库 | 📭 占位 / 暂无历史记录 / 无可用数据库 | 文案命中源码原文 | PASS |
| 全部弹窗取消路径 | 遮罩/Esc/取消 = 取消 | Esc 关闭栈（confirm→modal×7→ctx→shell）实测 | PASS |
| 加载锁定 | 场景锁定持续加载态 | 锁定生效，切场景恢复 | PASS |

---

## 六、Level 3 附加检查

### 6.1 数据结构 vs 真实模型（REVERSE_ANALYSIS §⑦）

| 真实模型 | 原型模拟结构 | 对照结论 |
|---|---|---|
| ConnectionInfo（§7.1）：id/name/**driver**/host/port/user/password(skip)/defaultDb | `{id:'c1', name:'本地开发库', driver:'mysql', host:'localhost', port:3306, user:'root'}` | **一致**——serde 字段名逐字对齐（driver 而非 driver_type）；password 不出现在对象（F005 钥匙串口径，表单密码仅内存传递且保存后不回显）；defaultDb 未模拟（P3/D3） |
| ConnectionTestResult：latency_ms/server_version/user/default_db | 绿块「连接成功!/延迟: 12ms/版本: 8.0.36/用户: root」 | **一致**——三要素全呈现 |
| TableInfo（§7.2）：name/type(BASE TABLE\|VIEW)/comment/engine/rowsEst | `{name:'users', type:'BASE TABLE', comment:'用户表'}`（+VIEW） | **一致**——name/type/comment 对齐；engine/rowsEst 未模拟（P3） |
| Column：name/type(COLUMN_TYPE)/nullable/… | `{name:'user_name', type:'VARCHAR(50)', pk:false}` | **一致**——type 采用 COLUMN_TYPE 形态（含长度） |
| QueryResult/QueryResultSet（§7.3）：queryId/sets[]/elapsedMs；set{columns[],meta{affectedRows,…},chunks} | `state.result={sets:[{cols,rows,affected}]}` + `lastQueryId`/`lastElapsed`/`activeSet` | **语义一致**——queryId（前 8 位展示）、elapsedMs（耗时展示）、多 sets、columns、affectedRows、行数据全在位；chunks/paging/warningCount 未模拟（分页由网格 pageSize=50 体现）（P3/D3） |
| SQL 历史（§7.4）：localStorage key、去重置顶、上限 100、敏感词正则 | `pushHistory`：去重 unshift、`length>100→=100`、`SENSITIVE_RE=/password\|secret\|token\|api[_-]?key\|access[_-]?key\|private[_-]?key\|credential/i`、key 名 `querylab_sql_history` 出现于提示 | **逐字一致**——敏感词正则 8 词与逆向报告完全相同 |
| 片段（§7.5）：24 个 {name,sql} | SNIPPETS 24 项（实测计数=24，类别覆盖逆向清单） | **一致** |
| 备份参数（§7.6）：ExportParams/ImportParams（drop_existing） | 导出文件名 `{db}_backup_{ts}.sql`、导入 drop_existing 复选 | **一致** |
| 系统库过滤（page-spec 一.12） | `SYSTEM_DBS=['information_schema','mysql','performance_schema','sys']` | **一致** |

### 6.2 设计系统一致性

- **Token 抽查 23/23 一致**：`--ql-bg-app #1e1e1e`、`--ql-bg-panel #252526`、`--ql-bg-raised #2d2d2d`、`--ql-bg-hover #3e3e3e`、`--ql-primary #007acc`、`--ql-text-primary #d4d4d4`、`--ql-text-secondary #888`、`--ql-danger #f48771`、`--ql-success #4ec9b0`、`--ql-warning #dcdcaa`、`--ql-pk-text #c586c0`、`--ql-pk-bg #252835`、`--ql-code-keyword #c678dd`、`--ql-code-string #98c379`、mono 字体栈、`--ql-font-size-sm 12px`、`--ql-radius-sm 4px`、`--ql-sidebar-width 280px`、`--ql-topbar-height 48px`、`--ql-statusbar-height 24px`、三视图主色（design #9b46c8 / sync #0e639c / backup #c84e4e）——全部与 TOKEN.md v1.0 相同。
- **Token 使用规则**：:root 定义 87 个 `--ql-*`，正文引用 78 个，**死引用 0**（使用未定义 = 0）；颜色/字号/间距/圆角全面走 CSS 变量（最高频 `--ql-space-2` ×56），符合 TOKEN.md §6.1「禁止裸值」要求（未使用 Token = D4 预留）。
- **组件统一性（COMPONENT.md 对照）**：C12 可编辑单元格统一——结果面板与网格共用同一 `renderCellEditor`（✓/✗/NULL + Enter/Esc），门禁文案同源；C15/C16/C17 原子组件跨页统一类名（`empty-hint`/`five-loading`/`five-empty` + spinner/error 样式）；C18/C19 全局确认与 Toast 唯一挂载（z 序遮罩 rgba(0,0,0,.5)、danger/info tone）；C30 视图切换器 active 按视图主色；C31 状态栏 24px 蓝底；C32 全量 SVG 图标替代 emoji（B12，实测无 emoji 按钮图标）；C21 索引只读标注「索引（只读展示，编辑能力规划中 · C 类留档 PF-14）」如实呈现；PAGE007 长度输入按类型显隐（LENGTH_TYPES）符合 C21 规格。
- **架构一致性**：单文件零外链（0 个 http(s) 资源、0 个 img/link 外部引用）；文案基准与 PAGE_SPEC 一.11 源码原文逐字一致（「暂无连接，点击 + 新建」「无可用数据库」「无表」「暂无历史记录」「此表当前没有数据」「执行 SQL 后结果显示在这里」「请先选择连接」等 11 处抽验命中）；每视图 page-tag 标注 PAGE 编号 + PRD 章节，与 PAGE_SPEC §四映射约定一致。

### 6.3 USER_FLOW 六组旅程实测（全部走通）

| 旅程 | 关键节点实测 | 结论 |
|---|---|---|
| F1 连接建立 | 表单→测试成功/失败→保存→点击→Connected→Schema 加载；删除清空 | 走通；v1 额外呈现 B1（密码留空语义）/B13（失败编辑直达、删连接清编辑器）修复态 |
| F2 日常查询 | 写 SQL→执行（选中/多语句）→结果→导出按钮→历史回填→格式化/片段/清空 | 走通；自动回填 F055 带提示条（B13 对「覆盖手写 SQL」缺陷的缓解呈现） |
| F3 数据修正 | 点表→分页/筛选→双击编辑→Enter 保存→新增行→勾选删除确认/取消 | 走通；只读降级横幅三态正确 |
| F4 结构演进 | +新建表→加列/联动→校验→创建成功；编辑→diff→ALTER 预览→确认执行；无变更提示 | 走通；**关闭丢弃无确认 = PL-03 口径如实保留**（D1 的联动 UI 滞后为 P2） |
| F5 备份+对比 | 同库拦截→比较→统计/差异/详情→danger 取消；导出默认全选→结果块；导入选库+文件→结果块 | 走通 |
| F6 批量+事务 | 批量开关→事务开关→运行→**B3 进度面板逐条推进→Batch completed** | 走通（旧项目未接线缺陷已按 B3 修复呈现） |

---

## 七、代码质量

| 检查项 | 方法 | 结果 |
|---|---|---|
| JS 语法 | 提取内嵌脚本（1021-3031 行，96KB）`node --check` | **0 错误**（v0 同法 0 错误） |
| 无行为按钮 | 96 个 button 静态扫描 + 运行时绑定核查 | **0 个** |
| 孤儿处理器 | 内联事件引用 × 顶层定义双向交叉 | 0 个（169 函数全引用；88 个内联调用名全可解析） |
| 死链接 | href/src 全量正则 + 外部资源扫描 | 0 个（单文件零外链） |
| CSS 变量完整性 | :root 定义 × var() 使用交叉 | 87 定义 / 78 使用 / 0 死引用 / 9 预留 |
| console | 独立 headless Chromium 全程监听（console + pageerror） | **JS 运行时错误 0**；favicon.ico 404 ×1（服务端无该文件，资源类，与功能无关；file:// 直开不存在——与 v0 验收报告同口径） |
| XSS 基线 | 值渲染统一 `esc()` 转义（&/</>/"） | 抽查 renderConnList/renderHistory/renderCols 等动态拼接均经转义 |

---

## 八、v0-old 快检结论

对照 `docs/09_test/HTML_V0_ACCEPTANCE.md`（2026-09-02，结论 PASS）口径：

| 快检项 | 结果 |
|---|---|
| 可打开（http://localhost:8305/v0-old/app-prototype.html） | PASS——title「QueryLab 可交互原型（lys-query-lab 重制评审版）」 |
| console | PASS——pageError=0、consoleError=0（本轮快检会话内无 favicon 404） |
| 页面数 | PASS——PAGE001-PAGE012 共 12 个标注全部在文档中；评审面板页面导航 12 项 |
| 与 v0 验收报告口径一致性 | PASS——2 条内置连接（本地开发库/测试环境）；场景库五态 6 项（成功/加载中/失败·连接/失败·SQL/空数据/异常·门禁）；点击连接 →「Connected to 本地开发库」+ 状态栏平台；场景切换（加载中）生效 |

**v0-old 快检结论：可打开、12 页齐全、console 干净、与 HTML_V0_ACCEPTANCE.md 口径一致（维持 PASS 归档有效）。**

---

## 九、环境说明与干扰记录

1. **服务**：`python3 -m http.server 8305`（serve QueryLab/prototype 目录；v1-new 200/173091B、v0-old 200/174134B），验收完成后已关闭。
2. **环境干扰（重要）**：共享的 Playwright MCP 浏览器实例存在**其他项目会话并发占用**（观察到 TermForge 127.0.0.1:8765、Steering-BLE 127.0.0.1:9471、RedisPilot localhost:8742、Batch Image Studio 127.0.0.1:8301 等 tab，且本任务 tab 一度被导航至 Batch Image Studio 页面）。为保证结果可信，**主验收改用独立 headless Chromium 进程**（Playwright 1.49.1，借用 smart-ble 项目 node_modules 运行，脚本与产物均置于 /tmp，未触碰任何现有项目文件）执行全部 124 条断言——两轮验证（MCP 实测 + 独立进程）结论一致，结果不受干扰污染。
3. **验证方式降级说明**：无降级——静态 + 动态均完整执行；动态为独立浏览器真实点击/键盘/事件驱动（含 evaluate 函数级驱动用于绕开 hover 显隐与多层遮罩的自动化交互限制，均附实现源码核对）。
4. **测试数据副作用**：全部测试在浏览器内存态进行（刷新即重置）；未修改原型文件、未写任何项目文件（本报告除外）、未执行 git 操作。
5. 断言统计：v1 主验 112 条（part1b 42 + part2 30 + part3 15 + part4 10 + part5 8 + part6 7）+ v0 快检 12 条 = **124 条**；功能层断言全部最终通过（过程失败均为脚本选择器/时序问题，已逐一复核定性，见 §三证据列）。

---

## 十、最终结论

| 维度 | 结果 |
|---|---|
| 页面覆盖 | 12/12（100%） |
| 功能覆盖 | 57/57（100%，原型口径；C 类留档 14 项按 PRD 不呈现） |
| 操作覆盖 | 96 按钮全有效（实测 50+ 控件，0 无行为、0 孤儿处理器） |
| 状态覆盖 | 五态 6 场景全覆盖（加载/成功/失败×2/空/门禁） |
| 异常覆盖 | 六项特检矩阵逐页全覆盖 |
| USER_FLOW | F1-F8 六组旅程全部走通（已知缺陷 PL-03 按 USER_FLOW 口径如实保留） |
| 数据 vs 真实模型 | 关键字段/文案/正则逐字一致；简化项 P3 记录在案 |
| DS 一致性 | Token 抽查 23/23 一致；组件规格对齐；零死引用 |
| 代码质量 | node --check 0 错误；0 死链；console JS 错误 0 |
| 缺陷 | P0=0，P1=0，P2=1（D1 联动 UI 滞后），P3=4 |

# 最终结论：**Level 3 达标 —— 准予进入开发**（建议开发期顺手修复 D1，并在实现 LIMIT 语义、defaultDb/engine 等简化项时以真实行为为准）。

> 验收人：产品测试负责人（AI 代理执行）
> 报告产物：本文件为本次验收唯一新增文件；未修改任何 HTML/源码/文档，未 commit/push。

---

## 复验附录（开发角色修复，2026-09-03）

> 角色：开发（修复者）。本附录为文末追加，原报告正文（含 §三 D1 条目）未改动。
> 修复对象：`prototype/v1-new/app-prototype.html`（唯一改动文件，另含本附录所属报告文件）。

### A1. D1（P2）修复说明

- **位置**：`colChange()`（内嵌脚本，原型文件内 L2436-2445 区域）。
- **改法**：将原 `if(field==='type'){renderCols();}` 扩展为 `if(field==='type'||field==='pk'||field==='nullable'||field==='ai'){renderCols();}`——pk/nullable/ai 变化后调用**既有** `renderCols()` 重渲染，使「自增」`disabled`（规格 `!pk`）与「NULL」`disabled/checked`（规格 `pk` 禁用、数据 `nullable` 定勾选）即时与数据层一致。**完全复用现有渲染规格函数，未新增/修改任何联动规则**；文本字段（name/length/def/comment）仍不重绘以免打断输入，与原行为一致。
- **不变项（按修复要求）**：数据层规则原样保留（勾自增自动回置主键 `if(field==='ai'&&val){c.pk=true;c.nullable=false;}`、勾主键回置 nullable）；`designerValidate()` 未改动。改法为最小 diff（1 行条件 + 1 行注释）。
- **语法自查**：提取内嵌脚本 `node --check` → 0 错误。

### A2. 浏览器复验（对齐 QA 取证路径：连接 → 展开 shop → + 新建表 → PAGE007 设计器）

独立 headless Chromium（playwright-core 1.49.1 ↔ chromium-1148，`python3 -m http.server 8305` 服务 `prototype/v1-new/`），全部真实点击/选择驱动，脚本 `/tmp/ql_fix_verify.js`。**25 条断言全 PASS**，要点：

| 复验项（QA 报告 §三 D1 对应） | 结果 | 证据 |
|---|---|---|
| ① 初始：主键勾选 → 自增可勾、NULL 禁用 | PASS | `initNullableDisabled=true`（与 QA 取证同名变量，初态一致） |
| ② 取消主键 → 联动即时刷新（D1 核心） | PASS | **`aiDisabledAfterUnpk=true`（QA 缺陷实测值为 false，已修复）**；NULL 即时解禁、勾选态=数据；UI=数据无滞后（3 组布尔逐一比对） |
| ③ 再次勾选主键 → 恢复 | PASS | 自增恢复可用+勾选、NULL 即时禁用且按既有规则回置未勾 |
| ④ designerValidate 仍拦截（F041 两文案回归） | PASS | 「列 "id" 设置了自增必须是主键」「目前只支持单列主键」逐字命中；合法组合保存仍成功（F038 建表链路 `shop.qa_fix_t` 不误伤） |
| ⑤ type 变化联动回归 | PASS | INT→VARCHAR 长度显示 / VARCHAR→INT 长度隐藏；重渲染后 pk/自增/NULL 联动态保持 |
| console 零 JS 错误 | PASS | pageerror=0、console.error=0（无 favicon 404 出现于本会话） |

完整断言输出（25 PASS / 0 FAIL）：T1 初始态×4、T2 取消主键联动×6、T3 NULL 可勾×2、T4 恢复×5、T5 校验拦截×3、T6 type 联动×4、T7 console×1。

### A3. 范围与 git 自查

- 改动文件仅 2 个：`prototype/v1-new/app-prototype.html`（修复本体）、`docs/09_test/HTML_QA_REPORT.md`（本附录）；未触碰 tracked 源码、v0-old、docs 其他文件；未执行任何 git 写操作；未删除功能/简化流程。
- git 自查：修复前后 `git status --porcelain` 输出完全一致（仓库内均为未跟踪目录 `??`，M 列表前后均为空）。
- 服务 `http.server 8305` 复验完成后已关闭。
