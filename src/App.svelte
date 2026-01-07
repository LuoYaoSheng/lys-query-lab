<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ConnectionManager from './components/ConnectionManager.svelte';
  import SchemaTree from './components/SchemaTree.svelte';
  import SqlEditor from './components/SqlEditor.svelte';
  import ResultsPanel from './components/ResultsPanel.svelte';

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

  // 将 executeQuery 暴露给子组件调用
  function handleExecuteQuery(sql) {
    executeQuery(sql);
  }

  function handleSelectTable(e) {
    const { database, table } = e.detail;
    console.log('Selected table:', database, table);
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
          connection={selectedConnection}
          onSelectTable={handleSelectTable}
        />
      </div>
    </aside>

    <section class="workspace">
      <div class="editor-section">
        <SqlEditor
          connection={selectedConnection}
          onExecute={handleExecuteQuery}
        />
      </div>

      <div class="results-section">
        <ResultsPanel
          result={queryResult}
          loading={queryLoading}
          error={queryError}
        />
      </div>
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

  .editor-section {
    flex: 1;
    min-height: 120px;
  }

  .results-section {
    flex: 1;
    min-height: 120px;
    border-top: 1px solid #3e3e3e;
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
