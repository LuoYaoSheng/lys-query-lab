# QueryLab 权限与安全边界评审（PM）

> 评审依据：《AI 产品重构逻辑评审规范 v1.0》· 2026-09-03
> 输入文档清单：`docs/01_reverse/REVERSE_ANALYSIS.md` §1.2/§2.3/§8（依赖与命令清单）、`docs/02_product/PRD.md` §9（安全非功能）、`docs/06_review/PRODUCT_REVIEW.md`（P4 交叉）、源码核实 `src-tauri/src/security/mod.rs`、`src-tauri/src/storage/connections.rs`、`src-tauri/src/commands/app.rs`、`src-tauri/src/commands/backup.rs`、`src-tauri/src/main.rs`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`。
> 评审范围：① keychain 访问时机与授权；② Tauri 系统能力面（插件/命令/文件系统/CSP）；③ 数据库凭据授权模型。
> 实测限制：无打包产物运行验证，Tauri 运行时权限行为按 Tauri 2 机制与源码配置推断（涉及处注明）。

---

## 一、权限清单

### 1.1 系统钥匙串（keyring 1.1.0）

| 操作 | 接口 | 调用时机（实测调用链） | 触发方 |
|------|------|------------------------|--------|
| 写入密码 | set_connection_password（security/mod.rs L11-16） | conn_upsert 且密码非空（connections.rs L79）；load_all 发现明文遗留时迁移（L41） | 用户显式保存 |
| 读取密码 | get_connection_password（L18-25） | **仅** load_all（connections.rs L52）→ 被 conn_list / conn_upsert / conn_delete 间接触发，每次批量读全部连接密码入内存 | 应用启动、每次保存/删除/列表刷新 |
| 删除密码 | delete_connection_password（L27-33） | conn_upsert 且密码为空（L76-77）；conn_delete（L94） | 用户保存（编辑不重输即误删，PF-01）/删除连接 |

授权特征：service=`com.i2kai.querylab.connection` 按连接 id 分条存取；首次访问由 OS 钥匙串按应用签名授权（macOS Keychain 弹窗机制，应用未签名时的行为【未知——无打包产物验证】）。
时机评估：读取时机与用途错配——高频批量读取（每次列表操作）服务于零消费的用途，而执行链路需要时反而不读（PM-04）。

### 1.2 Tauri 系统能力面

| 能力 | 声明/注册 | 前端消费 | 评估 |
|------|-----------|----------|------|
| tauri-plugin-shell | Cargo.toml L11 + main.rs L20 注册 + tauri.conf.json `plugins.shell.open:true` | **无任何前端调用**（全局 grep 无 import/usage） | 未消费能力面（PM-03） |
| @tauri-apps/plugin-dialog（save/open） | 仅前端 npm 依赖（package.json）；**Rust 侧未引入 crate、未注册、无 capabilities** | ResultsPanel/DataGrid（save）、DatabaseBackup（save+open） | 必然运行时失败（PL-02）；讽刺的是这使 dialog 权限问题不存在——因为功能本身不存在 |
| 自定义命令 fs_write_file（app.rs L29-36） | generate_handler 注册 | 导出链路 3 组件调用 | **无路径作用域限制 + create_dir_all**（PM-01） |
| 自定义命令 db_import（backup.rs） | generate_handler 注册 | 备份导入 | **任意路径文件读取**（前端传 file_path，PM-02） |
| 其余 14 个命令（conn_*/meta_*/query_*/db_export/app_get_info） | generate_handler 注册（main.rs L21-38，共 16 个） | 各组件 | 数据库操作，无文件系统能力 |
| CSP（tauri.conf.json security.csp） | **未配置**（配置文件无 security 节） | — | 默认无内容安全策略（PM-01 关联） |
| capabilities 目录（Tauri 2 ACL） | **不存在**（src-tauri/ 下仅 gen/schemas 生成物） | — | 未启用显式能力授权模型（PM-07） |
| withGlobalTauri | false（tauri.conf.json L13） | — | 正面（缩小注入面） |

### 1.3 数据库凭据授权

| 事项 | 现状 | 评估 |
|------|------|------|
| 凭据持有模型 | 前端全量持有连接参数对象并逐命令回传（ConnectionInfo 作为每个命令参数）；密码因序列化跳过而恒缺（PL-01） | 授权链路断链；凭据最小化方向应为前端仅持 id |
| 语句级授权 | 无：query_execute 执行任意 SQL（客户端本质），树/网格六类危险操作有 UI 确认，**编辑器直接执行 DROP/TRUNCATE 无确认**（PL-04） | 确认策略不对称 |
| 只读模式 | 无「只读连接」开关（连接表单 5 字段，ConnectionManager L16-25） | 缺失（PM-05） |
| 数据库权限错误透传 | MySQL 报错原文直接展示（错误态/红块） | 无解释性包装（UF-07 交叉） |

---

## 二、问题清单

### PM-01 fs_write_file 提供无作用域限制的任意路径写原语，且无 CSP【B·新】
- 当前设计：`fs_write_file(path, contents)` 对任意绝对/相对路径 `fs::write`，且 `create_dir_all` 自动建父目录（app.rs L29-36）；tauri.conf.json 未配置 CSP；Tauri 2 自定义命令默认对 webview 全量可调（无 capabilities 收敛）。
- 问题：该命令等价于「webview 内任意代码可写文件系统任意位置（含覆盖用户既有文件）」。当前合法调用都发生在 save 对话框之后，但命令本身不做任何校验；一旦 webview 出现 XSS（无 CSP 前提下风险放大），即获得持久化写原语。
- 影响：权限面与实际需求（导出落盘）严重不匹配；桌面应用供应链风险。
- 建议方向：① tauri.conf.json 配置 CSP；② fs_write_file 限制写入目录白名单（用户通过对话框选择的路径 / 导出目录），拒绝已存在文件的静默覆盖或至少校验来自对话框的结果；③ P7 建立 capabilities 声明（与 PM-07 合并落地）。

### PM-02 db_import 提供任意路径文件读原语【B·新】
- 当前设计：db_import 的 file_path 由前端任意传入（backup.rs ImportParams），后端直接读取并逐句执行其内容。
- 问题：与 PM-01 对称的读取面：可读取任意本地文件内容并作为 SQL 执行（错误信息还可能回显文件内容片段）。
- 影响：同 PM-01 的滥用面；正常用途（用户选的 .sql）不需要任意路径能力。
- 建议方向：路径来源校验（dialog 插件恢复后由对话框返回），或限制扩展名/目录；与 PM-01 一并收敛。

### PM-03 shell 插件注册但零消费【C·新】
- 当前设计：main.rs L20 注册 `tauri_plugin_shell::init()`；tauri.conf.json `plugins.shell.open:true`；前端无任何 shell/open 调用（全局 grep 证实）。
- 问题：未使用的能力面（最小权限原则违背）；`open:true` 是宽松配置（允许打开任意 URL/程序，具体语义随 Tauri 2 shell 权限模型）。
- 影响：低（无消费方即无直接攻击面），但属冗余攻击面与维护噪音。
- 建议方向：用户决策——移除，或保留并声明用途（如未来「打开帮助文档/官网」）并配 capabilities 精确授权。

### PM-04 keychain 密码批量读入内存的时机过宽【B·新，与 DS-01 一体】
- 当前设计：每次 conn_list/upsert/delete 触发 load_all → 循环读取**全部**连接密码入内存（connections.rs L51-53），无论调用目的；读取结果仅用于随后被 skip_serializing 丢弃的序列化。
- 问题：密钥访问违反最小化：频率高（启动 + 每次连接管理操作）、范围大（全部条目）、用途空转；而执行链路真正需要密码时反而零读取。
- 影响：明文驻留内存窗口放大；与 PL-01 合并构成「存取两头都错」的完整缺陷。
- 建议方向：随 PL-01 一并重构——load_all 不再回填密码，执行前按 id 单条读取即用即弃。

### PM-05 无只读连接/语句防护选项【C·新】
- 当前设计：连接模型无「只读」属性；query_execute 对语句类型不设限（SELECT 与 DROP 同通道）；防护完全依赖 UI 确认且不含编辑器路径（PL-04）。
- 问题：数据分析画像（小赵）与「连接生产库」场景缺乏降权手段；同类产品（DBeaver/Navicat）普遍提供只读连接或事务保护选项。
- 影响：误操作风险集中暴露在编辑器路径。
- 建议方向：范围决策——连接级只读开关（后端拒绝非 SELECT 或 SET SESSION TRANSACTION READ ONLY）是否进入 V1。

### PM-06 密码不出现在序列化输出与磁盘文件【D·正面】
- 当前设计：`#[serde(default, skip_serializing)]`（driver.rs L64-65）确保密码不进任何 JSON；save_all 落盘前 clear（connections.rs L60-67）；Rust 单测 `save_all_does_not_persist_password_field` 锁定该行为（L127-150）。
- 问题：无（该正面设计正确且被测试保护）。
- 建议方向：不动；重构 PL-01 时保持该属性。

### PM-07 未启用 Tauri 2 capabilities 显式授权模型【C·新】
- 当前设计：src-tauri 无 capabilities/ 目录；权限收敛依赖默认行为。
- 问题：Tauri 2 的 ACL（能力/权限声明）未使用，插件与命令的授权边界无显式文档化载体；P7 重构引入 dialog/fs 等插件时应同步建立。
- 影响：架构治理项，非当前运行时缺陷（自定义命令不依赖 ACL）。
- 建议方向：P7 落地 capabilities/default.json（core 最小集 + dialog + 按需 fs scope），作为 PM-01/02/03 的制度化收口。

---

## 三、小结

权限侧结论：**钥匙串使用「存对读错」、系统能力面「该有的没有（dialog）、不该有的过大（fs_write_file/db_import 任意路径）」**。正面基线是密码不落盘且有测试锁定（PM-06）、withGlobalTauri 关闭。共 7 项（B3、C3、D1），其中 PM-01/02/04 应与 PL-01/PL-02 同批进入重构必办清单；PM-03/05/07 留用户决策。
