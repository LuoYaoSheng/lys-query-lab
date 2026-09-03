# QueryLab Design System — 设计令牌（tokens）

> 版本：v1.0（2026-09-02）
> 提取方式：全部取自旧项目真实源码（`src/App.svelte`、`src/components/*.svelte`），每项注明来源文件；SQL 语法色板取自实际安装的 `@codemirror/theme-one-dark` 包（`src-ui/node_modules/@codemirror/theme-one-dark/dist/index.js`，根 `package.json` 依赖 `@codemirror/theme-one-dark ^6.x`，node_modules 仅 src-ui 有安装副本）。
> 命名规范：`--ql-<类别>-<名称>`；V1 原型（`prototype/v1-new/app-prototype.html`）与后续开发实现必须以本文件 CSS 变量为唯一定义源。
> 主题定位：深色单主题（VS Code 风格），来源 `src/App.svelte` `.app-container { background: #1e1e1e }`。

---

## 1. Color 颜色

### 1.1 背景与面板（Background / Surface）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-bg-app` | `#1e1e1e` | 应用最底层背景 | App.svelte `.app-container`（出现 22 次，全组件） |
| `--ql-bg-panel` | `#252526` | 侧栏 / 弹窗 / 卡片 / 下拉面板 | App.svelte `.sidebar`；NotificationCenter `.confirm-dialog`；SqlEditor `.cm-search` |
| `--ql-bg-raised` | `#2d2d2d` | 顶栏 / 分区头 / 次级面板 | App.svelte `.topbar`、`.view-switcher`（出现 31 次） |
| `--ql-bg-surface-3` | `#2a2d3e` | 数据表格列头 | DataGrid.svelte `.datagrid-table th` |
| `--ql-bg-hover` | `#3e3e3e` | 悬停态 / 次级按钮底 | 全组件（出现 85 次，最高频） |
| `--ql-bg-deep` | `#3c3c3c` / `#424242` | 输入框 / 嵌套表面 | ConnectionManager、TableDesigner（出现 15/6 次） |

### 1.2 边框（Border）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-border-default` | `#3e3e3e` | 一级分割线（同 hover 色复用） | App.svelte 顶栏/状态栏分隔线等 |
| `--ql-border-card` | `#333` | 卡片边框 | App.svelte shell-section |
| `--ql-border-strong` | `#4e4e4e` | 输入框/强分隔 | 各组件（出现 31 次） |

### 1.3 文本（Text）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-text-primary` | `#d4d4d4` | 正文主文本 | App.svelte（出现 57 次） |
| `--ql-text-secondary` | `#888` | 次要说明 / NULL 值 / 时间戳 | 全组件（出现 46 次） |
| `--ql-text-heading` | `#f4f4f4` | 弹窗标题 / Toast 文本 | NotificationCenter `.confirm-header h3`、`.toast` |
| `--ql-text-on-primary` | `#ffffff` | 主色按钮上的文字 | App.svelte 视图按钮 active |

### 1.4 品牌与交互（Brand / Action）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-primary` | `#007acc` | 品牌主色 / 主按钮 / 状态栏底 / 选中连接条 | App.svelte（出现 34 次） |
| `--ql-primary-hover` | `#005a9e` | 主色 hover / 深边框 | App.svelte 视图按钮 active border |
| `--ql-primary-pressed` | `#006cbd` | 主色按下 | SchemaTree 等 |
| `--ql-focus-ring` | `#007acc` | 焦点边框 | ResultsPanel L528、ConnectionManager 表单 |

### 1.5 语义色（Semantic）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-success` | `#4ec9b0` | 成功文本 / 数字值 / 只读提示 | 出现 15 次（ResultsPanel、DataGrid） |
| `--ql-success-bg` | `#1e3a1e` | 成功横幅底 | DataGrid `.message-banner.success` |
| `--ql-success-strong` | `#2da042` / `#238736` | 成功按钮 / 运行按钮 | SqlEditor btn-run |
| `--ql-success-toast-bg` | `#1f4b2a`（边框 `#2f7a45`） | 成功 Toast | NotificationCenter `.toast.success` |
| `--ql-danger` | `#f48771` | 错误文本（浅红） | 出现 30 次 |
| `--ql-danger-strong` | `#d73a49` / `#c73b3b` | 危险确认按钮 / 错误强调 | NotificationCenter `.btn-confirm.danger` |
| `--ql-danger-bg` | `#3c1f1e` / `#4b2424`（边框 `#8a3a3a`） | 错误块 / 错误 Toast | DataSync、NotificationCenter |
| `--ql-info-bg` | `#1f364b`（边框 `#2f5f8a`） | 信息 Toast | NotificationCenter `.toast.info` |
| `--ql-warning` | `#dcdcaa` | 视图名 / 差异徽标「有差异」/ 表名补全图标 | SchemaTree、DataSync、SqlEditor（出现 8 次） |

### 1.6 视图主色（View Identity，视图切换器 active 态）

| Token | 值 | 视图 | 来源 |
|-------|-----|------|------|
| `--ql-view-design` | `#9b46c8`（边框 `#7a35a0`） | 📋 设计表 | App.svelte `.design-btn.active` |
| `--ql-view-sync` | `#0e639c`（边框 `#0a4a74`） | 🔍 结构对比 | App.svelte `.sync-btn.active` |
| `--ql-view-backup` | `#c84e4e`（边框 `#a03535`） | 💾 备份还原 | App.svelte `.backup-btn.active` |
| `--ql-view-query` | `--ql-primary` | SQL 查询（默认） | App.svelte |

### 1.7 数据网格行态（Row States）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-row-hover` | `#2a2d2e` | 行悬停 | DataGrid `tr:hover td` |
| `--ql-row-selected` | `#1a3a2e` | 勾选行 | DataGrid `tr.selected-row td` |
| `--ql-row-editing` | `#1a2d1e` | 编辑行 | DataGrid `tr.editing-row td` |
| `--ql-row-new` | `#1e2d3e` | 新增行 | DataGrid `tr.new-row td` |
| `--ql-indicator-creating` | `#2a3a2e` 底 / `#4ec9b0` 字 | 「新建表: {db}」指示条 | App.svelte `.creating-indicator` |

### 1.8 主键列（Primary Key）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-pk-text` | `#c586c0` | 主键列头 🔑 / 主键单元格文本 | ResultsPanel L686、SqlEditor 补全图标 |
| `--ql-pk-bg` | `#252835` | 主键列底色 | page-spec 一.8（ResultsPanel/DataGrid 主键列样式） |

### 1.9 结果值类型渲染（Value Rendering）

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-value-null` | `#888` + italic | NULL 值 | ResultsPanel L681（font-style: italic） |
| `--ql-value-number` | `#b5cea8` | 数字值 | ResultsPanel L673 |
| `--ql-value-string` | `#ce9178` | 字符串值 | ResultsPanel L677 |
| `--ql-value-bytes` | `#888` | 二进制 `[N bytes]` | ResultsPanel L681 |

### 1.10 SQL 关键字高亮色板（CodeMirror oneDark 实测值）

> 来源：`src-ui/node_modules/@codemirror/theme-one-dark/dist/index.js`（SqlEditor.svelte 引入 `@codemirror/theme-one-dark`，oneDark 主题）。**B5 修复后**按 `driver` 字段映射 MySQL 方言，色板不变。

| Token | 值 | 语法类 |
|-------|-----|--------|
| `--ql-code-bg` | `#282c34` | 编辑器背景 |
| `--ql-code-fg` | `#abb2bf` | 默认前景 |
| `--ql-code-caret` | `#528bff` | 光标 |
| `--ql-code-selection` | `#3E4451` | 选区 |
| `--ql-code-keyword` | `#c678dd` | 关键字（SELECT/FROM/WHERE） |
| `--ql-code-string` | `#98c379` | 字符串字面量 |
| `--ql-code-number` | `#d19a66` | 数字/布尔常量 |
| `--ql-code-comment` | `#7d8799`（italic） | 注释 |
| `--ql-code-variable` | `#e06c75` | 变量/标签 |
| `--ql-code-typename` | `#e5c07b` | 类型名 |
| `--ql-code-operator` | `#56b6c2` | 运算符 |
| `--ql-code-meta` | `#61afef` | 元信息/链接 |

补全图标色（SqlEditor.svelte L744-756 自定义）：关键字 `#c586c0`、类型 `#4ec9b0`、表名 `#dcdcaa`。
历史/SQL 预览等业务内代码文本沿用 monospace 字体 + `--ql-code-keyword` 等色（DataSync SQL 预览、BatchProgressPanel 语句预览同源）。

---

## 2. Typography 字体

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-font-family` | `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif` | 全局 UI | App.svelte `:global(body)` L548 |
| `--ql-font-family-mono` | `'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace` | SQL 编辑器 / SQL 预览 / 语句列表 | SqlEditor L682、DataSync L1215、BatchProgressPanel L323 |

| Token | 值 | 场景 | 来源 |
|-------|-----|------|------|
| `--ql-font-size-xs` | `11px` | eyebrow / 图标字 / 行号 | App.svelte L630、L678 |
| `--ql-font-size-sm` | `12px` | 视图按钮 / 状态栏 / 表头副行 / 分页条 | App.svelte（出现 97 次，最高频档位） |
| `--ql-font-size-body` | `13px` | 正文按钮 / 表格内容 / Toast | App.svelte（33 次） |
| `--ql-font-size-md` | `14px` | 卡片标题 / 输入框 | App.svelte L661 |
| `--ql-font-size-lg` | `16px` | 弹窗标题 / logo | App.svelte L571、L626 |
| `--ql-font-size-xl` | `20px` | shell 面板标题 / 关闭 × | App.svelte L626、L640 |

字重：默认 400；按钮/列头 600（各组件 button/th）；无 >700 用法（如实记录）。

---

## 3. Spacing 间距

> 旧项目为逐处硬编码，以下为实测高频档位归一（4px 基网格）。

| Token | 值 | 高频用途 | 依据 |
|-------|-----|----------|------|
| `--ql-space-1` | `4px` | 复选框/图标内距、微调 | 全组件（radius/小间距 4px 出现 84 次） |
| `--ql-space-2` | `6px` / `8px` | 按钮内距 6px 12px；区块间距 8px | App.svelte L586；8px 出现 60 次 |
| `--ql-space-3` | `12px` | 按钮水平内距、Toast 内距 12px 14px | App.svelte L586、NotificationCenter |
| `--ql-space-4` | `14px` / `16px` | 卡片内距 16px、面板内距 16px 18px、列表项 8px 16px | App.svelte L563、NotificationCenter；16px 出现 57 次 |
| `--ql-space-5` | `18px` / `20px` | 分区间距（shell 内容 20px、confirm 16px 18px） | App.svelte L620/L645 |
| `--ql-sidebar-width` | `280px` | 侧栏固定宽 | App.svelte `.sidebar` L689 |
| `--ql-shellpanel-width` | `420px` | 右侧滑出面板宽 | App.svelte shellPanel |
| `--ql-history-width` | `320px` | 历史侧栏宽 | SqlEditor.svelte |
| `--ql-topbar-height` | `48px` | 顶栏高 | page-spec 一.1（App.svelte） |
| `--ql-statusbar-height` | `24px` | 状态栏高 | App.svelte `.app-footer` L821 |
| `--ql-window-default` | `1400×900` / 最小 `1000×600` | 桌面窗口 | tauri.conf.json |

---

## 4. Radius 圆角

| Token | 值 | 用途 | 来源 |
|-------|-----|------|------|
| `--ql-radius-sm` | `4px` | 按钮 / 视图切换按钮 / 输入框 | App.svelte L588（出现 84 次） |
| `--ql-radius-md` | `6px` | 确认对话框按钮 | NotificationCenter `.btn-cancel` |
| `--ql-radius-lg` | `8px` | Toast | NotificationCenter `.toast` |
| `--ql-radius-xl` | `10px` | 模态弹窗 / 确认对话框 | NotificationCenter `.confirm-dialog`、App.svelte L655 |

阴影（弹层统一）：Toast `0 12px 30px rgba(0,0,0,.28)`；确认框 `0 18px 40px rgba(0,0,0,.35)`；遮罩 `rgba(0,0,0,0.5)`（NotificationCenter，全项目遮罩同值）。

---

## 5. 动效（如实记录）

- Toast：success/info 3.2s、error 4.5s 自动消失（`src/lib/notifications.js`）。
- 其余交互无过渡动画定义（旧项目未写 transition；V1 原型仅补充弹窗 120ms 淡入用于评审呈现，标注为新增建议）。

---

## 6. Token 使用规则

1. V1 原型与开发实现中，颜色/字号/间距/圆角一律引用本表 CSS 变量，禁止再写裸值（旧项目 85 处 `#3e3e3e` 散写即反面教材）。
2. 语义色优先：错误用 `--ql-danger` 系，不得用品牌蓝替代。
3. 新增颜色必须先入本表并注明来源，再使用。
