<script>
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';

  export let result = null;
  export let loading = false;
  export let error = null;
  export let connection = null;  // 需要连接信息用于更新
  export let tableName = '';      // 当前查询的表名
  export let editableTableName = '';
  export let onRefresh = () => {}; // 刷新回调

  // 编辑状态
  let editingCell = null;  // { rowIndex, colIndex, value }
  let updateLoading = false;
  let updateMessage = null;
  let activeSetIndex = 0;
  let exportLoading = false;
  let tableSchema = null;
  let schemaKey = '';
  let lastQueryId = '';

  $: activeSet = result?.sets[activeSetIndex] || null;
  $: totalSets = result?.sets.length || 0;
  $: canExport = activeSet && activeSet.columns.length > 0 && getAllRows().length > 0;
  $: primaryKeyName = getPrimaryKeyName();
  $: primaryKeyColumn = getPrimaryKeyColumn();

  $: {
    const nextSchemaKey = connection && editableTableName
      ? `${connection.id || connection.host || 'connection'}:${editableTableName}`
      : '';

    if (nextSchemaKey !== schemaKey) {
      schemaKey = nextSchemaKey;
      tableSchema = null;
      if (nextSchemaKey) {
        loadTableSchema();
      }
    }
  }

  $: {
    if (result?.queryId && result.queryId !== lastQueryId) {
      lastQueryId = result.queryId;
      activeSetIndex = 0;
      editingCell = null;
      updateMessage = null;
    } else if (!result) {
      lastQueryId = '';
      activeSetIndex = 0;
      editingCell = null;
      updateMessage = null;
    }
  }

  async function loadTableSchema() {
    if (!connection || !editableTableName) {
      tableSchema = null;
      return;
    }

    try {
      const [database, table] = editableTableName.split('.');
      tableSchema = await invoke('meta_get_table_schema', {
        connection,
        database,
        table,
      });
    } catch (err) {
      console.error('Failed to load editable schema:', err);
      tableSchema = null;
    }
  }

  function getPrimaryKeyName() {
    const primaryIndex = tableSchema?.indexes?.find((index) => index.name === 'PRIMARY');
    if (!primaryIndex || primaryIndex.columns.length !== 1) {
      return null;
    }
    return primaryIndex.columns[0];
  }

  function getPrimaryKeyColumn() {
    if (!activeSet || !activeSet.columns || !primaryKeyName) return null;
    const index = activeSet.columns.findIndex((column) => column.name === primaryKeyName);
    return index >= 0 ? index : null;
  }

  // 判断值的类型
  function getValueType(value) {
    if (value === null) return 'null';
    if (typeof value === 'number') return 'number';
    if (typeof value === 'boolean') return 'boolean';
    if (Array.isArray(value)) return 'bytes';
    return 'string';
  }

  // 格式化值显示
  function formatValue(value) {
    if (value === null) return 'NULL';
    if (typeof value === 'number') return value.toString();
    if (typeof value === 'boolean') return value ? 'true' : 'false';
    if (Array.isArray(value)) return `[${value.length} bytes]`;
    return value;
  }

  // 获取原始值（用于编辑）
  function getRawValue(value) {
    if (value === null) return '';
    return String(value);
  }

  // 开始编辑单元格
  function startEdit(rowIndex, colIndex, currentValue) {
    if (!isEditable()) return;
    if (colIndex === primaryKeyColumn) return; // 不允许编辑主键

    editingCell = {
      rowIndex,
      colIndex,
      value: getRawValue(currentValue),
      isNull: currentValue === null
    };
  }

  // 取消编辑
  function cancelEdit() {
    editingCell = null;
  }

  // 保存编辑
  async function saveEdit() {
    if (!editingCell) return;

    updateLoading = true;
    updateMessage = null;

    try {
      const rows = getAllRows();
      const row = rows[editingCell.rowIndex];
      const primaryKeyValue = String(row[primaryKeyColumn]);
      const columnName = activeSet.columns[editingCell.colIndex].name;

      const result = await invoke('query_update_cell', {
        params: {
          connection,
          table: editableTableName,
          column: columnName,
          primary_key: primaryKeyName,
          primary_key_value: primaryKeyValue,
          new_value: editingCell.value,
          is_null: editingCell.isNull
        }
      });

      updateMessage = { success: true, text: result.message };

      // 延迟关闭编辑状态，让用户看到成功消息
      setTimeout(() => {
        editingCell = null;
        updateMessage = null;
        // 刷新数据
        onRefresh();
      }, 500);
    } catch (err) {
      updateMessage = { success: false, text: '更新失败: ' + err };
    } finally {
      updateLoading = false;
    }
  }

  // 处理键盘事件
  function handleKeydown(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      saveEdit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelEdit();
    }
  }

  // 获取所有行数据
  function getAllRows() {
    let allRows = [];
    if (activeSet && activeSet.chunks) {
      for (const chunk of activeSet.chunks) {
        allRows = allRows.concat(chunk.rows || []);
      }
    }
    return allRows;
  }

  // 检查是否可编辑
  function isEditable() {
    return Boolean(
      connection &&
      editableTableName &&
      activeSet &&
      activeSet.columns.length > 0 &&
      primaryKeyName &&
      primaryKeyColumn !== null
    );
  }

  // 导出为 CSV
  function exportCSV() {
    if (!activeSet) return;

    const rows = getAllRows();
    const columns = activeSet.columns;

    // 构建 CSV 内容
    let csv = '';

    // 表头
    csv += columns.map(col => escapeCSV(col.name)).join(',') + '\n';

    // 数据行
    for (const row of rows) {
      csv += row.map(cell => escapeCSV(cell === null ? 'NULL' : String(cell))).join(',') + '\n';
    }

    downloadFile(csv, `${tableName.replace(/.*\./, '')}_export.csv`, 'text/csv');
  }

  // 导出为 JSON
  function exportJSON() {
    if (!activeSet) return;

    const rows = getAllRows();
    const columns = activeSet.columns;

    // 构建 JSON 内容
    const data = rows.map(row => {
      const obj = {};
      columns.forEach((col, i) => {
        obj[col.name] = row[i] === null ? null : row[i];
      });
      return obj;
    });

    const json = JSON.stringify(data, null, 2);
    downloadFile(json, `${tableName.replace(/.*\./, '')}_export.json`, 'application/json');
  }

  // CSV 转义
  function escapeCSV(value) {
    const str = String(value);
    if (str.includes(',') || str.includes('\n') || str.includes('"')) {
      return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
  }

  // 下载文件
  async function downloadFile(content, filename, mimeType) {
    // 使用 Tauri 的 save 对话框
    try {
      await save({
        title: '导出数据',
        defaultPath: filename,
        filters: [
          {
            name: filename.split('.').pop(),
            extensions: [filename.split('.').pop()]
          }
        ]
      }).then(path => {
        if (path) {
          // 使用 Tauri API 写入文件
          invoke('fs_write_file', {
            path: path,
            contents: content
          }).catch(err => {
            console.error('Write error:', err);
            // 降级使用浏览器下载
            browserDownload(content, filename, mimeType);
          });
        }
      });
    } catch (err) {
      console.error('Save dialog error:', err);
      // 降级使用浏览器下载
      browserDownload(content, filename, mimeType);
    }
  }

  // 浏览器下载（降级方案）
  function browserDownload(content, filename, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
</script>

<div class="results-panel">
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <span>执行中...</span>
    </div>
  {:else if error}
    <div class="error-state">
      <span class="error-icon">⚠️</span>
      <pre class="error-message">{error}</pre>
    </div>
  {:else if !result}
    <div class="empty-state">
      <span>执行 SQL 后结果显示在这里</span>
    </div>
  {:else}
    <div class="results-header">
      <div class="results-left">
        <div class="results-tabs">
          {#each result.sets as set, idx}
            <button
              type="button"
              class="result-tab"
              class:active={idx === activeSetIndex}
              on:click={() => activeSetIndex = idx}
            >
              结果 {idx + 1}
              <span class="rows-count">({set.meta.affectedRows} 行)</span>
            </button>
          {/each}
        </div>
        {#if canExport}
          <div class="export-buttons">
            <button
              class="btn-export"
              on:click={exportCSV}
              disabled={exportLoading}
              title="导出为 CSV"
            >
              CSV
            </button>
            <button
              class="btn-export"
              on:click={exportJSON}
              disabled={exportLoading}
              title="导出为 JSON"
            >
              JSON
            </button>
          </div>
        {/if}
      </div>
      <div class="results-info">
        <span>耗时: {result.elapsedMs}ms</span>
        <span>查询ID: {result.queryId.slice(0, 8)}</span>
        {#if isEditable()}
          <span class="edit-hint">双击单元格编辑</span>
        {/if}
      </div>
    </div>

    {#if updateMessage}
      <div class="update-message" class:success={updateMessage.success} class:error={!updateMessage.success}>
        {updateMessage.text}
      </div>
    {/if}

    <div class="results-content">
      {#if activeSet}
        {#if activeSet.columns.length === 0 && activeSet.chunks.length === 0}
          <div class="success-message">
            执行成功，影响 {activeSet.meta.affectedRows} 行
          </div>
        {:else}
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  {#each activeSet.columns as col, i}
                    <th class:primary-key={i === primaryKeyColumn}>
                      <span class="col-name">{col.name}</span>
                      <span class="col-type">{(col.column_type || col.columnType || '').split('::').pop()}</span>
                    </th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each getAllRows() as row, rowIndex}
                  {@const isEditingRow = editingCell && editingCell.rowIndex === rowIndex}
                  <tr class:editing-row={isEditingRow}>
                    {#each row as cell, colIndex}
                      {@const isEditingThisCell = editingCell && editingCell.rowIndex === rowIndex && editingCell.colIndex === colIndex}
                      {@const type = getValueType(cell)}
                      {@const isPrimaryKey = colIndex === primaryKeyColumn}
                      <td
                        class="cell-{type}"
                        class:cell-primary-key={isPrimaryKey}
                        class:cell-editable={!isPrimaryKey && isEditable()}
                        on:dblclick={() => startEdit(rowIndex, colIndex, cell)}
                      >
                        {#if isEditingThisCell}
                          <div class="cell-editor">
                            <input
                              type="text"
                              bind:value={editingCell.value}
                              on:keydown={handleKeydown}
                              disabled={updateLoading}
                              class:is-null={editingCell.isNull}
                            />
                            <div class="cell-editor-actions">
                              <button class="btn-save" on:click={saveEdit} disabled={updateLoading}>
                                {updateLoading ? '...' : '✓'}
                              </button>
                              <button class="btn-cancel" on:click={cancelEdit} disabled={updateLoading}>
                                ✗
                              </button>
                              <button
                                class="btn-null"
                                on:click={() => editingCell.isNull = !editingCell.isNull}
                                class:active={editingCell.isNull}
                                disabled={updateLoading}
                                title="设为 NULL"
                              >
                                NULL
                              </button>
                            </div>
                          </div>
                        {:else}
                          {formatValue(cell)}
                        {/if}
                      </td>
                    {/each}
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .results-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
  }

  .loading-state, .error-state, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #888;
    font-size: 14px;
  }

  .error-state {
    color: #f48771;
    padding: 16px;
  }

  .error-message {
    margin-top: 12px;
    padding: 12px;
    background: #3c1f1e;
    border-radius: 4px;
    max-width: 80%;
    overflow: auto;
  }

  .success-message {
    padding: 32px;
    text-align: center;
    color: #4ec9b0;
  }

  .results-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
    flex-wrap: wrap;
    gap: 8px;
  }

  .results-left {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .results-tabs {
    display: flex;
    gap: 4px;
  }

  .export-buttons {
    display: flex;
    gap: 4px;
  }

  .btn-export {
    padding: 4px 10px;
    background: #3e3e3e;
    border: 1px solid #4e4e4e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 11px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .btn-export:hover:not(:disabled) {
    background: #4e4e4e;
    border-color: #007acc;
  }

  .btn-export:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .result-tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: #3e3e3e;
    border: none;
    border-radius: 4px 4px 0 0;
    color: inherit;
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }

  .result-tab.active {
    background: #007acc;
    color: white;
  }

  .rows-count {
    font-size: 11px;
    opacity: 0.8;
  }

  .results-info {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: #888;
    align-items: center;
  }

  .edit-hint {
    color: #4ec9b0;
    font-size: 11px;
  }

  .update-message {
    padding: 8px 16px;
    font-size: 12px;
    text-align: center;
  }

  .update-message.success {
    background: #1e3a1e;
    color: #4ec9b0;
  }

  .update-message.error {
    background: #3c1f1e;
    color: #f48771;
  }

  .results-content {
    flex: 1;
    overflow: auto;
  }

  .table-wrapper {
    min-height: 100%;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  thead {
    position: sticky;
    top: 0;
    background: #2d2d2d;
    z-index: 1;
  }

  th {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 1px solid #3e3e3e;
    border-right: 1px solid #2d2d2d;
    font-weight: 500;
    white-space: nowrap;
  }

  th.primary-key {
    background: #2a2d3e;
  }

  th.primary-key .col-name::after {
    content: ' 🔑';
    font-size: 10px;
  }

  th:last-child {
    border-right: none;
  }

  .col-name {
    display: block;
  }

  .col-type {
    display: block;
    font-size: 10px;
    color: #888;
    font-weight: normal;
  }

  td {
    padding: 6px 12px;
    border-bottom: 1px solid #2d2d2d;
    border-right: 1px solid #2d2d2d;
    white-space: nowrap;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    position: relative;
  }

  td:last-child {
    border-right: none;
  }

  tr:hover td {
    background: #2a2d2e;
  }

  .editing-row td {
    background: #1a2d1e !important;
  }

  .cell-null {
    color: #888;
    font-style: italic;
  }

  .cell-number {
    color: #b5cea8;
  }

  .cell-string {
    color: #ce9178;
  }

  .cell-bytes {
    color: #888;
  }

  .cell-primary-key {
    background: #252835;
    color: #c586c0;
    font-weight: 500;
  }

  .cell-editable {
    cursor: pointer;
  }

  .cell-editable:hover {
    background: #2a3a2e !important;
  }

  /* 单元格编辑器 */
  .cell-editor {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 0;
  }

  .cell-editor input {
    width: 100%;
    padding: 4px 8px;
    background: #3c3c3c;
    border: 1px solid #007acc;
    border-radius: 3px;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
    min-width: 150px;
  }

  .cell-editor input:focus {
    outline: none;
    border-color: #007acc;
  }

  .cell-editor input.is-null {
    opacity: 0.5;
    font-style: italic;
  }

  .cell-editor-actions {
    display: flex;
    gap: 4px;
  }

  .cell-editor-actions button {
    padding: 2px 6px;
    border: none;
    border-radius: 3px;
    font-size: 11px;
    cursor: pointer;
  }

  .btn-save {
    background: #2da042;
    color: white;
  }

  .btn-save:hover:not(:disabled) {
    background: #238736;
  }

  .btn-cancel {
    background: #f48771;
    color: white;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #d9403a;
  }

  .btn-null {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .btn-null.active {
    background: #007acc;
    color: white;
  }

  .btn-null:hover:not(:disabled) {
    background: #4e4e4e;
  }

  .cell-editor-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 滚动条样式 */
  .results-content::-webkit-scrollbar {
    width: 14px;
    height: 14px;
  }

  .results-content::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .results-content::-webkit-scrollbar-thumb {
    background: #424242;
    border-radius: 7px;
    border: 3px solid #1e1e1e;
  }

  .results-content::-webkit-scrollbar-thumb:hover {
    background: #4e4e4e;
  }
</style>
