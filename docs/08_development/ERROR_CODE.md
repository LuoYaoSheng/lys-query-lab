# QueryLab 错误处理与错误码规范（ERROR_CODE）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 现状盘点来源：`src-tauri/src/core/errors.rs`、`src-tauri/src/commands/*.rs`（逐文件核实）、`src/lib/notifications.js`、`src/components/*.svelte`；规范建议来源：`docs/08_development/API_SPEC.md`（原 api-design.md）§4 错误码草案（供 C14 决策时启用）与 `docs/04_architecture/SYSTEM_ARCH.md` §2 要点 1。

---

## 1. 现状错误处理盘点

### 1.1 后端：双轨并存，AppError 未接线

| 轨道 | 位置 | 现状 |
|------|------|------|
| **实际轨道**：`Result<T, String>` | 全部 16 个命令签名（grep 核实：`Result<..., String>`） | 错误为人类可读中文字符串直接透传前端（如「连接失败: {e}」「写入文件失败: {e}」「XX失败: err」） |
| **占位轨道**：`AppError` 结构体 | `src-tauri/src/core/errors.rs`（`#![allow(dead_code)]`） | 已定义 5 个构造器（见 §1.2）+ `From<anyhow::Error>`，但命令层零调用——dead code |

### 1.2 AppError 已定义错误码（占位，未启用）

| 构造器 | code | 语义 |
|--------|------|------|
| db_connection_failed | `DB_CONN_FAILED` | 数据库连接失败 |
| db_query_failed | `DB_QUERY_FAILED` | 查询失败 |
| conn_not_found | `CONN_NOT_FOUND` | 连接不存在: {id} |
| invalid_params | `INVALID_PARAMS` | 参数错误 |
| feature_required | `FEATURE_REQUIRED` | 需要 VIP 权限: {feature}（注：本项目无 VIP 概念，属模板残留语义【来源：errors.rs L52-58 原文】） |

结构：`AppError{code, message, detail?}`，serde 可序列化（detail 为 None 时跳过）。

### 1.3 前端：无错误分类层

- 错误消费为「字符串直显」：结果面板错误态全文展示、红块、toast、状态栏一行 `Query failed`（P4 PP-07）。
- 无错误码解析、无重试/直达编辑类指引（UF-07：连接类错误三处三种反馈强度）。
- 8 个组件各自 invoke，无统一错误映射（SYSTEM_ARCH §2 要点 1 指出该缺失）。

### 1.4 已知错误链路缺陷（编号索引）

| 缺陷 | 编号 |
|------|------|
| 带密码账户错误文案为 MySQL `Access denied ... using password: NO` 原文，不指向真实原因（密码未传递） | PL-01/UF-01 |
| loadDatabases / open() 失败仅 console.error 静默 | UF-07/UF-06 |
| SQL 报错无错误定位（行号/语句序号） | FL-04（C） |
| 状态栏错误信息只有一行 | PP-07（D） |

## 2. 错误码规范建议（目标态）

> 依据：API_SPEC §4 草案（C14 决策时启用）+ errors.rs 既有前缀 + 本次盘点补全。命名沿用「域前缀 + 动词完成态」大写下划线风格。

### 2.1 码表

| 域 | 错误码 | 触发场景 | 前端五态/反馈归类 | 现状对应文案 |
|----|--------|----------|-------------------|----------------|
| DB | `DB_CONN_FAILED` | 连接失败/超时（conn_test 5s）/无效连接参数 | 错误态/红块 + 建议直达「编辑连接」 | 连接失败: {e} / 连接超时（5秒）/ 无效的连接参数 |
| DB | `DB_SQL_ERROR`（附 MySQL errno） | SQL 语法/执行错误 | 错误态全文 + 语句定位（配 FL-04） | MySQL 报错原文透传 |
| DB | `DB_ACCESS_DENIED` | MySQL access denied（errno 1045 等） | 错误态 + 解释性包装（当前用户无权/凭据问题分流） | 报错原文（无包装，UF-07） |
| DB | `DB_UPDATE_FAILED` | 单元格更新/行写入失败 | 红横幅/错误块 | 更新失败类文案 |
| DB | `DB_NOT_FOUND` | 目标库/表不存在（DDL 对象缺失） | 错误态 | 【未知——现无独立分支】 |
| CONN | `CONN_NOT_FOUND` | connection_id 不存在（目标态凭据下沉后必需） | toast | errors.rs 已定义（未启用） |
| CONN | `CONN_NO_PASSWORD` | 钥匙串无该连接密码（PL-01 修复后的分流码） | 错误态 + 直达「编辑连接重输密码」 | 【新增建议，源 UF-01 识别 using password: NO 的过渡方案】 |
| KEYRING | `KEYRING_FAILED` | 钥匙串读写删失败（security/mod.rs 三函数） | 错误态 | 无法写入/读取/删除系统钥匙串密码 |
| FS | `FS_WRITE_FAILED` | fs_write_file 目录创建/写入失败 | 导出结果块红/toast | 创建目录失败: {e} / 写入文件失败: {e} |
| FS | `FS_PATH_REJECTED` | 路径超出白名单（PM-01 修复后新增） | toast | 【新增建议】 |
| FS | `FS_READ_FAILED` | db_import 读取 .sql 失败 | 导入结果块红 | 读取失败类文案【具体原文未逐一核对】 |
| PARAM | `INVALID_PARAMS` | 参数校验失败（空库名、源=目标库等） | 表单级错误 | 源数据库和目标数据库不能相同 等 |
| APP | `INTERNAL_ERROR` | 未分类异常（From<anyhow>） | 错误态 | err.to_string() |

### 2.2 载体与接线建议

1. 启用 `AppError`（core/errors.rs）作为命令统一返回：`Result<T, AppError>`（serde 序列化 code/message/detail），替换 `Result<T, String>`；`FEATURE_REQUIRED` 构造器删除或改义（无 VIP 概念）。
2. 前端建立 `src/lib/api/` 统一 invoke 层：按 code 映射用户文案、五态归类与恢复动作（直达编辑/重试），消除 8 组件各自拼字符串（SYSTEM_ARCH §2）。
3. 连接类错误三处（表单测试/加载库/查询执行）统一反馈组件（UF-07 建议），至少消除 loadDatabases 静默分支。
4. 错误码与恢复动作的映射表随 C14 决策（API_SPEC §4）一并冻结；本文码表为决策输入而非已冻结契约。

## 3. 关联阅读

- 命令契约全文：`docs/08_development/API_SPEC.md`
- 错误链路缺陷上下文：`docs/06_review/UX_REVIEW.md`、`docs/product-review/USER_FLOW_REVIEW.md` §三
- 权限相关错误（FS_PATH_REJECTED 等）：`docs/08_development/PERMISSION.md`
