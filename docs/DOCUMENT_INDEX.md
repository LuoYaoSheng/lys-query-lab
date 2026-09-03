# QueryLab 文档索引（DOCUMENT_INDEX）

> 建立日期：2026-09-03，依据《旧 App AI 重构 SOP v2.0》编号文档体系（00_context ～ 09_test + prototype/{v0-old,v1-new}）。
> 本索引覆盖：编号体系全部文档（迁移 17 项 + 新建 16 项）、评审六件套（原位）、原型（原位）。项目自有文档（docs/PRD.md、README.md、RELEASE_* 等）不属编号体系，仅在 §4 登记。

---

## 1. 编号目录一览

| 目录 | 主题 | 文档数 |
|------|------|--------|
| 00_context | 项目上下文/技术栈/资产/依赖 | 4（全部新建） |
| 01_reverse | 逆向分析 | 1（迁移） |
| 02_product | 产品需求与能力 | 4（2 迁移 + 2 新建） |
| 03_flow | 用户/页面/业务流程 | 3（全部新建） |
| 04_architecture | 系统与模块架构 | 3（迁移）+ DATA_FLOW（新建） |
| 05_sequence | 时序图集 | 1（新建） |
| 06_review | 评审摘编 | 1（迁移）+ 2 新建 |
| 07_design_system | 设计体系 | 5（迁移） |
| 08_development | 开发规格 | 2（迁移）+ 2 新建 |
| 09_test | 测试与验收 | 3（迁移） |
| prototype/ | HTML 原型 | 2（原位不动） |

## 2. 编号体系文档清单

### 00_context（新建，2026-09-03）

| 文档 | 内容 | 主要来源 |
|------|------|----------|
| [00_context/PROJECT_CONTEXT.md](00_context/PROJECT_CONTEXT.md) | 项目定位/仓库布局（src 与 src-ui 双前端现状）/构建运行/关键入口 | README.md、package.json、tauri.conf.json、main.rs |
| [00_context/TECH_STACK.md](00_context/TECH_STACK.md) | 技术栈清单（含 plugin-dialog 前后端断裂标注） | package.json、Cargo.toml、PERMISSION_REVIEW |
| [00_context/ASSET_INVENTORY.md](00_context/ASSET_INVENTORY.md) | 资产清单（图标/静态资源/设计文档/原型/测试/运行期数据） | 文件系统实测 |
| [00_context/DEPENDENCY_LIST.md](00_context/DEPENDENCY_LIST.md) | 依赖+用途+死依赖标注（含 Rust 侧缺失插件） | package.json、Cargo.toml、全局 grep |

### 01_reverse（迁移自 docs/reverse-analysis.md）

| 文档 | 内容 |
|------|------|
| [01_reverse/REVERSE_ANALYSIS.md](01_reverse/REVERSE_ANALYSIS.md) | 旧项目逆向分析报告（PAGE001-012 / F001-057 编号主键，2026-09-02） |

### 02_product

| 文档 | 来源 | 内容 |
|------|------|------|
| [02_product/PRD.md](02_product/PRD.md) | 迁移自 docs/product/prd.md | 产品需求文档 v2.0 |
| [02_product/PAGE_SPEC.md](02_product/PAGE_SPEC.md) | 迁移自 docs/product/page-spec.md | 页面交互规格（12 页 × 11 维度） |
| [02_product/PRODUCT_MODEL.md](02_product/PRODUCT_MODEL.md) | 新建 | 产品定位/用户角色/使用场景/核心价值 |
| [02_product/FEATURE_MAP.md](02_product/FEATURE_MAP.md) | 新建 | 产品能力树（F001-F057 → 六分组） |

### 03_flow（新建）

| 文档 | 内容 |
|------|------|
| [03_flow/USER_FLOW.md](03_flow/USER_FLOW.md) | 用户旅程 6 组（Mermaid），含旅程健康度总表 |
| [03_flow/PAGE_FLOW.md](03_flow/PAGE_FLOW.md) | 页面跳转关系图 + 出入口表 |
| [03_flow/BUSINESS_FLOW.md](03_flow/BUSINESS_FLOW.md) | 正常/异常/边界流程（含 PL-01 凭据链路断裂标注） |

### 04_architecture

| 文档 | 来源 | 内容 |
|------|------|------|
| [04_architecture/SYSTEM_ARCH.md](04_architecture/SYSTEM_ARCH.md) | 迁移自 docs/architecture/tech-architecture.md | V1 技术架构与关键数据流 |
| [04_architecture/MODULE_ARCH.md](04_architecture/MODULE_ARCH.md) | 迁移自 docs/architecture/module-split.md | V1 模块拆分建议（11 模块） |
| [04_architecture/STATE_MACHINE.md](04_architecture/STATE_MACHINE.md) | 迁移自 docs/architecture/state-management.md | V1 状态管理 |
| [04_architecture/DATA_FLOW.md](04_architecture/DATA_FLOW.md) | 新建 | 数据流动（连接配置→keychain、命令→Rust→DB；密码 skip_serializing 链路事实） |

### 05_sequence（新建）

| 文档 | 内容 |
|------|------|
| [05_sequence/SEQUENCE_DIAGRAMS.md](05_sequence/SEQUENCE_DIAGRAMS.md) | 5 张 sequenceDiagram：新建连接+测试 / 执行查询 / 浏览表结构 / 备份导出 / 表设计器保存 |

### 06_review

| 文档 | 来源 | 内容 |
|------|------|------|
| [06_review/PRODUCT_REVIEW.md](06_review/PRODUCT_REVIEW.md) | 迁移自 docs/review/product-review.md | P4 产品体验审查（PF/PP/FL，A3/B13/C14/D8） |
| [06_review/UX_REVIEW.md](06_review/UX_REVIEW.md) | 新建 | 综合 PRODUCT_REVIEW 与 USER_FLOW_REVIEW |
| [06_review/IA_REVIEW.md](06_review/IA_REVIEW.md) | 新建 | 综合 INFORMATION_ARCHITECTURE_REVIEW |

### 07_design_system（迁移自 design-system/）

| 文档 | 迁移来源 |
|------|----------|
| [07_design_system/TOKEN.md](07_design_system/TOKEN.md) | design-system/tokens.md |
| [07_design_system/COMPONENT.md](07_design_system/COMPONENT.md) | design-system/components.md |
| [07_design_system/PATTERN.md](07_design_system/PATTERN.md) | design-system/patterns.md |
| [07_design_system/ASSETS.md](07_design_system/ASSETS.md) | design-system/assets.md |
| [07_design_system/GUIDELINES.md](07_design_system/GUIDELINES.md) | design-system/guidelines.md |

### 08_development

| 文档 | 来源 | 内容 |
|------|------|------|
| [08_development/DATA_MODEL.md](08_development/DATA_MODEL.md) | 迁移自 docs/architecture/data-model.md | V1 数据模型 |
| [08_development/API_SPEC.md](08_development/API_SPEC.md) | 迁移自 docs/architecture/api-design.md | V1 API 设计（16 命令契约 + 错误码草案） |
| [08_development/ERROR_CODE.md](08_development/ERROR_CODE.md) | 新建 | 错误处理现状盘点 + 错误码规范建议 |
| [08_development/PERMISSION.md](08_development/PERMISSION.md) | 新建 | 权限规格（keychain/fs/dialog 现状 + 目标态） |

### 09_test（迁移）

| 文档 | 迁移来源 | 内容 |
|------|----------|------|
| [09_test/COVERAGE_CHECKLIST.md](09_test/COVERAGE_CHECKLIST.md) | docs/product/html-coverage-checklist.md | HTML 覆盖检查清单 |
| [09_test/HTML_V0_ACCEPTANCE.md](09_test/HTML_V0_ACCEPTANCE.md) | docs/product/html-acceptance-report.md | 旧版原型验收报告 |
| [09_test/V1_ACCEPTANCE.md](09_test/V1_ACCEPTANCE.md) | docs/review/v1-acceptance.md | V1 新版原型验收报告 |

### prototype/（原位不动）

| 路径 | 内容 |
|------|------|
| prototype/v0-old/app-prototype.html | 旧版 HTML 原型 |
| prototype/v1-new/app-prototype.html | V1 新版 HTML 原型 |

## 3. 评审六件套（原位保留，docs/product-review/）

| 文档 | 内容 |
|------|------|
| [product-review/PRODUCT_LOGIC_REVIEW.md](product-review/PRODUCT_LOGIC_REVIEW.md) | 总报告（PL-01…08；47 条问题汇总；PL-01 凭据断裂/PL-02 对话框未注册） |
| [product-review/USER_FLOW_REVIEW.md](product-review/USER_FLOW_REVIEW.md) | 用户流程评审（UF-01…10） |
| [product-review/INFORMATION_ARCHITECTURE_REVIEW.md](product-review/INFORMATION_ARCHITECTURE_REVIEW.md) | 信息架构评审（IA-01…07） |
| [product-review/DATA_STORAGE_REVIEW.md](product-review/DATA_STORAGE_REVIEW.md) | 数据存储评审（DS-01…09） |
| [product-review/STATE_REVIEW.md](product-review/STATE_REVIEW.md) | 状态模型评审（ST-01…06） |
| [product-review/PERMISSION_REVIEW.md](product-review/PERMISSION_REVIEW.md) | 权限评审（PM-01…07） |

## 4. 项目自有文档（不属编号体系，原位不动）

- 根目录：`README.md`（权威工作区约定）、`DEVELOPMENT.md`。
- `docs/` 下：`PRD.md`（早期 PRD，与 02_product/PRD.md v2.0 为不同文件）、`README.md`、`index.md`、`RELEASE_CHECKLIST.md`、`RELEASE_VERIFICATION_2026-04-22.md`、`CORE_LOGIC_REVIEW_2026-04-22.md`、`API_SQL_补全策略.md`、`.vitepress/`（docs 站点）、`public/`、`package.json`、`node_modules/`。
- CI：`.github/workflows/deploy-docs.yml`。

## 5. 阅读路径建议

- 新人入门：PROJECT_CONTEXT → TECH_STACK → PRD → PAGE_SPEC。
- 重构执行：REVERSE_ANALYSIS → PRODUCT_LOGIC_REVIEW（PL-01/02/03）→ SYSTEM_ARCH/MODULE_ARCH → API_SPEC。
- 评审追溯：UX_REVIEW / IA_REVIEW → 对应 product-review/ 分报告。

## 附录：2026-09-03 迁移记录

- 移动 17 个文件（18 项映射中第 18 项「prototype/ 原位不动」无操作）：docs/reverse-analysis.md、docs/product/{prd,page-spec,html-coverage-checklist,html-acceptance-report}.md、docs/review/{product-review,v1-acceptance}.md、docs/architecture/{tech-architecture,module-split,state-management,data-model,api-design}.md、design-system/{tokens,components,patterns,assets,guidelines}.md。
- 移动后源目录 docs/product、docs/review、docs/architecture、design-system 已空并删除；docs/product-review/ 与 prototype/ 未动。
- 新建 16 个文件（00_context×4、02_product×2、03_flow×3、04_architecture/DATA_FLOW、05_sequence×1、06_review×2、08_development×2、DOCUMENT_INDEX×1）。
- 交叉引用：编号体系 .md 与原型 HTML 内旧路径已替换为新路径（详见各文件）。
