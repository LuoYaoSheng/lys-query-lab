# QueryLab

> 目标仓库名：`lys-query-lab`
> 当前状态：已有真实工程骨架，README 正在按当前实现收口

---

## 这是什么

`QueryLab` 是一个本地优先的数据库工作台，当前首期聚焦：

- MySQL / MariaDB 首期支持
- 本地桌面体验
- 查询、结果查看、表数据处理
- 后续再逐步扩到更多数据库和更重的能力

它不是停留在 PRD 层的概念项目，而是已经有 Tauri + Rust + Svelte 的真实工作区。

---

## 当前技术栈

- Tauri 2
- Rust
- Svelte 5
- Vite
- CodeMirror 6
- `mysql_async`

文档和规划必须以这里的工程现实为准，不再按旧的 `React + Monaco` 口径描述。

---

## 当前仓库结构

```text
QueryLab/
├── src/                 # 当前 Svelte UI
├── src-tauri/           # 当前 Rust / Tauri 后端
├── src-ui/              # 并行工作区副本，用于 UI / Tauri 联调整理
└── docs/                # PRD、API、文档索引
```

---

## 当前目标

当前重点不是继续发散数据库范围，而是先把 MySQL 工作台主流程跑顺：

- 连接管理
- Schema 浏览
- SQL 编辑
- 结果面板
- 表数据相关基础能力

---

## 开发命令

默认以仓库根目录这套工作区为准：

```bash
npm install
npm run dev
```

### Tauri 联调

```bash
npm run tauri:dev
```

### 构建

```bash
npm run build
npm run tauri:build
```

---

## 推荐阅读

- [docs/README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/docs/README.md)
- [docs/PRD.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/docs/PRD.md)
- [src-ui/README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/src-ui/README.md)
