# QueryLab 用户旅程（USER_FLOW）

> 编制日期：2026-09-03（SOP v2.0 编号文档体系迁移时新建）
> 事实来源：`docs/02_product/PRD.md` §3/§7/§8（场景/流程/验收）、`docs/product-review/USER_FLOW_REVIEW.md`（F1-F8 路径对比与异常分支，2026-09-03 评审实测口径）、`src/App.svelte`、`src/components/*.svelte`。评审已标注：无 MySQL 实例，连接类行为以源码逻辑推断（涉及处注明）。

---

## 旅程一：首次连接建立（对应评审 F1 + PRD 场景 1）

```mermaid
flowchart TD
    A[用户首次打开应用] --> B[侧栏连接区点 ＋]
    B --> C[填写 5 字段表单<br/>名称/主机/端口/用户/密码]
    C --> D{填写完整?}
    D -- 否 --> E[必填校验阻止提交] --> C
    D -- 是 --> F[可选：点「测试连接」conn_test<br/>表单携带手输密码]
    F -- 失败/超时 5s --> G[红色结果块<br/>连接失败: err / 连接超时（5秒）]
    G --> H[修正参数] --> F
    F -- 成功 --> I[绿色结果块<br/>延迟/版本/用户]
    I --> J[保存 conn_upsert<br/>密码写入系统钥匙串]
    J --> K[conn_list 刷新列表]
    K --> L[点击连接项]
    L --> M[重置工作区+加载数据库列表<br/>状态栏 Connected to X]
    M --> N{密码账户?}
    N -- 是 --> O[⚠️ 已知缺陷 PL-01<br/>加载库即失败 Access denied<br/>测试成功→使用失败]
    N -- 否（无密码账户）--> P[进入主流程：浏览/查询]
```

- 路径形态评审结论（USER_FLOW_REVIEW F1）：路径**符合**最快合理路径（2-3 步）；但带密码账户在「点连接项」后加载库即失败（UF-01，代码路径核实未实测）。
- 异常出口：测试失败红块（无直达编辑，P4 FL-01 已列 B）；保存失败 toast。

## 旅程二：日常查询闭环（对应评审 F2/F3 + PRD 场景 2）

```mermaid
flowchart TD
    A[已选中连接] --> B{两条子路径}
    B -- 路径A: 点表直看 --> C[Schema 树展开库 → 点表<br/>自动切数据网格 COUNT+LIMIT 50]
    B -- 路径B: 手写 SQL --> D[SQL 查询视图默认<br/>CodeMirror 编辑器写 SQL]
    D --> E[Ctrl+Enter 执行<br/>选中片段优先]
    E --> F{已选连接?}
    F -- 否 --> G[toast 请先选择连接<br/>状态栏 No connection]
    F -- 是 --> H[query_execute<br/>maxRows=1000]
    H -- 执行错误 --> I[结果面板错误态 全文展示<br/>状态栏 Query failed]
    H -- 成功 --> J[结果面板五态渲染<br/>多 Tab 每语句一个结果集]
    J --> K[导出 CSV/JSON]
    K --> L[save 对话框<br/>⚠️ 插件未注册必然 reject PL-02]
    L --> M[静默降级浏览器下载<br/>落点不可控 UF-03]
    J --> N[历史回溯 Ctrl+H<br/>敏感词过滤 仅保留当前会话]
    C --> O[切回「SQL 查询」视图<br/>自动回填 SELECT * FROM db.table LIMIT 1000<br/>⚠️ 覆盖手写 SQL P4 FL-02]
```

- 评审结论（F2）：路径骨架**符合**；两处断点：UF-01 凭据、自动回填覆盖手写 SQL。
- 评审结论（F3 导出）：**不符合**——主路径（原生对话框）不存在，实际永远走降级；文件名在非单表查询时为 `_export.csv`（PF-04）。
- 异常出口缺口：SQL 执行无超时无取消，「执行中」态可能无终点（UF-02）。

## 旅程三：数据修正（对应评审 F4 + PRD 场景 3）

```mermaid
flowchart TD
    A[Schema 树点表进入网格] --> B[加载表结构 meta_get_table_schema]
    B --> C{单列 PRIMARY 且在列中?}
    C -- 否 --> D[只读降级横幅<br/>仅浏览/筛选/导出/插入 F037]
    C -- 是 --> E[开放编辑/删除]
    E --> F[筛选定位<br/>列/全列 LIKE + 转义 回第 1 页]
    F --> G[双击单元格改值]
    G --> H[Enter 保存 UPDATE…LIMIT 1<br/>Esc 取消 · ⚠️ Tab 键丢值 PF-09]
    H --> I[刷新数据 + 横幅反馈 更新成功]
    E --> J[+ 新增行：逐列录入 Tab 跳列<br/>排除自增列 INSERT]
    J --> I
    E --> K[勾选行 → 删除确认 danger → 逐行 DELETE]
    K -- 确认 --> I
    K -- 取消 --> L[不动]
    D --> M[仍可 + 新增 INSERT]
    M --> I
```

- 评审结论（F4）：路径**符合**最快合理路径（3 步）；无单列主键表的只读降级有横幅提示（正面）。

## 旅程四：表结构演进（对应评审 F5 + PRD 场景 4）

```mermaid
flowchart TD
    subgraph 新建
        A[展开目标库<br/>⚠️ 入口需先展开 UF-08] --> B[+ 新建表]
        B --> C[设计器新建模式<br/>📝 新建表: db 指示条]
    end
    subgraph 编辑
        D[⚠️ 无右键入口 IA-01<br/>须 点表进网格 → 视图切换器「📋 设计表」两跳] --> E[设计器编辑模式<br/>快照 originalColumns]
    end
    C & E --> F[增删列/改属性/改表选项<br/>联动规则：主键→NOT NULL·自增→主键]
    F --> G[头部出现「有未保存的更改」F042]
    G --> H{用户点关闭?}
    H -- 是 --> I[⚠️ 直接丢弃无确认 PL-03/UF-05]
    H -- 保存 --> J{结构校验 F041}
    J -- 失败 --> K[错误横幅阻止提交]
    J -- 通过 --> L[新建: meta_create_table<br/>编辑: diff 生成 ALTER 序列执行]
    L --> M[toast 后端消息 + Schema 自动刷新<br/>新建成功后切网格]
```

- 评审结论（F5）：新建**符合**；编辑路径多一跳且意图错位（IA-01）；关闭无异常出口（PL-03）。

## 旅程五：备份与结构对齐（对应评审 F6/F7 + PRD 场景 5/6）

```mermaid
flowchart TD
    subgraph 结构对比
        A[选源库/目标库 模式锁定仅结构] --> B{源=目标?}
        B -- 是 --> C[报错 源和目标不能相同]
        B -- 否 --> D[开始比较]
        D --> E{有差异?}
        E -- 否 --> F[结构相同提示]
        E -- 是 --> G[差异列表默认全选<br/>统计: 新增/删除/修改表·±~列·索引]
        G --> H{出仓方式}
        H -- 复制/导出 SQL --> I[剪贴板 / 下载 .sql]
        H -- 执行 --> J[danger 确认 不可撤销]
        J -- 确认 --> K[逐句执行 + toast 完成 + 刷新]
        J -- 取消 --> G
    end
    subgraph 备份导出
        L[选库 → 加载表 → 默认全选] --> M[取消全选/勾选]
        M --> N[开始导出]
        N --> O[save 对话框<br/>⚠️ 必然失败 PL-02 → 降级下载]
        O -- 取消 --> P[已取消导出 正面反馈]
    end
    subgraph 备份导入
        Q[选目标库 + 选 .sql 文件 open 对话框] --> R[⚠️ open 必然失败且无降级<br/>console.error 静默 UF-06<br/>importFile 恒空 → 开始导入按钮永久禁用]
    end
```

- 评审结论（F6 结构对比）：路径**符合**，危险确认与完成刷新齐备（正面）；「新增表仅注释」为承诺边界。
- 评审结论（F7 备份）：**不符合**——导入流程在打包态第一步就走不通（UF-06）；导出取消路径反馈「已取消导出」是正面细节。

## 旅程六：批量执行含事务（对应评审 F8）

```mermaid
flowchart TD
    A[SQL 编辑器开启批量模式开关] --> B{开启事务?}
    B -- 是 --> C[语句被 START TRANSACTION/COMMIT 包裹<br/>App.svelte L185-189]
    B -- 否 --> D[整段提交]
    C & D --> E[运行]
    E -- 成功 --> F[状态栏一行<br/>Batch completed: N statements, Xms]
    E -- 失败 --> G[Batch execution failed<br/>⚠️ 无显式 ROLLBACK 靠连接关闭隐式回滚 UF-04<br/>用户不知已执行部分是否回滚]
    F --> H[⚠️ 逐条进度面板未接线 PF-03<br/>无 per-statement 反馈 ST-04]
```

- 评审结论（F8）：**部分符合**——执行可用，反馈层缺位；事务失败无 ROLLBACK 感知（mysql_async 连接 drop 回滚语义【未知——需实测】）。

## 附：旅程健康度总表（源 USER_FLOW_REVIEW §二）

| 旅程 | 五要素完整性 | 主要缺口 |
|------|--------------|----------|
| F1 连接 | 触发/前置/步骤/结果清晰 | 带密码账户「测试成功→使用失败」无解释（UF-01） |
| F2 主查询 | 清晰 | 执行无超时无取消（UF-02）；自动回填覆盖手写 SQL |
| F3 导出 | 设计上清晰 | 主路径损坏（UF-03=PL-02）；降级落点不可控 |
| F4 网格修正 | 清晰 | Tab 丢值（PF-09） |
| F5 结构演进 | 清晰 | 关闭丢弃未保存无确认（UF-05=PL-03）；编辑两跳（IA-01） |
| F6 结构对比 | 清晰 | 基本完备（正面） |
| F7 备份 | 设计上清晰 | 导入文件选择不可用（UF-06=PL-02） |
| F8 批量 | 清晰 | 无逐条进度（PF-03）；事务失败无 ROLLBACK 感知（UF-04） |
