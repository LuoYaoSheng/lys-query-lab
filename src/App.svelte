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

  function handleConnect(conn) {
    selectedConnection = conn;
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
    console.log('executeQuery called:', sql);
    console.log('Connection:', selectedConnection);

    if (!selectedConnection) {
      queryError = '请先选择连接';
      statusMessage = 'No connection';
      return;
    }

    queryLoading = true;
    queryError = null;
    queryResult = null;
    statusMessage = 'Executing...';

    try {
      console.log('Calling query_execute...');
      const result = await invoke('query_execute', {
        connection: selectedConnection,
        sql,
        maxRows: 1000
      });
      console.log('Query result:', result);
      queryResult = result;
      statusMessage = `Query completed in ${result.elapsedMs}ms`;
    } catch (err) {
      console.error('Query error:', err);
      queryError = String(err);
      statusMessage = 'Query failed';
    } finally {
      queryLoading = false;
      console.log('Query done, loading = false');
    }
  }

  // 批量执行处理
  async function handleBatchExecute(sql, useTransaction) {
    console.log('handleBatchExecute called:', sql, 'useTransaction:', useTransaction);

    if (!selectedConnection) {
      queryError = '请先选择连接';
      statusMessage = 'No connection';
      return;
    }

    queryLoading = true;
    queryError = null;
    queryResult = null;

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
</script>

<div class="app-container">
  <header class="app-header">
    <div class="logo">QueryLab</div>
    <nav class="nav">
      <button>文件</button>
      <button>编辑</button>
      <button>视图</button>
      <button>帮助</button>
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
          🔄 数据同步
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
