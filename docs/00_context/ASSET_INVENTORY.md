# QueryLab 资产清单（ASSET_INVENTORY）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 盘点方式：文件系统实测（find/ls，2026-09-03）+ 文档核对。本项目无独立设计资产目录（图标/字体/插画等均来自 Tauri/Vite 模板默认），设计规范以文档形态存在于 `docs/07_design_system/`。

---

## 1. 应用图标（源：`src-tauri/icons/`，Tauri 模板默认图标）

| 文件 | 大小 | 说明 |
|------|------|------|
| icon.icns | 1497 B | macOS 图标集 |
| icon.ico | 1497 B | Windows 图标 |
| icon.png | 1809 B | 通用 PNG |
| 32x32.png / 128x128.png / 128x128@2x.png | 1497 B | 各尺寸 PNG |

- 结论：均为 Vite/Tauri 脚手架默认图标（与 `public/vite.svg` 同为模板残留，`docs/04_architecture/SYSTEM_ARCH.md` §4 成品化清理项）。品牌图标未设计【未知——未见任何设计源文件】。

## 2. 前端静态资源

| 路径 | 说明 |
|------|------|
| `public/vite.svg` | Vite 模板 logo（模板残留，评审建议移除） |
| `src/assets/svelte.svg` | Svelte 模板 logo（同上） |

## 3. 设计体系资产（文档形态）

迁移后位于 `docs/07_design_system/`（原 `design-system/` 目录，2026-09-03 迁移）：

| 文件 | 内容 | 迁移来源 |
|------|------|----------|
| TOKEN.md | 设计令牌（颜色/字号/间距等） | design-system/tokens.md |
| COMPONENT.md | 组件规范 | design-system/components.md |
| PATTERN.md | 交互模式 | design-system/patterns.md |
| ASSETS.md | 设计资产清单（原有文档） | design-system/assets.md |
| GUIDELINES.md | 设计准则 | design-system/guidelines.md |

## 4. 原型资产（原位不动）

| 路径 | 内容 |
|------|------|
| `prototype/v0-old/app-prototype.html` | 旧版 HTML 原型（174,123 B，2026-09-02 23:54） |
| `prototype/v1-new/app-prototype.html` | V1 新版 HTML 原型（173,055 B，2026-09-03 01:13） |

## 5. 文档资产

### 5.1 本编号体系（SOP v2.0，2026-09-03 建立）

`docs/00_context` 至 `docs/09_test` 十个编号目录 + `docs/DOCUMENT_INDEX.md`（索引）。迁移映射与新建清单见 `docs/DOCUMENT_INDEX.md` 附录。

### 5.2 评审六件套（原位不动，`docs/product-review/`）

DATA_STORAGE_REVIEW.md、INFORMATION_ARCHITECTURE_REVIEW.md、PERMISSION_REVIEW.md、PRODUCT_LOGIC_REVIEW.md（总报告）、STATE_REVIEW.md、USER_FLOW_REVIEW.md（均 2026-09-03，47 条编号问题：A1/B21/C17/D8）。

### 5.3 项目自有文档（不属编号体系，原位不动）

| 路径 | 说明 |
|------|------|
| `README.md` | 项目自述（技术栈/结构/命令，权威工作区约定） |
| `DEVELOPMENT.md` | 开发计划与进度（2026-01-08 基线） |
| `docs/PRD.md` | 项目早期 PRD（与 `docs/02_product/PRD.md` v2.0 不同文件，路径不冲突） |
| `docs/README.md`、`docs/index.md`、`docs/.vitepress/`、`docs/public/`、`docs/package.json` | docs 站点（VitePress）与部署配置 |
| `docs/RELEASE_CHECKLIST.md` | 发布检查清单（产品边界口径来源） |
| `docs/RELEASE_VERIFICATION_2026-04-22.md` | 发布验证记录 |
| `docs/CORE_LOGIC_REVIEW_2026-04-22.md` | 核心逻辑评审（历史） |
| `docs/API_SQL_补全策略.md` | API/SQL 补全策略（历史） |
| `.github/workflows/deploy-docs.yml` | docs 站点部署工作流 |

## 6. 代码资产（盘点口径，不展开）

| 类别 | 清单 |
|------|------|
| 前端组件（`src/components/`） | ConnectionManager、SchemaTree、SqlEditor、ResultsPanel、DataGrid、TableDesigner、DataSync、DatabaseBackup、BatchProgressPanel（未接线）、NotificationCenter（含于 lib） |
| 前端测试 | ConnectionManager.test.js、DataGrid.test.js、DatabaseBackup.test.js、`src/test/setup.js` |
| 前端 lib | notifications.js（Toast/确认 store）；Counter.svelte（模板残留死代码） |
| Rust 模块（`src-tauri/src/`） | main.rs、commands/（5 文件）、db/（2）、storage/（1+mod）、security/（1）、core/（2，占位）、util/（1，占位） |
| Rust 单测 | storage/connections.rs 2 个（密码不落盘锁定 + 往返）、commands/query.rs 1 个（grep `#[test]` 计数） |
| 旧副本 | `src-ui/` 整目录（含独立 .git；C9 待决策删除，本会话未动） |

## 7. 运行期数据资产（非仓库文件）

| 数据 | 位置 | 说明 |
|------|------|------|
| 连接配置 | `dirs::config_dir()/querylab/connections.json` | 不含密码（save_all 前清空，有单测锁定） |
| 连接密码 | 系统钥匙串，service=`com.i2kai.querylab.connection` | 按连接 id 分条 |
| SQL 历史 | localStorage（会话/本地双模式） | 上限 100，敏感词过滤 |
| 查询结果 | 仅内存不落盘 | DATA_STORAGE_REVIEW DS-07 正面项 |
