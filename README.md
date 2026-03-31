# QueryLab UI Workspace

> 目标仓库名：`lys-query-lab`
> 当前状态：本地开发中的 Tauri 桌面客户端工作区

---

## 当前技术栈

这个工作区当前实际使用：

- Svelte 5
- Vite
- Tauri 2
- Rust
- CodeMirror 6
- `mysql_async`

它不是默认模板项目，文档和实现都应以当前工作区现实为准。

---

## 工作区结构

```text
src-ui/
├── src/
│   ├── components/
│   │   ├── ConnectionManager.svelte
│   │   ├── SchemaTree.svelte
│   │   ├── SqlEditor.svelte
│   │   ├── ResultsPanel.svelte
│   │   ├── DataGrid.svelte
│   │   └── ...
│   ├── App.svelte
│   └── main.js
└── src-tauri/
    ├── Cargo.toml
    └── src/
```

---

## 当前目标

这个工作区当前的重点是构建数据库工作台的最小可用骨架：

- 连接管理
- Schema 浏览
- SQL 编辑
- 结果面板
- 表数据相关基础能力

---

## 开发命令

### 前端开发

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

## 相关文档

- [docs/README.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/docs/README.md)
- [docs/PRD.md](/Users/luoyaosheng/Desktop/project/Open/QueryLab/docs/PRD.md)
