# QueryLab 文档中心

> 项目目标仓库名：`lys-query-lab`
> 当前本地状态：已有工程骨架，文档仍在收口中

---

## 项目简介

`QueryLab` 是一个本地优先的数据库客户端，当前聚焦：

- MySQL / MariaDB 首期支持
- 桌面端工作台体验
- 查询、结果查看、表数据编辑和导入导出

当前本地工程的实际技术栈已经明确为：

- Tauri 2
- Rust
- Svelte 5
- Vite
- CodeMirror 6
- `mysql_async`

---

## 当前文档索引

- [README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/README.md)
  - 仓库入口、技术栈和当前工作区说明
- [PRD.md](./PRD.md)
  - 产品边界、MVP 范围、用户场景
- [API_SQL_补全策略.md](./API_SQL_补全策略.md)
  - SQL 补全与元数据策略
- [src-ui/README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/src-ui/README.md)
  - `src-ui/` 并行工作区说明

---

## 当前阶段

现在的重点不是继续发散需求，而是先把下面几件事讲清楚：

1. 这个产品的首期边界是什么
2. 当前代码现实是什么
3. 哪些旧文档已经过时

因此本目录的文档会优先服务于：

- 产品边界收口
- 工程现实对齐
- 根目录工作区与 `src-ui/` 的关系说明
- 后续 README 和网站展示

---

## 当前建议阅读顺序

1. 先看 [README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/README.md)
2. 再看 [PRD.md](./PRD.md)
3. 然后看 [src-ui/README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/src-ui/README.md)
4. 最后按需看 [API_SQL_补全策略.md](./API_SQL_补全策略.md)
