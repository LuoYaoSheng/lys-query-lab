<script>
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  export let connection = null;

  const dispatch = createEventDispatcher();

  let databases = [];
  let tablesData = {}; // { db: [tables] }
  let expandedDbs = new Set();
  let loading = false;
  let loadingTables = new Set();
  let error = null;

  // 表选择事件
  function selectTable(db, table) {
    dispatch('selectTable', { database: db, table });
  }

  // 获取数据库列表
  async function loadDatabases() {
    if (!connection) return;
    loading = true;
    error = null;
    try {
      databases = await invoke('meta_list_databases', { connection });
      // 过滤掉系统数据库
      databases = databases.filter(db =>
        !['information_schema', 'mysql', 'performance_schema', 'sys'].includes(db)
      );
    } catch (err) {
      error = err.toString();
      console.error('Failed to load databases:', err);
    } finally {
      loading = false;
    }
  }

  // 获取表列表
  async function loadTables(db) {
    if (loadingTables.has(db) || tablesData[db]) return;
    loadingTables.add(db);
    try {
      const result = await invoke('meta_list_tables', {
        connection,
        database: db,
        includeViews: true
      });
      tablesData[db] = result;
      tablesData = { ...tablesData };
    } catch (err) {
      console.error('Failed to load tables:', err);
    } finally {
      loadingTables = new Set([...loadingTables].filter(x => x !== db));
    }
  }

  // 切换数据库展开状态
  async function toggleDatabase(db) {
    if (expandedDbs.has(db)) {
      expandedDbs.delete(db);
      expandedDbs = new Set(expandedDbs);
    } else {
      expandedDbs.add(db);
      expandedDbs = new Set(expandedDbs);
      await loadTables(db);
    }
  }

  // 点击表项 - 阻止冒泡
  function handleTableClick(db, table, event) {
    console.log('[SchemaTree] handleTableClick called:', db, table);
    event.stopPropagation();
    selectTable(db, table);
  }

  // 监听连接变化
  $: if (connection) {
    loadDatabases();
    tablesData = {};
    expandedDbs = new Set();
  }

  // 判断是否是视图
  function isView(table) {
    return table.table_type === 'VIEW';
  }

  // 判断是否是系统数据库
  function isSystemDb(db) {
    return ['information_schema', 'mysql', 'performance_schema', 'sys'].includes(db);
  }
</script>

<div class="schema-tree">
  {#if !connection}
    <div class="empty-state">请先选择连接</div>
  {:else if loading}
    <div class="loading">加载中...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if databases.length === 0}
    <div class="empty-state">无可用数据库</div>
  {:else}
    <div class="tree">
      {#each databases as db}
        {@const isExpanded = expandedDbs.has(db)}
        {@const tables = tablesData[db] || []}
        {@const isLoadingTables = loadingTables.has(db)}

        <div class="tree-node">
          <div
            class="tree-node-header"
            on:click={() => toggleDatabase(db)}
          >
            <span class="expand-icon">{isExpanded ? '▼' : '▶'}</span>
            <span class="node-icon">📁</span>
            <span class="node-label">{db}</span>
          </div>

          {#if isExpanded}
            <div class="tree-children">
              {#if isLoadingTables}
                <div class="loading-tables">加载中...</div>
              {:else if tables.length === 0}
                <div class="empty-tables">无表</div>
              {:else}
                {#each tables as table}
                  <div
                    class="table-item"
                    class:view={isView(table)}
                    on:click={(e) => handleTableClick(db, table.name, e)}
                  >
                    <span class="table-icon">{isView(table) ? '👁️' : '📊'}</span>
                    <span class="table-name">{table.name}</span>
                    {#if table.comment}
                      <span class="table-comment" title={table.comment}>{table.comment}</span>
                    {/if}
                  </div>
                {/each}
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .schema-tree {
    height: 100%;
    overflow-y: auto;
  }

  .empty-state, .loading, .error {
    padding: 16px;
    text-align: center;
    color: #666;
    font-size: 13px;
  }

  .error {
    color: #f48771;
  }

  .tree {
    padding: 4px 0;
  }

  .tree-node {
    user-select: none;
  }

  .tree-node-header {
    display: flex;
    align-items: center;
    padding: 4px 16px;
    cursor: pointer;
    font-size: 13px;
  }

  .tree-node-header:hover {
    background: #2a2d2e;
  }

  .expand-icon {
    width: 16px;
    font-size: 10px;
    color: #888;
  }

  .node-icon {
    margin-right: 6px;
  }

  .node-label {
    flex: 1;
  }

  .tree-children {
    padding-left: 0;
  }

  .loading-tables, .empty-tables {
    padding: 4px 16px 4px 32px;
    font-size: 12px;
    color: #888;
  }

  .table-item {
    display: flex;
    align-items: center;
    padding: 4px 16px 4px 32px;
    cursor: pointer;
    font-size: 13px;
  }

  .table-item:hover {
    background: #2a2d2e;
  }

  .table-icon {
    margin-right: 6px;
  }

  .table-name {
    flex: 1;
  }

  .table-comment {
    font-size: 11px;
    color: #888;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100px;
  }

  .table-item.view .table-name {
    color: #dcdcaa;
  }
</style>
