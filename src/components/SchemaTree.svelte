<script>
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  import { notifyError, notifyInfo, notifySuccess } from '../lib/notifications';

  export let connection = null;

  const dispatch = createEventDispatcher();

  let databases = [];
  let tablesData = {}; // { db: [tables] }
  let expandedDbs = new Set();
  let loading = false;
  let loadingTables = new Set();
  let error = null;

  // 对话框状态
  let showCreateDbDialog = false;
  let newDbName = '';
  let creating = false;
  let selectedCharset = 'utf8mb4';
  let selectedCollation = 'utf8mb4_unicode_ci';

  // 右键菜单状态
  let contextMenu = null; // { db, table, x, y, isView }
  let showDeleteTableDialog = false;
  let showRenameTableDialog = false;
  let showTruncateTableDialog = false;
  let deleteTableName = '';
  let renameTableDatabase = '';
  let renameTableName = '';
  let renameTableNewName = '';
  let truncateTableName = '';
  let tableOperating = false;

  // 暴露给父组件的方法
  export function refreshDatabase(db) {
    // 清除缓存，强制重新加载
    delete tablesData[db];
    tablesData = { ...tablesData };
    // 确保数据库展开，然后重新加载表列表
    if (!expandedDbs.has(db)) {
      expandedDbs.add(db);
      expandedDbs = new Set(expandedDbs);
    }
    loadTables(db);
  }

  export function refreshAll() {
    tablesData = {};
    loadDatabases();
  }

  // 可用的字符集
  const charsets = [
    { value: 'utf8mb4', label: 'utf8mb4 (推荐)' },
    { value: 'utf8', label: 'utf8' },
    { value: 'latin1', label: 'latin1' },
    { value: 'gbk', label: 'gbk (中文)' },
    { value: 'big5', label: 'big5 (繁体中文)' },
    { value: 'ascii', label: 'ascii' }
  ];

  // 字符集对应的排序规则
  const collationMap = {
    'utf8mb4': [
      { value: 'utf8mb4_unicode_ci', label: 'utf8mb4_unicode_ci (推荐)' },
      { value: 'utf8mb4_general_ci', label: 'utf8mb4_general_ci' },
      { value: 'utf8mb4_0900_ai_ci', label: 'utf8mb4_0900_ai_ci' }
    ],
    'utf8': [
      { value: 'utf8_unicode_ci', label: 'utf8_unicode_ci' },
      { value: 'utf8_general_ci', label: 'utf8_general_ci' }
    ],
    'latin1': [
      { value: 'latin1_swedish_ci', label: 'latin1_swedish_ci' },
      { value: 'latin1_general_ci', label: 'latin1_general_ci' }
    ],
    'gbk': [
      { value: 'gbk_chinese_ci', label: 'gbk_chinese_ci' }
    ],
    'big5': [
      { value: 'big5_chinese_ci', label: 'big5_chinese_ci' }
    ],
    'ascii': [
      { value: 'ascii_general_ci', label: 'ascii_general_ci' }
    ]
  };

  // 当前可用的排序规则
  let availableCollations = collationMap[selectedCharset];

  // 当字符集改变时，更新排序规则选项
  function handleCharsetChange() {
    availableCollations = collationMap[selectedCharset] || [];
    // 自动选择第一个排序规则
    if (availableCollations.length > 0) {
      selectedCollation = availableCollations[0].value;
    }
  }

  // 表选择事件
  function selectTable(db, table) {
    dispatch('selectTable', { database: db, table });
  }

  // 刷新数据库列表
  async function refreshDatabases() {
    await loadDatabases();
  }

  // 获取数据库列表
  async function loadDatabases() {
    if (!connection) return;
    loading = true;
    error = null;
    try {
      const result = await invoke('meta_list_databases', { connection });
      // 过滤掉系统数据库
      databases = result.filter(db =>
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
    event.stopPropagation();
    // 关闭右键菜单
    contextMenu = null;
    selectTable(db, table);
  }

  // 右键菜单
  function handleTableContextMenu(db, table, isView, event) {
    event.preventDefault();
    event.stopPropagation();
    contextMenu = {
      db,
      table,
      isView,
      x: event.clientX,
      y: event.clientY
    };
  }

  // 关闭右键菜单
  function closeContextMenu() {
    contextMenu = null;
  }

  function closeCreateDbDialog() {
    showCreateDbDialog = false;
  }

  function closeDeleteTableDialog() {
    showDeleteTableDialog = false;
  }

  function closeRenameTableDialog() {
    showRenameTableDialog = false;
    renameTableDatabase = '';
    renameTableName = '';
    renameTableNewName = '';
  }

  function closeTruncateTableDialog() {
    showTruncateTableDialog = false;
  }

  function handleOverlayClick(event, close) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleOverlayKeydown(event, close) {
    if (event.key === 'Escape') {
      close();
    }
  }

  // 删除表
  async function deleteTable() {
    if (!contextMenu) return;
    const { db, table } = contextMenu;
    deleteTableName = `${db}.${table}`;
    showDeleteTableDialog = true;
    contextMenu = null;
  }

  // 执行删除表
  async function executeDeleteTable() {
    if (!connection || !deleteTableName) return;

    tableOperating = true;
    try {
      const [db, table] = deleteTableName.split('.');
      const sql = `DROP TABLE \`${db}\`.\`${table}\`;`;
      await invoke('query_execute', { connection, sql, maxRows: 0 });
      notifySuccess('表删除成功');
      showDeleteTableDialog = false;
      deleteTableName = '';
      // 刷新表列表
      delete tablesData[db];
      tablesData = { ...tablesData };
      await loadTables(db);
    } catch (err) {
      notifyError('删除表失败: ' + err);
    } finally {
      tableOperating = false;
    }
  }

  // 重命名表
  async function renameTable() {
    if (!contextMenu || contextMenu.isView) {
      notifyInfo('视图不支持重命名');
      contextMenu = null;
      return;
    }
    const { db, table } = contextMenu;
    renameTableDatabase = db;
    renameTableName = table;
    renameTableNewName = table;
    showRenameTableDialog = true;
    contextMenu = null;
  }

  // 执行重命名表
  async function executeRenameTable() {
    if (!connection || !renameTableDatabase || !renameTableName || !renameTableNewName) return;
    if (renameTableName === renameTableNewName) {
      showRenameTableDialog = false;
      return;
    }

    tableOperating = true;
    try {
      const targetDb = renameTableDatabase;
      const sql = `RENAME TABLE \`${targetDb}\`.\`${renameTableName}\` TO \`${targetDb}\`.\`${renameTableNewName}\`;`;
      await invoke('query_execute', { connection, sql, maxRows: 0 });
      notifySuccess('表重命名成功');
      showRenameTableDialog = false;
      renameTableDatabase = '';
      renameTableName = '';
      renameTableNewName = '';
      // 刷新表列表
      delete tablesData[targetDb];
      tablesData = { ...tablesData };
      await loadTables(targetDb);
    } catch (err) {
      notifyError('重命名表失败: ' + err);
    } finally {
      tableOperating = false;
    }
  }

  // 清空表
  async function truncateTable() {
    if (!contextMenu || contextMenu.isView) {
      notifyInfo('视图不支持清空');
      contextMenu = null;
      return;
    }
    const { db, table } = contextMenu;
    truncateTableName = `${db}.${table}`;
    showTruncateTableDialog = true;
    contextMenu = null;
  }

  // 执行清空表
  async function executeTruncateTable() {
    if (!connection || !truncateTableName) return;

    tableOperating = true;
    try {
      const [db, table] = truncateTableName.split('.');
      const sql = `TRUNCATE TABLE \`${db}\`.\`${table}\`;`;
      await invoke('query_execute', { connection, sql, maxRows: 0 });
      notifySuccess('表数据已清空');
      showTruncateTableDialog = false;
      truncateTableName = '';
    } catch (err) {
      notifyError('清空表失败: ' + err);
    } finally {
      tableOperating = false;
    }
  }

  // 刷新表列表
  async function refreshTableList() {
    if (!contextMenu) return;
    const { db } = contextMenu;
    contextMenu = null;
    delete tablesData[db];
    tablesData = { ...tablesData };
    await loadTables(db);
  }

  // 创建数据库
  async function createDatabase() {
    if (!newDbName.trim()) {
      notifyError('请输入数据库名称');
      return;
    }

    if (!connection) {
      notifyError('请先连接数据库');
      return;
    }

    const dbName = newDbName.trim();
    creating = true;
    try {
      const result = await invoke('meta_create_database', {
        params: {
          connection,
          name: dbName,
          charset: selectedCharset,
          collation: selectedCollation
        }
      });

      notifySuccess('数据库创建成功');

      // 关闭对话框并重置表单
      showCreateDbDialog = false;
      newDbName = '';
      selectedCharset = 'utf8mb4';
      selectedCollation = 'utf8mb4_unicode_ci';
      availableCollations = collationMap['utf8mb4'];

      // 刷新数据库列表
      await loadDatabases();

      // 自动展开新创建的数据库
      expandedDbs.add(dbName);
      expandedDbs = new Set(expandedDbs);

      // 预加载表列表（即使是空的）
      await loadTables(dbName);
    } catch (err) {
      console.error('Create database error:', err);
      notifyError('创建数据库失败: ' + err);
    } finally {
      creating = false;
    }
  }

  // 创建表事件
  function handleCreateTable(db) {
    dispatch('createTable', { database: db });
  }

  // 监听连接变化
  $: if (connection) {
    loadDatabases();
    tablesData = {};
    expandedDbs = new Set();
  } else {
    databases = [];
    tablesData = {};
    expandedDbs = new Set();
    error = null;
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
  {:else}
    <!-- 工具栏 -->
    <div class="schema-toolbar">
      <button
        class="toolbar-btn"
        on:click={() => showCreateDbDialog = true}
        title="新建数据库"
      >
        + 数据库
      </button>
    </div>

    <!-- 数据库列表 -->
    <div class="tree">
      {#if databases.length === 0}
        <div class="empty-state">无可用数据库</div>
      {/if}

      {#each databases as db}
        {@const isExpanded = expandedDbs.has(db)}
        {@const tables = tablesData[db] || []}
        {@const isLoadingTables = loadingTables.has(db)}

        <div class="tree-node">
          <button
            type="button"
            class="tree-node-header"
            on:click={() => toggleDatabase(db)}
            aria-expanded={isExpanded}
          >
            <span class="expand-icon">{isExpanded ? '▼' : '▶'}</span>
            <span class="node-icon">📁</span>
            <span class="node-label">{db}</span>
          </button>

          {#if isExpanded}
            <div class="tree-children">
              <!-- 新建表按钮 -->
              <div class="add-table-row">
                <button
                  class="add-table-btn"
                  on:click={() => handleCreateTable(db)}
                  title="新建表"
                >
                  <span class="add-icon">+</span>
                  <span>新建表</span>
                </button>
              </div>

              {#if isLoadingTables}
                <div class="loading-tables">加载中...</div>
              {:else if tables.length === 0}
                <div class="empty-tables">无表</div>
              {:else}
                {#each tables as table}
                  {@const tableView = isView(table)}
                  <button
                    type="button"
                    class="table-item"
                    class:view={tableView}
                    on:click={(e) => handleTableClick(db, table.name, e)}
                    on:contextmenu={(e) => handleTableContextMenu(db, table.name, tableView, e)}
                  >
                    <span class="table-icon">{tableView ? '👁️' : '📊'}</span>
                    <span class="table-name">{table.name}</span>
                    {#if table.comment}
                      <span class="table-comment" title={table.comment}>{table.comment}</span>
                    {/if}
                  </button>
                {/each}
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- 创建数据库对话框 -->
{#if showCreateDbDialog}
  <div
    class="dialog-overlay"
    role="button"
    tabindex="0"
    aria-label="关闭新建数据库对话框"
    on:click={(e) => handleOverlayClick(e, closeCreateDbDialog)}
    on:keydown={(e) => handleOverlayKeydown(e, closeCreateDbDialog)}
  >
    <div class="dialog" role="dialog" aria-modal="true" aria-label="新建数据库">
      <div class="dialog-header">
        <h3>新建数据库</h3>
        <button class="dialog-close" on:click={closeCreateDbDialog}>&times;</button>
      </div>
      <div class="dialog-body">
        <div class="form-group">
          <label for="create-db-name">数据库名</label>
          <input
            id="create-db-name"
            type="text"
            class="form-input"
            bind:value={newDbName}
            placeholder="my_database"
            on:keydown={(e) => e.key === 'Enter' && createDatabase()}
          />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label for="create-db-charset">字符集</label>
            <select id="create-db-charset" class="form-select" bind:value={selectedCharset} on:change={handleCharsetChange}>
              {#each charsets as cs}
                <option value={cs.value}>{cs.label}</option>
              {/each}
            </select>
          </div>
          <div class="form-group">
            <label for="create-db-collation">排序规则</label>
            <select id="create-db-collation" class="form-select" bind:value={selectedCollation}>
              {#each availableCollations as col}
                <option value={col.value}>{col.label}</option>
              {/each}
            </select>
          </div>
        </div>
      </div>
      <div class="dialog-footer">
        <button class="btn btn-secondary" on:click={closeCreateDbDialog}>取消</button>
        <button class="btn btn-primary" on:click={createDatabase} disabled={creating}>
          {creating ? '创建中...' : '创建'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- 右键菜单 -->
{#if contextMenu}
  <div
    class="context-menu-overlay"
    role="button"
    tabindex="0"
    aria-label="关闭表右键菜单"
    on:click={closeContextMenu}
    on:keydown={(e) => e.key === 'Escape' && closeContextMenu()}
    style="left: 0; top: 0; right: 0; bottom: 0; position: fixed;"
  >
    <div
      class="context-menu"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    >
      <button class="context-menu-item" on:click={refreshTableList}>
        <span class="menu-icon">🔄</span> 刷新
      </button>
      <button class="context-menu-item" on:click={renameTable}>
        <span class="menu-icon">✏️</span> 重命名表
      </button>
      <button class="context-menu-item" on:click={truncateTable}>
        <span class="menu-icon">🗑️</span> 清空表数据
      </button>
      <div class="context-menu-divider"></div>
      <button class="context-menu-item danger" on:click={deleteTable}>
        <span class="menu-icon">❌</span> 删除表
      </button>
    </div>
  </div>
{/if}

<!-- 删除表确认对话框 -->
{#if showDeleteTableDialog}
  <div
    class="dialog-overlay"
    role="button"
    tabindex="0"
    aria-label="关闭删除表对话框"
    on:click={(e) => handleOverlayClick(e, closeDeleteTableDialog)}
    on:keydown={(e) => handleOverlayKeydown(e, closeDeleteTableDialog)}
  >
    <div class="dialog" role="dialog" aria-modal="true" aria-label="确认删除表">
      <div class="dialog-header">
        <h3>确认删除表</h3>
        <button class="dialog-close" on:click={closeDeleteTableDialog}>&times;</button>
      </div>
      <div class="dialog-body">
        <p>确定要删除表 <strong>{deleteTableName}</strong> 吗？</p>
        <p class="confirm-warning">⚠️ 此操作不可撤销！表结构和数据将被永久删除！</p>
      </div>
      <div class="dialog-footer">
        <button class="btn btn-secondary" on:click={closeDeleteTableDialog} disabled={tableOperating}>取消</button>
        <button class="btn btn-danger" on:click={executeDeleteTable} disabled={tableOperating}>
          {tableOperating ? '删除中...' : '确认删除'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- 重命名表对话框 -->
{#if showRenameTableDialog}
  <div
    class="dialog-overlay"
    role="button"
    tabindex="0"
    aria-label="关闭重命名表对话框"
    on:click={(e) => handleOverlayClick(e, closeRenameTableDialog)}
    on:keydown={(e) => handleOverlayKeydown(e, closeRenameTableDialog)}
  >
    <div class="dialog" role="dialog" aria-modal="true" aria-label="重命名表">
      <div class="dialog-header">
        <h3>重命名表</h3>
        <button class="dialog-close" on:click={closeRenameTableDialog}>&times;</button>
      </div>
      <div class="dialog-body">
        <div class="form-group">
          <label for="rename-table-current">原表名</label>
          <input id="rename-table-current" type="text" class="form-input" value={renameTableName} disabled />
        </div>
        <div class="form-group">
          <label for="rename-table-next">新表名</label>
          <input
            id="rename-table-next"
            type="text"
            class="form-input"
            bind:value={renameTableNewName}
            placeholder="new_table_name"
            on:keydown={(e) => e.key === 'Enter' && executeRenameTable()}
          />
        </div>
      </div>
      <div class="dialog-footer">
        <button class="btn btn-secondary" on:click={closeRenameTableDialog} disabled={tableOperating}>取消</button>
        <button class="btn btn-primary" on:click={executeRenameTable} disabled={tableOperating}>
          {tableOperating ? '重命名中...' : '确认'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- 清空表确认对话框 -->
{#if showTruncateTableDialog}
  <div
    class="dialog-overlay"
    role="button"
    tabindex="0"
    aria-label="关闭清空表对话框"
    on:click={(e) => handleOverlayClick(e, closeTruncateTableDialog)}
    on:keydown={(e) => handleOverlayKeydown(e, closeTruncateTableDialog)}
  >
    <div class="dialog" role="dialog" aria-modal="true" aria-label="确认清空表">
      <div class="dialog-header">
        <h3>确认清空表</h3>
        <button class="dialog-close" on:click={closeTruncateTableDialog}>&times;</button>
      </div>
      <div class="dialog-body">
        <p>确定要清空表 <strong>{truncateTableName}</strong> 的所有数据吗？</p>
        <p class="confirm-warning">⚠️ 此操作不可撤销！所有数据将被永久删除！</p>
      </div>
      <div class="dialog-footer">
        <button class="btn btn-secondary" on:click={closeTruncateTableDialog} disabled={tableOperating}>取消</button>
        <button class="btn btn-danger" on:click={executeTruncateTable} disabled={tableOperating}>
          {tableOperating ? '清空中...' : '确认清空'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .schema-tree {
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .schema-toolbar {
    padding: 8px;
    border-bottom: 1px solid #3e3e3e;
  }

  .toolbar-btn {
    width: 100%;
    padding: 6px 12px;
    background: #2da042;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .toolbar-btn:hover {
    background: #238736;
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
    flex: 1;
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
    width: 100%;
    background: transparent;
    border: none;
    color: inherit;
    font: inherit;
    text-align: left;
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

  .add-table-row {
    padding: 4px 16px 4px 32px;
  }

  .add-table-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    background: transparent;
    border: 1px dashed #4e4e4e;
    border-radius: 4px;
    color: #888;
    font-size: 12px;
    cursor: pointer;
    width: 100%;
    transition: all 0.2s;
  }

  .add-table-btn:hover {
    background: #2a3a2e;
    border-color: #007acc;
    color: #d4d4d4;
  }

  .add-icon {
    font-size: 14px;
    font-weight: bold;
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
    width: 100%;
    background: transparent;
    border: none;
    color: inherit;
    font: inherit;
    text-align: left;
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

  /* 对话框样式 */
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #2d2d2d;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    width: 450px;
    max-width: 90vw;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #3e3e3e;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
  }

  .dialog-close {
    background: none;
    border: none;
    color: #888;
    font-size: 20px;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .dialog-close:hover {
    color: #d4d4d4;
  }

  .dialog-body {
    padding: 16px;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 12px;
    color: #888;
  }

  .form-input {
    width: 100%;
    padding: 8px 12px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
  }

  .form-input:focus {
    outline: none;
    border-color: #007acc;
  }

  .form-row {
    display: flex;
    gap: 12px;
    margin-top: 12px;
  }

  .form-row .form-group {
    flex: 1;
  }

  .form-select {
    width: 100%;
    padding: 8px 12px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
  }

  .form-select:focus {
    outline: none;
    border-color: #007acc;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px;
    border-top: 1px solid #3e3e3e;
  }

  .btn {
    padding: 6px 16px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4e4e4e;
  }

  .btn-primary {
    background: #007acc;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #006cbd;
  }

  .btn-danger {
    background: #d73a49;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #cb2c3b;
  }

  .confirm-warning {
    color: #f48771;
    font-size: 13px;
    margin-top: 12px;
  }

  /* 右键菜单样式 */
  .context-menu-overlay {
    position: fixed;
    z-index: 2000;
  }

  .context-menu {
    position: fixed;
    background: #2d2d2d;
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    min-width: 160px;
    padding: 4px 0;
    z-index: 2001;
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: #d4d4d4;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
  }

  .context-menu-item:hover {
    background: #3e3e3e;
  }

  .context-menu-item.danger {
    color: #f48771;
  }

  .context-menu-item.danger:hover {
    background: #4a2a2a;
  }

  .menu-icon {
    width: 16px;
    text-align: center;
  }

  .context-menu-divider {
    height: 1px;
    background: #3e3e3e;
    margin: 4px 0;
  }
</style>
