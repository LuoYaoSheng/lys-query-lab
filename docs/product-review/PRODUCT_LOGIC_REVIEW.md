# QueryLab 产品逻辑评审总报告（P-Logic）

> 评审依据：《AI 产品重构逻辑评审规范 v1.0》· 2026-09-03
> 输入文档清单：`docs/01_reverse/REVERSE_ANALYSIS.md`（P1 逆向报告）、`docs/02_product/PRD.md`（P2 PRD v2.0）、`docs/02_product/PAGE_SPEC.md`（P3 页面规格）、`docs/06_review/PRODUCT_REVIEW.md`（P4 产品体验审查，A/B/C/D 编号交叉引用）、源码抽查（`src/`、`src-tauri/` 权威工作区，README.md 约定根目录为准）。
> 评审方式：只评审、不修改。所有「当前设计」描述附文件路径与行号；无法证实者标注【未知】。规范原文未随任务提供，§17 八项验收清单按任务给定评审重点（§2-§14）归纳执行，未虚构规范条文。
> 分级沿用项目既有口径：A=文档勘误（本评审仅建议，不直接改文档）、B=重构落地、C=需用户决策、D=观察不动。
> 实测限制：与 P4 相同，本机无可用 MySQL 实例，连接类行为以源码逻辑推断为准，涉及处已注明「代码路径核实，未实测」。

---

## 一、评审范围与核心结论

QueryLab 是本地优先的 MySQL/MariaDB 数据库工作台（Tauri 2 + Svelte 5），产品目标、边界口径（结构对比仅预览、备份仅 SQL、编辑门禁）与文档-代码一致性总体良好（P4 已确认的正面基线维持）。

但本次评审在数据与权限层发现 **两个 P4 未覆盖的链路级缺陷**，均直接影响产品核心承诺：

1. **数据库凭据链路断裂（PL-01）**：连接密码写入系统钥匙串后，执行链路（查询/浏览/备份）从不读钥匙串，而前端持有的连接对象不含密码——带密码账户「测试连接成功，进入工作区后一切操作认证失败」。P4 PF-01 的缓解结论「重输密码即可恢复」在代码层面不成立。
2. **原生文件对话框插件未注册（PL-02）**：前端 3 个组件调用 `@tauri-apps/plugin-dialog`，但 Rust 侧未引入、未注册该插件，也无 capabilities 声明——打包态导出必走浏览器降级下载、备份导入文件选择完全不可用。

其余问题集中在：状态模型的「乐观连接态」、设计器未保存关闭无拦截、编辑器危险语句确认不对称、`fs_write_file` 无路径限制等。

---

## 二、§17 八项验收清单逐项结论

| # | 验收项（按评审重点归纳） | 结论 | 关键证据 |
|---|--------------------------|------|----------|
| 1 | 产品目标明确性与现状一致性 | **有条件通过**。目标（本地优先 MySQL 工作台）与边界口径诚实（`docs/RELEASE_CHECKLIST.md`）；但核心承诺「连接→查询」对带密码账户在代码层不成立（PL-01），属目标级阻断 | driver.rs L64-65；connections.rs L52；query.rs L232-242 |
| 2 | 核心流程最短路径（连接→浏览→查询→保存结果） | **不通过**。连接→浏览→查询路径形态合理（点连接→展开库→点表/写 SQL）；但「保存结果」主链路（原生保存对话框）因插件未注册而损坏（PL-02），仅剩行为不可控的降级下载 | Cargo.toml（无 tauri-plugin-dialog）；main.rs L20；DataGrid.svelte L637-646 |
| 3 | 页面职责单一性 | **基本通过**。12 页中 10 页职责清晰；例外：「设置」面板名不副实（仅静态说明，无任何可操作项，IA-03）；编辑器单页承载 7+ 辅助功能（历史/片段/格式化/补全/批量/事务/清空，属工作台惯例，D） | App.svelte L462-479 |
| 4 | 流程完整性五要素（触发/前置/步骤/结果/异常出口） | **不通过**。主链路五要素齐全且异常覆盖好（P4 正面基线）；两处缺口：① SQL 执行无超时无取消，「执行中」态可能无终点（UF-02）；② 设计器「关闭」直接丢弃未保存更改，无确认出口（PL-03） | query.rs（无 timeout）；TableDesigner.svelte L505；App.svelte L389-393 |
| 5 | 信息架构可预测性 | **有条件通过**。7 例抽查 3 例不符预期：设计表入口需两跳且右键菜单无此项（IA-01）；sync/backup 视图关闭语义与其他视图不一致（IA-02）；导出入口位置随页面漂移（IA-04） | 见 INFORMATION_ARCHITECTURE_REVIEW.md |
| 6 | 数据存储合理性（临时/用户/配置/日志分类） | **不通过**。分类大体正确、结果集仅内存（正面）；但密码「存而不取」+ 读取时机错配（DS-01）、SQL 历史跨连接共享且敏感过滤不覆盖数据字面量（DS-03/05） | connections.rs L51-53；SqlEditor.svelte L33-41/L93-120 |
| 7 | 状态合理性（必要/过多/冲突/缺失） | **有条件通过**。结果五态完整（正面）；「Connected to X」为乐观断言且无断连回退（ST-01）；exportLoading 恒 false 为死状态（ST-03） | App.svelte L104；ResultsPanel.svelte L18 |
| 8 | 权限合理性（keychain 时机/系统能力/数据库凭据） | **不通过**。密码不落盘正面；但 `fs_write_file`/`db_import` 提供无作用域限制的任意路径写/读原语（PM-01/02）、keychain 密码被高频读入内存却从不用于执行（PM-04）、无 Tauri capabilities 声明（PM-07） | app.rs L29-36；backup.rs；tauri.conf.json |

---

## 三、问题汇总表（PL 编号；细则见分报告）

> 「与 P4 关系」：新=P4 未覆盖；交叉=P4 已有编号，此处不重复展开；升级=P4 分级或结论需修正。

| 编号 | 级别 | 问题 | 与 P4 关系 | 证据 |
|------|------|------|-----------|------|
| PL-01 | B | 数据库凭据链路断裂：钥匙串密码从未被执行链路消费，前端传参恒空密码，带密码连接测试成功后浏览/查询必失败（代码路径核实，未实测） | 新（修正 PF-01 缓解结论） | driver.rs L64-65；connections.rs L52（唯一读取点）；query.rs L232-242；metadata.rs L431-441；backup.rs L348；App.svelte L43/L153；ConnectionManager.svelte L74-89 |
| PL-02 | B | 原生保存/打开对话框插件未注册：Cargo.toml 无 tauri-plugin-dialog、main.rs 未注册、无 capabilities/ 目录；导出全部落入浏览器降级下载，备份导入「浏览 SQL」无降级完全不可用 | 新（FL-07 由 C 升 B） | Cargo.toml；main.rs L20；DatabaseBackup.svelte L199-220；DataGrid.svelte L637-646；ResultsPanel.svelte L258-296 |
| PL-03 | B | 表设计器「关闭」直接丢弃未保存更改：hasChanges 指示条存在但关闭按钮无确认弹窗 | 新 | TableDesigner.svelte L505；App.svelte L389-393 |
| PL-04 | C | 危险操作确认策略不对称：树/网格六类危险操作有 danger 确认，编辑器直接执行 DROP/TRUNCATE 无任何确认或事务提示 | 新 | page-spec.md 一.3（危险清单不含编辑器执行）；App.svelte executeQuery 无确认 |
| PL-05 | C | 多库扩展的方言能力归属硬编码：引擎/字符集/information_schema/LIMIT 语法散落在视图组件与 meta 命令，未归 driver 层（driver.rs 的 Capabilities 已定义未用） | 新（与 IA-05 同源） | TableDesigner.svelte（硬编码引擎/字符集）；metadata.rs（硬编码 information_schema）；driver.rs L11-17 |
| PL-06 | C | 无本地日志/诊断数据：全部排障信息仅 console，与「运维」画像的排障诉求不匹配 | 新（与 DS-06 同源） | 全局 grep 无日志写入；RELEASE_VERIFICATION「仍需手工核验」 |
| PL-07 | A（建议勘误，本评审不直接修改） | `docs/02_product/PRD.md` §7.1「保存会清空钥匙串密码 - 已知缺陷 重输即可」表述与代码事实不符：重输密码后钥匙串有值，但执行链路仍拿不到密码（PL-01），「重输即可」会误导后续开发 | 新 | prd.md L263；reverse-analysis.md ⑨.4.3 同样表述需同步修正 |
| PL-08 | D | 产品边界口径诚实（结构对比仅预览、备份仅 SQL、编辑门禁明示），文档-代码-设置面板三处口径一致 | 交叉（P4 正面基线） | RELEASE_CHECKLIST.md；App.svelte 设置面板 L472-477 |

**分报告问题索引**：IA-01…IA-07（信息架构）、UF-01…UF-10（用户流程）、DS-01…DS-09（数据存储）、ST-01…ST-06（状态）、PM-01…PM-07（权限）。其中与 P4 交叉引用的条目在各分报告中标注「P4 交叉」，不重复计入新发现。

---

## 四、最重要发现详述

### PL-01 数据库凭据链路断裂【B·本次评审最高优先级】

- **当前设计**：
  - 密码保存：`conn_upsert` → `ConnectionStorage::upsert`，密码非空时 `set_connection_password` 写入钥匙串（`src-tauri/src/storage/connections.rs` L76-80），`save_all` 落盘前清空密码字段（L60-67），文件确不含密码（Rust 单测覆盖）。
  - 密码读取：`get_connection_password` 全仓唯一调用点在 `load_all`（`connections.rs` L52），将密码填入内存对象；但 `ConnectionInfo.password` 标注 `#[serde(default, skip_serializing)]`（`src-tauri/src/db/driver.rs` L64-65），`conn_list` 返回前端的 JSON **不含密码**。
  - 执行链路：`query_execute`/`meta_list_databases`/`meta_list_tables`/`meta_get_table_schema`/`meta_create_database`/`meta_create_table`/`query_update_cell`/`db_export`/`db_import` 全部直接用**前端回传的 ConnectionInfo** 构建连接（`query.rs` L232-242 `build_opts`、`metadata.rs` L431-441、`backup.rs` L348），无任何「按 connection id 回查钥匙串」逻辑。前端 `selectedConnection` 来自 `conn_list`（`App.svelte` L56-57、L153-157），password 字段不存在 → 后端 serde default 得空串 → `pass(Some(""))` 以空密码认证。
- **问题**：钥匙串密码「存而不取」；带密码账户的实际行为为：表单内「测试连接」成功（表单携带手输密码，ConnectionManager.svelte L91-94）→ 保存 → 点击连接 → 加载库列表即失败（`连接失败: Access denied ... using password: NO`）。列表项 ⚡ 测试同样传无密码对象（L74-89），亦失败。开发期被掩盖的原因：`src-ui` 旧副本表单预填 `'root123456'`（src-ui/src/components/ConnectionManager.svelte L21），且历史验证（`docs/RELEASE_VERIFICATION_2026-04-22.md`）仅确认「进入运行态」，手工核验项未闭环。
- **影响**：产品核心承诺（README「连接管理→查询」主链路）对带密码账户不可用；「测试成功→使用失败」的体验极度反直觉；P4 PF-01 提出的 V1 修复（密码框「留空保持原密码」+ upsert 保留语义）**不足以修复本问题**——即使钥匙串密码完好，执行链路也拿不到它。
- **建议方向**：凭据解析下沉服务端——`query_*`/`meta_*`/`db_*` 命令改为接收 `connection_id`，由后端在执行时从钥匙串取密码；或至少在 `build_opts` 前按 id 补全密码。同步修正 P7 API 契约与 PRD §7.1 表述（PL-07）。

### PL-02 原生文件对话框插件未注册【B】

- **当前设计**：前端 `ResultsPanel.svelte` L3、`DataGrid.svelte` L3 导入 `save`，`DatabaseBackup.svelte` L3 导入 `save, open`（均来自 `@tauri-apps/plugin-dialog`，package.json 有依赖）；但 `src-tauri/Cargo.toml` 无 `tauri-plugin-dialog`，`main.rs` L20 仅注册 `tauri_plugin_shell`，且 `src-tauri/` 下无 `capabilities/` 目录（仅 gen/schemas 生成物）。
- **问题**：打包态 `save()`/`open()` 调用必然 reject（插件未注册）。导出路径：save 失败 → catch → `browserDownload`（DataGrid.svelte L637-646、ResultsPanel.svelte L258-296）——在 Tauri webview 内 `URL.createObjectURL` + `a.download` 的落点行为不可控（P4 FL-07 已存疑）；备份导入：`open()` 失败仅 `console.error`（DatabaseBackup.svelte L199-220），无降级 → `importFile` 恒空 → 「开始导入」按钮永久禁用。
- **影响**：F024/F035/F048（导出）主链路损坏，F049（导入）文件选择不可用；P4 将降级策略定为 C 类「技术实现决策」，低估了「主路径根本不存在」的事实。
- **建议方向**：P7 架构补 `tauri-plugin-dialog`（Cargo 依赖 + main.rs 注册 + capabilities 授权）；或统一改走后端文件命令（带路径作用域，见 PM-01）。FL-07 建议由 C 升 B。

### PL-03 表设计器未保存关闭无拦截【B】

- **当前设计**：设计器头部显示「有未保存的更改」指示（F042），但「关闭」按钮直接 `onClose`（TableDesigner.svelte L505）→ App 复位新建态切回网格（App.svelte L389-393），全程无确认。
- **问题**：与全应用「危险操作必确认」基线（page-spec 一.3）相悖；列定义工作（可能数十列的属性编辑）一次误点全部丢失。
- **影响**：数据丢失级体验风险；六类危险确认遗漏了「丢弃未保存设计」这一第七类。
- **建议方向**：hasChanges 为真时关闭走 `confirmAction`（tone=danger，文案明示将丢弃的变更范围）。

---

## 五、与 P4 评审的冲突 / 升级结论

| # | P4 原结论 | 本次评审结论 | 处置 |
|---|----------|--------------|------|
| 1 | PF-01/B1：编辑清空钥匙串密码，「重输密码即可恢复」；V1 修复=密码留空语义 | 「重输即可恢复」不成立（PL-01）：执行链路从不读钥匙串，重输后查询仍失败。B1 的「留空保持原密码」必要但不充分 | PF-01 维持 B，但修复方案需扩展为服务端凭据解析；prd.md §7.1 表述建议勘误（PL-07） |
| 2 | FL-07/C：导出降级浏览器下载「行为不明」，属技术实现决策 | 降级不是边缘策略而是**唯一可用路径**（插件未注册，PL-02），主路径不存在 | 建议 C→B 升级 |
| 3 | FL-05/C：查询取消需后端任务架构，范围待决策 | 维持 C，但补充事实：`query_execute` 连超时都没有（conn_test 有 5s，查询无），「执行中」态可无限期无出口（UF-02）。建议最小修复（前端超时提示）单列 B | 补充不升级原条目 |
| 4 | C8（meta_get_schema_tree 剔除）、C9（src-ui 删除） | 维持 C，无新证据冲突；补充：src-ui 预填密码 `'root123456'` 是 PL-01 曾被掩盖的环境性原因，删除时注意留档 | 维持 |
| 5 | PF-03/B：批量进度面板接线 | 维持 B；状态视角补充 ST-04（批量复用单查询状态三元组，per-statement 状态缺失） | 维持+补充 |

---

## 六、阻塞与后续

- **阻塞项（建议进入重构必办清单）**：PL-01（凭据链路）、PL-02（对话框插件）、PL-03（设计器关闭确认）、PM-01（fs_write_file 作用域）。四项均属 B 级可落地，且 PL-01/PL-02 直接决定产品核心流程是否真实可用。
- **后续建议**：PL-07 勘误由文档维护者执行；PL-04/05/06、DS-03/04/09、PM-03/05/07、ST-02、UF-07/08 留用户决策。
- 本报告及 5 个分报告共 47 条编号问题（A 建议 1、B 21、C 17、D 8；分布：PL 8 / IA 7 / UF 10 / DS 9 / ST 6 / PM 7），未改动项目任何现有文件。
