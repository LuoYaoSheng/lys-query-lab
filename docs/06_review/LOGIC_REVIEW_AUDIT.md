# QueryLab 产品逻辑评审元审计报告

> 审计对象：`docs/product-review/` 六件套（PRODUCT_LOGIC_REVIEW / INFORMATION_ARCHITECTURE_REVIEW / USER_FLOW_REVIEW / DATA_STORAGE_REVIEW / STATE_REVIEW / PERMISSION_REVIEW，2026-09-03 产出）
> 审计依据：《AI 产品重构逻辑评审规范 v1.0》四维检查点（结构合规 / 证据可追溯性 / 跨产物一致性 / 分级合理性）+ 本项目特别关注项（推翻 P4 PF-01/B1 缓解结论的证据链复核）
> 审计日期：2026-09-03 · 审计方式：独立元审计，逐条源码复核，只评审、不修改被审产物以外的任何项目文件
> 规范原文状态：**未随任务提供、未入库**——六件套头部已诚实声明「规范原文未随任务提供，§17 八项验收清单按任务给定评审重点（§2-§14）归纳执行，未虚构规范条文」。本审计的十三重点映射同样基于产物内容与任务检查点归纳，若与规范实际条文有出入需人工二次比对（见 E-01）。

---

## 一、审计结论：**通过**

- 47 条编号问题（A1/B21/C17/D8）内部统计自洽（逐文件复核：PL 8 = A1·B3·C3·D1；IA 7 = B4·C2·D1；UF 10 = B5·C3·D2；DS 9 = B3·C4·D2；ST 6 = B3·C2·D1；PM 7 = B3·C3·D1，合计与总报告 §六 完全一致）。
- 证据抽查 17 组 16 组完全属实，唯一不符为 1 处行号偏差（ST-03 的 exportLoading 声明行 L25→实际 L18，结论不受影响），已按 A 类处置规则直接修复（2 处，见 §四）。
- **推翻 P4 的证据链（PL-01）三要素全部源码核实成立**，且四个 v2.0 后续产物（DATA_FLOW / PERMISSION / ERROR_CODE / USER_FLOW）全部采纳推翻后口径（见 §七专项结论）。
- E 类留档 4 项，均不构成返工理由。

---

## 二、A 维度：结构合规

### A1 六文件齐全与头部要件

6/6 文件齐全；每件头部均含「评审依据（《AI 产品重构逻辑评审规范 v1.0》· 2026-09-03）+ 输入文档清单 + 实测限制声明」。**通过**。

### A2 问题四段式（当前设计/问题/影响/建议方向）覆盖率

统计口径：有效问题 = 排除正面基线/观察条目（PL-08、IA-06、UF-09/10、DS-07/08、ST-05、PM-06 共 8 条，此类按惯例三段、无「影响」段，合理）。

| 文件 | 有效问题 | 完整四段 | 部分/交叉承载 | 覆盖率 |
|------|----------|----------|----------------|--------|
| PRODUCT_LOGIC | 8 | 3（PL-01/02/03 详述）+ 2 由分报告承载（PL-05→IA-05、PL-06→DS-06） | 3 表格简式（PL-04/07/08*） | 5/8（PL-07 为勘误建议类、PL-08 正面，简式合理；PL-04 仅在 PM §1.3 表格一行提及，无独立四段展开——轻微缺口） |
| INFORMATION_ARCH | 6 | 6 | — | 100% |
| USER_FLOW | 8 | 5（UF-01/02/04/07/08） | 1 部分（UF-03）+ 2 纯交叉（UF-05/06，注明「见 PL-03/02」） | 62.5% 独立 / +交叉承载 100% |
| DATA_STORAGE | 7 | 6 | 1 缺影响段（DS-06） | 85.7% |
| STATE | 5 | 5 | — | 100% |
| PERMISSION | 6 | 6 | — | 100% |

合计：有效问题 40 条，完整四段 31 条（77.5%）；交叉引用式承载（注明出处、避免重复展开）后 37/40（92.5%）。**达标**（UF-05/06 与 DS-06 为可接受偏差，留观察）。

### A3 §17 八项验收清单

PRODUCT_LOGIC_REVIEW §二以表格逐项给出「结论（通过/有条件通过/不通过）+ 关键证据（文件+行号）」，8/8 齐全。**通过**。

### A4 视角合规

抽查六件套全部「建议方向」：均为方向性表述（如「右键菜单增加设计表项」「凭据解析下沉服务端」），**无 UI 稿、无代码片段、无直接修改源码的操作性建议**；PL-07 勘误建议明确标注「本评审不直接修改」。**通过**。

### A5 规范 §2-§14 十三重点映射表

> 注：规范原文不可得（E-01），下表为审计员从六件套内容与任务给定检查点归纳；「产物自述编号」列记录六件套内部自称的评审重点编号，两者编号体系已对齐（重点 N ↔ §N+1）。

| # | 规范节 | 评审重点（归纳） | 承载位置 | 覆盖 |
|---|--------|------------------|----------|------|
| 1 | §2 | 产品目标明确性与现状一致性 | PL §一/§二#1、PL-08 | ✓ |
| 2 | §3 | 页面职责单一性 | PL §二#3、IA §二逐页归属表 | ✓ |
| 3 | §4 | 导航层级与信息架构 | IA §一导航层级图、IA-01/02 | ✓ |
| 4 | §5 | 核心流程最短路径对比 | UF §一（F1-F8）、PL §二#2 | ✓ |
| 5 | §6 | 流程五要素完整性 | UF §二总表、PL §二#4 | ✓ |
| 6 | §7 | 可预测性抽查（要求 ≥5 例） | IA §三（7 例，超要求） | ✓ |
| 7 | §8 | 多数据库类型下功能归属 | IA §四（产物自述「评审重点 7」） | ✓ |
| 8 | §9 | 数据存储分类与生命周期五段 | DS §一/§二、PL §二#6 | ✓ |
| 9 | §10 | 状态模型（必要/重复/冲突/缺失） | ST §一/§二、PL §二#7 | ✓ |
| 10 | §11 | 权限与安全（keychain 时机/系统能力/数据库凭据） | PM 全文、PL §二#8 | ✓ |
| 11 | §12 | 日志与诊断数据 | DS-06、PL-06 | ✓ |
| 12 | §13 | 异常分支专项（连接失败/查询超时/权限拒绝） | UF §三（产物自述「评审重点 12」） | ✓ |
| 13 | §14 | 敏感数据与隐私边界 | DS-03/05、DS-07、PM-06 | ✓ |

**13/13 全覆盖，无缺失**（以归纳口径计）。

**A 维度小结：5 项检查点全部通过（结构合规度 100%，其中四段式以承载口径计 92.5%）。**

---

## 三、B 维度：证据可追溯性

### B6 证据抽查记录表（17 组，重点覆盖任务指定 6 项）

| # | 发现 | 产物引用 | 审计复核结果 |
|---|------|----------|--------------|
| 1 | PL-01a serde skip_serializing | driver.rs L64-65 | ✓ `#[serde(default, skip_serializing)] pub password: String` 逐字属实，密码不进任何 JSON |
| 2 | PL-01b 唯一读取点 | connections.rs L52 | ✓ 全仓 grep `get_connection_password`：定义（security/mod.rs L18）+ import + **唯一调用** L52，别无调用点 |
| 3 | PL-01c 9 命令不回查钥匙串 | query.rs L232-242；metadata.rs L431-441；backup.rs L348 | ✓ 逐一核实 query_execute（query.rs L106-111）、query_update_cell（L47-48）、meta_list_databases（metadata.rs L224-225）、meta_list_tables（L242-247）、meta_get_table_schema（L283-288）、meta_create_database（L72-73）、meta_create_table（L105-106）、db_export（backup.rs L51-52）、db_import（L145-146）——全部直接用前端回传 ConnectionInfo → 三处 build_opts 均 `.pass(Some(conn.password.clone()))`，无任何按 id 回查逻辑 |
| 4 | PL-01d 前端无密码对象 | App.svelte L56-57/L153-157；ConnectionManager L74-94 | ✓ conn_list 结果存 connections（L56-57）、query_execute 传 selectedConnection（L153-157）；表单测试 {...formData} 含手输密码（L92-93）而列表 ⚡ 测试传无密码 conn（L74-89）——「测试成功→使用失败」机理成立；conn_test 独立 build_mysql_opts（connection.rs L55-56）不回查钥匙串 |
| 5 | PL-01e 掩盖原因 | src-ui ConnectionManager L21；RELEASE_VERIFICATION | ✓ src-ui 旧副本 L21 `password: 'root123456'` 属实；RELEASE_VERIFICATION_2026-04-22.md L16 仅勾选「能进入运行态」，L58-71「仍需手工核验的项」（含 L66 加载库列表）未闭环 |
| 6 | PL-02 对话框插件未注册 | Cargo.toml；main.rs L20；package.json | ✓ Cargo.toml 依赖仅 tauri-plugin-shell（L11）无 dialog；main.rs L20 仅注册 shell、L21-38 generate_handler 16 命令；src-tauri 无 capabilities/ 目录（仅 gen/schemas 生成物）；package.json L33 前端 npm 依赖 @tauri-apps/plugin-dialog ^2.4.2 存在（「前端有依赖、Rust 未注册」的不对称属实）；DatabaseBackup L3（save+open）/L199-221（open 失败仅 console.error）/L386（importFile 空则禁用）、DataGrid L3/L630-646、ResultsPanel L3/L258-288 全部属实 |
| 7 | PL-03 设计器关闭丢弃 | TableDesigner L505；App L389-393 | ✓ L505 关闭按钮直接 on:click={onClose}，L502-503 hasChanges 指示条存在，App L389-393 onClose 复位切网格，全程无确认 |
| 8 | PM-01 fs_write_file 无限制+无 CSP | app.rs L29-36；tauri.conf.json | ✓ L29-36 任意 path fs::write + create_dir_all，无白名单校验；tauri.conf.json 无 security 节（CSP 未配置）、withGlobalTauri:false（L13）、plugins.shell.open:true（L30-33）均属实 |
| 9 | PM-02 db_import 任意路径读 | backup.rs | ✓ ImportParams.file_path 前端传入（L19/L36），L150 `std::fs::read_to_string(&params.file_path)` 直接读取并逐句执行 |
| 10 | ST-01 乐观连接断言 | App.svelte L104；query.rs L116；metadata.rs L226 | ✓ L104 点击瞬间写 `Connected to ${conn.name}`；每命令独立 `mysql_async::Conn::new`（query.rs L116、metadata.rs L226 等），无持久连接可探测 |
| 11 | DS-09 孤儿钥匙串 | connections.rs L34-35 | ✓ `serde_json::from_str(&content).unwrap_or_default()` 解析失败静默返回空列表，后续 save_all 覆盖文件 |
| 12 | ST-03 exportLoading 死状态 | ResultsPanel「L25」 | ✗→已修复：声明实际在 **L18**（L25 是 canExport），L340/L348 仅 disabled 绑定、无置真赋值——结论（恒 false）不变，行号 A 类修复 2 处 |
| 13 | IA-01 设计表入口两跳 | page-spec §一.6；App L306-321 | ✓ page-spec L15 右键菜单恰 4 项（刷新/重命名/清空/删除）；「📋 设计表」按钮 L306-321 仅 currentTableName 非空时出现 |
| 14 | UF-02 查询无超时 | connection.rs L59-64；query.rs | ✓ conn_test 有 `tokio::time::timeout(Duration::from_secs(5))`（L59-64）；query.rs 全文无 timeout，不对称属实 |
| 15 | PM-04 密码批量读入 | conn_list L16；connections.rs L51-53 | ✓ conn_list/conn_upsert/conn_delete 均经 ConnectionStorage 触发 load_all（connection.rs L16/L30/L39→storage L82/L96），L51-53 循环读取全部连接密码入内存 |
| 16 | DS-03 敏感词七类正则 | SqlEditor L33-41 | ✓ 七类正则逐字属实（password/secret/token/api_key/access_key/private_key/credential），仅关键词不含数据字面量 |
| 17 | PM-06 单测锁定 | connections.rs L127-150 | ✓ `save_all_does_not_persist_password_field` 存在于 L127-150，断言文件不含密码 |

**抽查通过率：16/17 全对 + 1 处行号偏差（94.1%；A 类修复后 100%）。任务指定的 6 个关键发现（PL-01 全链/PL-02/PL-03/PM-01/02/ST-01）全部属实，无一 E 级。**

### B7 【未知】标记使用

六件套共 6 处【未知】（IA×2、PL/UF/ST/PM 各 1），逐处检查均附原因（「无运行实例可证」「并发场景未实测，源码无互斥锁」「需实测」「无打包产物验证」），且各文件头部有统一的实测限制声明。**使用规范**。

---

## 四、A 类修复清单（已直接修复，仅动 docs/product-review/ 六件套）

| # | 文件 | 修复内容 |
|---|------|----------|
| A-1 | STATE_REVIEW.md ST-03 | exportLoading 声明行号 L25 → **L18**，并补注「仅 L340/L348 disabled 绑定」 |
| A-2 | PRODUCT_LOGIC_REVIEW.md §17 表第 7 项 | 证据列 ResultsPanel.svelte L25 → **L18**（与 A-1 同源错误） |

---

## 五、E 类清单（留档不改）

| # | 事项 | 说明 |
|---|------|------|
| E-01 | 规范原文缺失 | 《AI 产品重构逻辑评审规范 v1.0》未入库，§2-§14/§17 条目编号与验收项表述无法逐字核对。六件套已诚实声明并按任务给定重点归纳、未虚构条文，本审计的十三重点映射同理。建议规范入库后做一次编号对齐 |
| E-02 | DATA_FLOW.md §4 表笔误 | `docs/04_architecture/DATA_FLOW.md` L113「**连连接**密码」重复字（应为「连接密码」）。不在本次处置范围（只许改六件套），留档待文档维护者修正 |
| E-03 | HTML_QA_REPORT 口径衔接不完整 | `docs/09_test/HTML_QA_REPORT.md` 验收 V1 原型连接旅程时呈现 B1（密码留空保持原密码）判 PASS，但未注明 PL-01 的「B1 必要但不充分」警示。**不构成事实冲突**：验收对象是纯前端模拟原型（无真实后端/凭据链路），PL-01 属运行时代码缺陷在原型范围外；但作为引用 USER_FLOW.md（旅程一含 PL-01 标注）为基线的报告，建议后续补一行口径衔接说明 |
| E-04 | 分级微观察（不影响结论） | ① ST-03 定 B 而自评「影响：小」——因绑定 PL-02 同批落地，可接受；② PL-04（编辑器危险语句无确认）在总报告为 C，仅 PM §1.3 一行提及、无独立四段展开，若用户决策时建议先补齐细则；③ UF-05/06 纯交叉引用无独立四段式（合规性取决于规范对交叉引用的容忍度） |

---

## 六、C 维度：跨产物一致性

### C8 引用 P4 编号核对

六件套引用的全部 P4 编号在 `docs/06_review/PRODUCT_REVIEW.md` 中逐一存在：PF-01/03/04/05/09/10/11、PP-02/07、FL-01/02/03/05/07/08/09/10、B1/B3（5.1 表）、C5/C8/C9/C12、A2/A3（第七节）。**零悬空引用**。

**推翻 P4 的留痕清晰度**：PL §五冲突表第 1 行明确「PF-01 维持 B，但修复方案需扩展为服务端凭据解析」；PL-01 详述含「P4 PF-01 提出的 V1 修复……不足以修复本问题——即使钥匙串密码完好，执行链路也拿不到它」；问题汇总表 PL-01 行标注「新（修正 PF-01 缓解结论）」、PL-07 勘误项指向 PRD L263 与 REVERSE L676 两处旧表述。**留痕完整**。

### C9 旧路径残留 grep

全仓 `*.md` grep `prototype/app-prototype`：4 处命中，全部为历史注记形式（「原 prototype/app-prototype.html，现位于 prototype/v0-old/」），分别位于 COVERAGE_CHECKLIST L3、HTML_V0_ACCEPTANCE L5、PRODUCT_REVIEW L5（注记式）与 L280（A2 勘误记录）。**六件套内残留为零**；非注记式残留为零。**通过**。

### C10 与 HTML_QA_REPORT 冲突检查

- PL-03 口径：QA 报告 L109/137/194 明确「关闭丢弃无确认 = PL-03 口径如实保留」，与 USER_FLOW_REVIEW UF-05 一致——**无冲突**。
- PL-01 口径：QA 报告未直接引用 PL-01（原型为模拟态，凭据链路不在验收范围），其引用的基线 USER_FLOW.md 旅程一已含 PL-01 标注——无事实冲突，口径衔接不完整处见 E-03。
- 密码口径（QA L66/L170「连接对象不含 password 字段 = F005 钥匙串口径」）：与 PM-06/DS 正面基线一致——**无冲突**。

### C11 与 v2.0 新产物一致性（PL-01 口径采纳核查）

| 产物 | 采纳证据 | 结论 |
|------|----------|------|
| `docs/04_architecture/DATA_FLOW.md` | §2.2「密码 skip_serializing 链路事实（PL-01 核心）」完整复现三要素（L64-65 / 唯一读取点 L51-53 / 9 命令 build_opts → `pass(Some(""))`）；§2.3/§3.2 目标态=按 connection_id 服务端解析 | ✓ 已采纳 |
| `docs/08_development/PERMISSION.md` | §1.1「读取时机……服务于零消费的用途（PM-04）」；§1.3「密码因 skip_serializing 恒缺（PL-01）——授权链路断链」；§3.3 凭据流目标态时序图（单条读取、即用即弃） | ✓ 已采纳 |
| `docs/08_development/ERROR_CODE.md` | §1.4「Access denied … using password: NO 原文不指向真实原因（PL-01/UF-01）」；§2.1 新增 `CONN_NO_PASSWORD`（PL-01 修复后分流码，源 UF-01 过渡方案） | ✓ 已采纳 |
| `docs/03_flow/USER_FLOW.md` | 旅程一节点「⚠️ 已知缺陷 PL-01 加载库即失败 Access denied 测试成功→使用失败」；旅程二/五标注 PL-02、旅程四标注 PL-03/IA-01 | ✓ 已采纳 |

**四个后续产物全部采纳推翻后口径，无一处残留「重输即可恢复」旧口径。通过。**

---

## 七、D 维度：分级合理性 + 推翻 P4 证据链专项复核

### D12 分级抽查（15 项）

| 编号 | 定级 | 审计判断 |
|------|------|----------|
| PL-01 | B | ✓ 核心流程（连接→查询）对带密码账户断裂，任务指定应为 B，符合 |
| PL-02 | B | ✓ 导出主链路损坏 + 导入不可用，核心流程断裂级，符合 |
| PL-03 | B | ✓ 数据丢失级体验风险 + 违背全应用危险确认基线，符合 |
| PL-04 | C | ✓ SQL 客户端本质 vs 确认对称属产品范围决策，留 C 合理 |
| IA-01 | B | ✓ 右键菜单加项属 V1 低成本落地，符合项目 B 的定义 |
| IA-05 | C | ✓ 扩库方言归属属范围决策，合理 |
| DS-01 | B | ✓ 与 PL-01 同根的数据流断链，符合 |
| DS-03 | C | ✓ 敏感策略档位属产品边界决策，合理 |
| DS-05 | B | ✓ 跨环境误执行（测试库语句跑生产连接）是实际数据风险，B 合理 |
| DS-09 | C | ✓ 低频不可自愈，C 合理 |
| ST-01 | B | ✓ 全应用唯一「承诺事实但无法兑现」的状态，修复成本低收益高，合理 |
| ST-03 | B | △ 影响自评「小」，因绑定 PL-02 同批落地定 B——可接受（E-04①） |
| PM-01/02 | B | ✓ 任意路径写/读原语 + 无 CSP，安全面与需求严重不匹配，B 合理 |
| PM-04 | B | ✓ 与 PL-01 一体（「存取两头都错」），合理 |
| UF-02 | B | ✓ 最小修复（超时）单列 B 有明确依据（与 conn_test 5s 不对称），合理 |

**抽查 15 项：14 项完全合理、1 项（ST-03）可接受偏差。PL-01/PL-02 均为 B 级，符合任务要求。**

### 推翻 P4 证据链复核专项结论（本审计最重要事项）

**结论：推翻成立，证据链完全扎实。** 三要素逐项源码复核：

1. **serde skip_serializing**：`src-tauri/src/db/driver.rs` L64-65 逐字核实 `#[serde(default, skip_serializing)] pub password: String`——密码不进 conn_list IPC 返回、不进磁盘 JSON（双保险：save_all 落盘前逐条 clear，connections.rs L60-67，且有 L127-150 单测锁定）。前端连接对象**结构性不含密码**，非时序问题。
2. **唯一读取点**：`get_connection_password` 全仓 grep 仅 3 处命中——security/mod.rs L18（定义）、connections.rs L5（import）、connections.rs L52（**唯一调用**，位于 load_all）。钥匙串读取从未进入任何执行命令路径。
3. **9 命令不回查钥匙串**：逐一核实 query_execute / query_update_cell / meta_list_databases / meta_list_tables / meta_get_table_schema / meta_create_database / meta_create_table / db_export / db_import——九个命令全部直接以前端回传的 ConnectionInfo 经三处同构 build_opts（query.rs L232-242、metadata.rs L431-441、backup.rs L348）取 `.pass(Some(conn.password.clone()))` 建连，**不存在任何按 connection id 回查钥匙串的代码路径**。
4. **推论闭合**：前端 selectedConnection 源自 conn_list（App.svelte L56-57）→ 无 password 字段 → Rust serde default 得空串 → 空密码认证 → 带密码账户点击连接后加载库即 `Access denied … using password: NO`。而表单内「测试连接」因携带手输密码（ConnectionManager L92-93）且 conn_test 走独立 build_mysql_opts（connection.rs L55-56）而成功——「测试成功→使用失败」的假阳性机理完整成立。列表态 ⚡ 测试传无密码对象必失败（L74-89）亦核实。
5. **「重输即可恢复」为何不成立**：重输密码后钥匙串有条目（upsert L79 写入），但执行链路消费的是前端回传对象（恒空密码）而非钥匙串——重输只修复了存储侧，使用侧断链依旧。P4 PF-01/B1 的「留空保持原密码」只解决「编辑时误删钥匙串条目」（存储侧自毁），对使用侧断链无效——**必要而不充分**的判定准确。需勘误的两处旧表述（PRD.md L263「重输即可」、REVERSE_ANALYSIS L676「需重输密码否则后续连接失败」）均实际存在，PL-07 勘误指向准确。
6. **掩盖原因佐证**：src-ui 旧副本 ConnectionManager.svelte L21 预填 `password: 'root123456'` 属实（开发期表单态测试恒带密码，掩盖断裂）；RELEASE_VERIFICATION_2026-04-22.md L16 仅确认「能进入运行态」，L58-71 手工核验清单（含 L66「选中连接后手工确认数据库列表可以加载」）全部未勾选闭环——「历史验证未覆盖本路径」的论证成立。
7. **后续产物采纳**：见 §六 C11——DATA_FLOW/PERMISSION/ERROR_CODE/USER_FLOW 四产物全部按推翻后口径编制，无一残留旧口径。

---

## 八、审计处置汇总

- A 类修复：**2 处**（已完成，仅改动六件套中的 STATE_REVIEW.md 与 PRODUCT_LOGIC_REVIEW.md 各 1 处行号）。
- E 类留档：**4 项**（E-01 规范原文缺失 / E-02 DATA_FLOW 笔误 / E-03 QA 报告口径衔接 / E-04 分级与四段式微观察）。
- 未改动任何源码、HTML、src-ui/ 及六件套以外的文档；未执行 git commit。
- 阻塞项：无。

---

## 九、复核方式附注（可追溯性）

本审计全部结论基于以下一手复核：driver.rs / connections.rs / security/mod.rs / query.rs / metadata.rs / backup.rs / app.rs / connection.rs / main.rs / Cargo.toml / tauri.conf.json 全文或关键段；App.svelte 16 组行号段、ConnectionManager（src 与 src-ui 两份）/ DatabaseBackup / DataGrid / ResultsPanel / TableDesigner / SqlEditor / DataSync 引用行；`get_connection_password`、`BatchProgressPanel`、`exportLoading`、`prototype/app-prototype`、日志写入、shell 调用六组定向 grep；P4 PRODUCT_REVIEW、四个 v2.0 产物、HTML_QA_REPORT、PRD §7.1、REVERSE §⑨.4.3、RELEASE 两文件的交叉比对。
