<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ConnectionManager from './components/ConnectionManager.svelte';
  import SchemaTree from './components/SchemaTree.svelte';
  import SqlEditor from './components/SqlEditor.svelte';
  import ResultsPanel from './components/ResultsPanel.svelte';
  import DataGrid from './components/DataGrid.svelte';
  import TableDesigner from './components/TableDesigner.svelte';
  import DataSync from './components/DataSync.svelte';
  import DatabaseBackup from './components/DatabaseBackup.svelte';
  import NotificationCenter from './components/NotificationCenter.svelte';

  let appInfo = {
    version: '',
    platform: '',
    build: ''
  };

  let connections = [];
  let selectedConnection = null;
  let queryResult = null;
  let queryLoading = false;
  let queryError = null;
  let statusMessage = 'Ready';
  let currentTableName = ''; // 当前查询的表名
  let currentDatabase = '';  // 当前数据库
  let viewMode = 'query';   // 'query' 或 'grid' 或 'design' 或 'sync' 或 'backup'
  let isCreatingNewTable = false; // 是否正在创建新表
  let targetDatabase = ''; // 新建表的目标数据库
  let databases = []; // 数据库列表
  let shellPanel = null; // 'settings' | 'help' | 'about'
  let editableResultTableName = '';

  // 用于更新编辑器中的 SQL 和刷新 SchemaTree
  let editorComponent;
  let schemaTreeComponent;

  // 获取数据库列表
  async function loadDatabases() {
    if (!selectedConnection) return;
    try {
      const result = await invoke('meta_list_databases', { connection: selectedConnection });
      databases = result.filter(db =>
        !['information_schema', 'mysql', 'performance_schema', 'sys'].includes(db)
      );
    } catch (err) {
      console.error('Failed to load databases:', err);
    }
  }

  onMount(async () => {
    try {
      const info = await invoke('app_get_info');
      appInfo = info;
      const connList = await invoke('conn_list');
      connections = connList;
    } catch (err) {
      console.error('Failed to initialize:', err);
    }
  });

  function resetWorkspaceState() {
    queryResult = null;
    queryLoading = false;
    queryError = null;
    currentTableName = '';
    currentDatabase = '';
    viewMode = 'query';
    isCreatingNewTable = false;
    targetDatabase = '';
    editableResultTableName = '';
  }

  function extractEditableTableName(sqlText) {
    const trimmed = sqlText.trim();
    if (!trimmed) {
      return '';
    }

    const normalized = trimmed.replace(/\s+/g, ' ').replace(/;$/, '');
    if (normalized.includes(';')) {
      return '';
    }

    const match = normalized.match(/^select \* from `([^`]+)`\.`([^`]+)`(?: limit \d+)?$/i);
    if (!match) {
      return '';
    }

    return `${match[1]}.${match[2]}`;
  }

  function handleConnect(conn) {
    selectedConnection = conn;
    resetWorkspaceState();

    if (!conn) {
      databases = [];
      statusMessage = 'No connection';
      return;
    }

    statusMessage = `Connected to ${conn.name || conn.host}`;
    loadDatabases();
  }

  // 处理数据同步完成
  function handleSyncComplete() {
    // 刷新 SchemaTree
    if (schemaTreeComponent) {
      schemaTreeComponent.refreshAll();
    }
    // 重新加载数据库列表
    loadDatabases();
  }

  // 打开数据同步
  function openDataSync() {
    viewMode = 'sync';
  }

  // 打开数据库备份
  function openDatabaseBackup() {
    viewMode = 'backup';
  }

  // 处理新建表事件
  function handleCreateTable(e) {
    const { database } = e.detail;
    targetDatabase = database;
    isCreatingNewTable = true;
    viewMode = 'design';
  }

  // SQL 执行处理 - 直接调用，不通过事件
  async function executeQuery(sql) {
    if (!selectedConnection) {
      editableResultTableName = '';
      queryError = '请先选择连接';
      statusMessage = 'No connection';
      return;
    }

    queryLoading = true;
    queryError = null;
    queryResult = null;
    editableResultTableName = '';
    statusMessage = 'Executing...';

    try {
      const nextEditableTableName = extractEditableTableName(sql);
      const result = await invoke('query_execute', {
        connection: selectedConnection,
        sql,
        maxRows: 1000
      });
      queryResult = result;
      editableResultTableName = result.sets?.length === 1 ? nextEditableTableName : '';
      statusMessage = `Query completed in ${result.elapsedMs}ms`;
    } catch (err) {
      console.error('Query error:', err);
      queryError = String(err);
      editableResultTableName = '';
      statusMessage = 'Query failed';
    } finally {
      queryLoading = false;
    }
  }

  // 批量执行处理
  async function handleBatchExecute(sql, useTransaction) {
    if (!selectedConnection) {
      editableResultTableName = '';
      queryError = '请先选择连接';
      statusMessage = 'No connection';
      return;
    }

    queryLoading = true;
    queryError = null;
    queryResult = null;
    editableResultTableName = '';

    // 构建执行SQL
    let finalSql = sql;
    if (useTransaction) {
      finalSql = 'START TRANSACTION;\n' + sql + '\nCOMMIT;';
    }

    statusMessage = useTransaction ? 'Batch executing with transaction...' : 'Batch executing...';

    try {
      const startTime = Date.now();
      const result = await invoke('query_execute', {
        connection: selectedConnection,
        sql: finalSql,
        maxRows: 1000
      });
      const elapsed = Date.now() - startTime;
      queryResult = result;
      statusMessage = `Batch completed: ${result.sets.length} statements, ${elapsed}ms`;
    } catch (err) {
      console.error('Batch query error:', err);
      queryError = String(err);
      statusMessage = 'Batch execution failed';
    } finally {
      queryLoading = false;
    }
  }

  // 将 executeQuery 暴露给子组件调用
  function handleExecuteQuery(sql) {
    executeQuery(sql);
  }

  function handleSelectTable(e) {
    const { database, table } = e.detail;

    // 保存当前表信息
    currentDatabase = database;
    currentTableName = `${database}.${table}`;

    // 重置新建表状态
    isCreatingNewTable = false;
    targetDatabase = '';

    // 切换到网格模式（类似 Navicat）
    viewMode = 'grid';
  }

  // 切换视图模式
  function setViewMode(mode) {
    viewMode = mode;
    if (mode === 'query' && currentTableName) {
      // 切换回查询模式时，自动执行查询
      const parts = currentTableName.split('.');
      if (parts.length === 2) {
        const sql = `SELECT * FROM \`${parts[0]}\`.\`${parts[1]}\` LIMIT 1000;`;
        executeQuery(sql);
        if (editorComponent && editorComponent.setSql) {
          editorComponent.setSql(sql);
        }
      }
    }
  }

  // 刷新网格数据
  function refreshGrid() {
    // DataGrid 组件会自动刷新
  }

  function openShellPanel(panel) {
    shellPanel = panel;
  }

  function closeShellPanel() {
    shellPanel = null;
  }
</script>

<div class="app-container">
  <NotificationCenter />
  <header class="app-header">
    <div class="logo">QueryLab</div>
    <nav class="nav">
      <button on:click={() => openShellPanel('settings')}>设置</button>
      <button on:click={() => openShellPanel('help')}>帮助</button>
      <button on:click={() => openShellPanel('about')}>关于</button>
    </nav>
    <div class="version-info">v{appInfo.version} ({appInfo.build})</div>
  </header>

  <main class="app-main">
    <aside class="sidebar">
      <div class="sidebar-section">
        <div class="sidebar-header">连接</div>
        <ConnectionManager
          connections={connections}
          selectedConnection={selectedConnection}
          onConnect={handleConnect}
        />
      </div>

      <div class="sidebar-section">
        <div class="sidebar-header">Schema</div>
        <SchemaTree
          bind:this={schemaTreeComponent}
          connection={selectedConnection}
          on:selectTable={handleSelectTable}
          on:createTable={handleCreateTable}
        />
      </div>
    </aside>

    <section class="workspace">
      <!-- 视图切换器 -->
      <div class="view-switcher">
        <button
          class="view-btn"
          class:active={viewMode === 'query'}
          on:click={() => setViewMode('query')}
        >
          SQL 查询
        </button>
        {#if currentTableName}
          <button
            class="view-btn"
            class:active={viewMode === 'grid'}
            on:click={() => setViewMode('grid')}
          >
            数据网格
          </button>
          <button
            class="view-btn design-btn"
            class:active={viewMode === 'design'}
            on:click={() => setViewMode('design')}
          >
            📋 设计表
          </button>
        {/if}
        <button
          class="view-btn sync-btn"
          class:active={viewMode === 'sync'}
          on:click={openDataSync}
        >
          🔍 结构对比（预览）
        </button>
        <button
          class="view-btn backup-btn"
          class:active={viewMode === 'backup'}
          on:click={openDatabaseBackup}
        >
          💾 备份还原
        </button>
      </div>

      {#if isCreatingNewTable}
        <div class="creating-indicator">📝 新建表: {targetDatabase}</div>
      {/if}

      {#if viewMode === 'query' || (!currentTableName && !isCreatingNewTable)}
        <!-- SQL 查询模式 -->
        <div class="editor-section">
          <SqlEditor
            bind:this={editorComponent}
            connection={selectedConnection}
            onExecute={handleExecuteQuery}
            onBatchExecute={handleBatchExecute}
          />
        </div>

        <div class="results-section">
          <ResultsPanel
            result={queryResult}
            loading={queryLoading}
            error={queryError}
            connection={selectedConnection}
            tableName={currentTableName}
            editableTableName={editableResultTableName}
            onRefresh={() => {
              if (currentTableName) {
                const parts = currentTableName.split('.');
                if (parts.length === 2) {
                  const sql = `SELECT * FROM \`${parts[0]}\`.\`${parts[1]}\` LIMIT 1000;`;
                  executeQuery(sql);
                }
              }
            }}
          />
        </div>
      {:else if viewMode === 'grid'}
        <!-- 数据网格模式 -->
        <div class="grid-section">
          <DataGrid
            connection={selectedConnection}
            tableName={currentTableName}
            onRefresh={refreshGrid}
          />
        </div>
      {:else if viewMode === 'design'}
        <!-- 表设计模式 -->
        <div class="design-section">
          <TableDesigner
            connection={selectedConnection}
            tableName={isCreatingNewTable ? null : currentTableName}
            targetDatabase={targetDatabase}
            isCreatingNewTable={isCreatingNewTable}
            onClose={() => {
              isCreatingNewTable = false;
              targetDatabase = '';
              setViewMode('grid');
            }}
            onRefresh={() => {
              // 如果是新建表模式，刷新 SchemaTree 中对应数据库的表列表
              if (isCreatingNewTable && targetDatabase && schemaTreeComponent) {
                schemaTreeComponent.refreshDatabase(targetDatabase);
                // 保持在当前数据库并切换到网格模式，方便查看新表
                currentDatabase = targetDatabase;
                viewMode = 'grid';
              } else if (currentTableName) {
                // 编辑模式，刷新当前表数据
                refreshGrid();
              }
              // 重置新建表状态
              isCreatingNewTable = false;
              targetDatabase = '';
            }}
          />
        </div>
      {:else if viewMode === 'sync'}
        <!-- 数据同步模式 -->
        <div class="sync-section">
          <DataSync
            connection={selectedConnection}
            databases={databases}
            on:syncComplete={handleSyncComplete}
            on:close={() => setViewMode('query')}
          />
        </div>
      {:else if viewMode === 'backup'}
        <!-- 数据库备份模式 -->
        <div class="backup-section">
          <DatabaseBackup
            connection={selectedConnection}
            databases={databases}
            onClose={() => setViewMode('query')}
          />
        </div>
      {/if}
    </section>
  </main>

  <footer class="app-footer">
    <span>{statusMessage}</span>
    <span>平台: {appInfo.platform}</span>
  </footer>

  {#if shellPanel}
    <div
      class="shell-panel-overlay"
      role="button"
      tabindex="0"
      aria-label="关闭信息面板"
      on:click={(event) => event.target === event.currentTarget && closeShellPanel()}
      on:keydown={(event) => event.key === 'Escape' && closeShellPanel()}
    >
      <div class="shell-panel" role="dialog" aria-modal="true" aria-label={shellPanel}>
        <div class="shell-panel-header">
          <div>
            <div class="shell-panel-eyebrow">QueryLab</div>
            <h2>
              {#if shellPanel === 'settings'}设置{/if}
              {#if shellPanel === 'help'}帮助{/if}
              {#if shellPanel === 'about'}关于{/if}
            </h2>
          </div>
          <button class="shell-panel-close" on:click={closeShellPanel}>&times;</button>
        </div>

        <div class="shell-panel-body">
          {#if shellPanel === 'settings'}
            <section class="shell-section">
              <h3>安全与存储</h3>
              <ul>
                <li>连接密码存储在系统钥匙串中，不再写入 `connections.json`。</li>
                <li>SQL 历史默认仅保留当前会话；如需本地保存，可在 SQL 编辑器历史面板中开启。</li>
                <li>数据库备份当前仅支持 SQL 文件导出与导入。</li>
              </ul>
            </section>

            <section class="shell-section">
              <h3>当前限制</h3>
              <ul>
                <li>`结构对比（预览）` 当前只支持结构差异分析与结构 SQL 执行，不提供真实数据同步。</li>
                <li>结果面板仅在识别为“单表直接查询”时开放单元格编辑；复杂 SQL 结果默认只读。</li>
              </ul>
            </section>
          {/if}

          {#if shellPanel === 'help'}
            <section class="shell-section">
              <h3>快速开始</h3>
              <ol>
                <li>先在左上角连接区新建数据库连接。</li>
                <li>用“测试连接”确认可达，再点击连接名称进入主工作区。</li>
                <li>在 Schema 区选择表，进入数据网格或表设计视图。</li>
                <li>在 SQL 查询页执行语句；历史记录和代码片段在编辑器工具栏中打开。</li>
              </ol>
            </section>

            <section class="shell-section">
              <h3>核心能力边界</h3>
              <ul>
                <li>备份还原：目前只支持 SQL 备份文件。</li>
                <li>结构对比：目前仅支持结构差异对比与结构变更预演。</li>
                <li>数据编辑：支持表格单元格编辑、插入、删除，但复杂批量修改仍建议先备份再操作。</li>
              </ul>
            </section>

            <section class="shell-section">
              <h3>快捷键</h3>
              <ul>
                <li>`Ctrl+Enter`：执行 SQL</li>
                <li>`Ctrl+S`：格式化 SQL</li>
                <li>`Ctrl+H`：打开历史记录</li>
                <li>`Ctrl+K`：清空编辑器</li>
                <li>`F1`：插入 SQL 代码片段</li>
              </ul>
            </section>
          {/if}

          {#if shellPanel === 'about'}
            <section class="shell-section">
              <h3>应用信息</h3>
              <ul>
                <li>版本：`{appInfo.version}`</li>
                <li>平台：`{appInfo.platform}`</li>
                <li>构建：`{appInfo.build}`</li>
              </ul>
            </section>

            <section class="shell-section">
              <h3>定位</h3>
              <p>
                QueryLab 是一个本地优先的数据库工作台，当前首期聚焦 MySQL / MariaDB，
                重点覆盖连接测试、Schema 浏览、SQL 执行、数据网格、表设计和 SQL 备份。
              </p>
            </section>

            <section class="shell-section">
              <h3>当前发布阶段</h3>
              <p>
                当前版本仍处于开发中，已具备核心工作流，但部分能力仍以预览或受限形态提供。
              </p>
            </section>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    overflow: hidden;
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #1e1e1e;
    color: #d4d4d4;
  }

  .app-header {
    display: flex;
    align-items: center;
    padding: 0 16px;
    height: 48px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
  }

  .logo {
    font-weight: 600;
    font-size: 16px;
    margin-right: 24px;
    color: #007acc;
  }

  .nav {
    display: flex;
    gap: 4px;
    flex: 1;
  }

  .nav button {
    background: transparent;
    border: none;
    color: #d4d4d4;
    padding: 6px 12px;
    cursor: pointer;
    border-radius: 4px;
    font-size: 13px;
  }

  .nav button:hover {
    background: #3e3e3e;
  }

  .shell-panel-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: flex-end;
    z-index: 2200;
  }

  .shell-panel {
    width: 420px;
    max-width: calc(100vw - 24px);
    height: 100%;
    background: #202123;
    border-left: 1px solid #3e3e3e;
    box-shadow: -12px 0 32px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
  }

  .shell-panel-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 18px 20px;
    border-bottom: 1px solid #3e3e3e;
  }

  .shell-panel-header h2 {
    margin: 4px 0 0;
    font-size: 20px;
  }

  .shell-panel-eyebrow {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #888;
  }

  .shell-panel-close {
    background: transparent;
    border: none;
    color: #888;
    font-size: 20px;
    cursor: pointer;
  }

  .shell-panel-body {
    padding: 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .shell-section {
    background: #252526;
    border: 1px solid #333;
    border-radius: 10px;
    padding: 16px;
  }

  .shell-section h3 {
    margin: 0 0 10px;
    font-size: 14px;
  }

  .shell-section p,
  .shell-section li {
    font-size: 13px;
    line-height: 1.6;
    color: #d4d4d4;
  }

  .shell-section ul,
  .shell-section ol {
    margin: 0;
    padding-left: 18px;
  }

  .version-info {
    font-size: 12px;
    color: #888;
  }

  .app-main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 280px;
    background: #252526;
    border-right: 1px solid #3e3e3e;
    display: flex;
    flex-direction: column;
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .sidebar-section:first-child {
    flex: 0 0 auto;
    max-height: 50%;
    border-bottom: 1px solid #3e3e3e;
  }

  .sidebar-section:last-child {
    flex: 1;
  }

  .sidebar-header {
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: #888;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
  }

  .workspace {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .view-switcher {
    display: flex;
    gap: 4px;
    padding: 8px 16px;
    background: #252526;
    border-bottom: 1px solid #3e3e3e;
  }

  .view-btn {
    padding: 6px 16px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: #888;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .view-btn:hover {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .view-btn.active {
    background: #007acc;
    color: white;
    border-color: #005a9e;
  }

  .editor-section {
    flex: 1;
    min-height: 120px;
  }

  .results-section {
    flex: 1;
    min-height: 120px;
    border-top: 1px solid #3e3e3e;
  }

  .grid-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .design-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .design-btn.active {
    background: #9b46c8;
    border-color: #7a35a0;
  }

  .sync-btn.active {
    background: #0e639c;
    border-color: #0a4a74;
  }

  .backup-btn.active {
    background: #c84e4e;
    border-color: #a03535;
  }

  .sync-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .backup-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .creating-indicator {
    padding: 8px 16px;
    background: #2a3a2e;
    color: #4ec9b0;
    font-size: 12px;
  }

  .app-footer {
    display: flex;
    justify-content: space-between;
    padding: 0 16px;
    height: 24px;
    background: #007acc;
    font-size: 12px;
    align-items: center;
  }
</style>
