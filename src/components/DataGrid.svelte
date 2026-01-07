<script>
  import { invoke } from '@tauri-apps/api/core';

  export let connection = null;
  export let tableName = '';
  export let onRefresh = () => {};

  // 数据状态
  let data = {
    columns: [],
    rows: [],
    totalRows: 0
  };
  let tableSchema = null; // 表结构信息
  let loading = false;
  let error = null;
  let isEmptyTable = false; // 是否为空表

  // 分页状态
  let currentPage = 1;
  let pageSize = 50;
  let totalPages = 1;

  // 编辑状态
  let editingCell = null;  // { rowIndex, colIndex, value, isNew }
  let newRows = [];  // 新增的行数据
  let selectedRows = new Set();
  let updateMessage = null;

  // 筛选状态
  let filterText = '';
  let filterColumn = 'all';

  // 主键列
  $: primaryKeyColumn = getPrimaryKeyColumn();

  function getPrimaryKeyColumn() {
    if (!data.columns.length) return 0;
    const idCol = data.columns.findIndex(c =>
      c.name.toLowerCase() === 'id' ||
      c.name.toLowerCase() === '_id' ||
      c.name.endsWith('_id')
    );
    return idCol >= 0 ? idCol : 0;
  }

  // 检查列是否是主键
  function isPrimaryKeyColumn(colName) {
    if (!tableSchema || !tableSchema.indexes) return false;
    const primaryIndex = tableSchema.indexes.find(idx => idx.name === 'PRIMARY');
    if (!primaryIndex) return false;
    return primaryIndex.columns.includes(colName);
  }

  // 加载数据
  async function loadData() {
    if (!connection || !tableName) return;

    loading = true;
    error = null;
    isEmptyTable = false;

    try {
      const [database, table] = tableName.split('.');
      const offset = (currentPage - 1) * pageSize;

      let sql = `SELECT * FROM \`${database}\`.\`${table}\``;

      // 添加筛选
      if (filterText && filterColumn !== 'all') {
        sql += ` WHERE \`${filterColumn}\` LIKE '%${filterText}%'`;
      } else if (filterText) {
        // 多列筛选
        const conditions = data.columns.map(col =>
          `\`${col.name}\` LIKE '%${filterText}%'`
        ).join(' OR ');
        sql += ` WHERE ${conditions}`;
      }

      sql += ` LIMIT ${pageSize} OFFSET ${offset}`;

      const result = await invoke('query_execute', {
        connection,
        sql,
        maxRows: pageSize
      });

      if (result.sets && result.sets[0]) {
        const set = result.sets[0];
        data.columns = set.columns || [];
        data.rows = getAllRowsFromSet(set);
        data.totalRows = set.meta?.affectedRows || data.rows.length;
        totalPages = Math.ceil(data.totalRows / pageSize);

        // 如果没有数据，获取表结构
        if (data.rows.length === 0) {
          isEmptyTable = true;
          await loadTableSchema(database, table);
        }
      }
    } catch (err) {
      error = String(err);
      console.error('Load data error:', err);
    } finally {
      loading = false;
    }
  }

  // 加载表结构
  async function loadTableSchema(database, table) {
    try {
      const schema = await invoke('meta_get_table_schema', {
        connection,
        database,
        table
      });
      tableSchema = schema;
    } catch (err) {
      console.error('Load schema error:', err);
    }
  }

  function getAllRowsFromSet(set) {
    let allRows = [];
    if (set.chunks) {
      for (const chunk of set.chunks) {
        allRows = allRows.concat(chunk.rows || []);
      }
    }
    return allRows;
  }

  // 当表名或连接变化时重新加载数据
  $: if (connection && tableName) {
    currentPage = 1;
    loadData();
  }

  // 分页
  function nextPage() {
    if (currentPage < totalPages) {
      currentPage++;
      loadData();
    }
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      loadData();
    }
  }

  function gotoPage(page) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
      loadData();
    }
  }

  // 筛选
  function applyFilter() {
    currentPage = 1;
    loadData();
  }

  function clearFilter() {
    filterText = '';
    filterColumn = 'all';
    currentPage = 1;
    loadData();
  }

  // 刷新
  async function refresh() {
    await loadData();
    onRefresh();
  }

  // 新增行
  function addRow() {
    const newRow = new Array(data.columns.length).fill(null);
    newRow[primaryKeyColumn] = ''; // 主键设为空
    newRows = [...newRows, { data: newRow, index: data.rows.length + newRows.length }];
    editingCell = {
      rowIndex: data.rows.length + newRows.length - 1,
      colIndex: 0,
      value: '',
      isNew: true
    };
  }

  // 删除选中行
  async function deleteSelectedRows() {
    if (selectedRows.size === 0) return;

    if (!confirm(`确定要删除选中的 ${selectedRows.size} 行数据吗？`)) {
      return;
    }

    loading = true;
    try {
      const [database, table] = tableName.split('.');
      const pkCol = data.columns[primaryKeyColumn].name;

      for (const rowIndex of selectedRows) {
        const row = data.rows[rowIndex];
        const pkValue = row[primaryKeyColumn];

        const sql = `DELETE FROM \`${database}\`.\`${table}\` WHERE \`${pkCol}\` = ${quoteValue(String(pkValue))} LIMIT 1`;
        await invoke('query_execute', { connection, sql, maxRows: 0 });
      }

      selectedRows.clear();
      await refresh();
      updateMessage = { success: true, text: `成功删除 ${selectedRows.size} 行` };
    } catch (err) {
      updateMessage = { success: false, text: '删除失败: ' + err };
    } finally {
      loading = false;
    }
  }

  function quoteValue(value) {
    if (value === null || value === '') return 'NULL';
    if (value.parseNumeric?.() !== undefined) return value;
    return `'${value.replace(/'/g, "''").replace(/\\/g, '\\\\')}'`;
  }

  // 编辑单元格
  function startEdit(rowIndex, colIndex, currentValue) {
    const isNewRow = rowIndex >= data.rows.length;
    editingCell = {
      rowIndex,
      colIndex,
      value: currentValue === null ? '' : String(currentValue),
      isNew: isNewRow
    };
  }

  function cancelEdit() {
    editingCell = null;
    // 如果是新增行取消编辑，移除该行
    if (editingCell?.isNew) {
      newRows = newRows.filter(r => r.index !== editingCell.rowIndex);
    }
  }

  async function saveEdit() {
    if (!editingCell) return;

    const isNewRow = editingCell.isNew;
    const { rowIndex, colIndex, value } = editingCell;

    if (isNewRow) {
      // 新增行，只更新内存中的数据
      const newRow = newRows.find(r => r.index === rowIndex);
      if (newRow) {
        newRow.data[colIndex] = value || null;
      }

      // 如果是最后一列，保存整行
      if (colIndex === data.columns.length - 1) {
        await insertNewRow(newRow);
        return;
      }

      // 移动到下一列
      editingCell = {
        ...editingCell,
        colIndex: colIndex + 1,
        value: String(newRow.data[colIndex + 1] || '')
      };
    } else {
      // 更新现有行
      await updateCell(rowIndex, colIndex, value);
    }
  }

  async function updateCell(rowIndex, colIndex, value) {
    loading = true;
    try {
      const row = data.rows[rowIndex];
      const pkValue = String(row[primaryKeyColumn]);
      const columnName = data.columns[colIndex].name;
      const [database, table] = tableName.split('.');

      const sql = `
        UPDATE \`${database}\`.\`${table}\`
        SET \`${columnName}\` = ${quoteValue(value)}
        WHERE \`${data.columns[primaryKeyColumn].name}\` = ${quoteValue(pkValue)}
        LIMIT 1
      `;

      await invoke('query_execute', { connection, sql, maxRows: 0 });
      await refresh();
      updateMessage = { success: true, text: '更新成功' };
    } catch (err) {
      updateMessage = { success: false, text: '更新失败: ' + err };
    } finally {
      loading = false;
      editingCell = null;
    }
  }

  async function insertNewRow(newRow) {
    loading = true;
    try {
      const [database, table] = tableName.split('.');
      const columns = data.columns.map(c => `\`${c.name}\``).join(', ');
      const values = newRow.data.map(v => quoteValue(v === '' ? null : v)).join(', ');

      const sql = `INSERT INTO \`${database}\`.\`${table}\` (${columns}) VALUES (${values})`;
      await invoke('query_execute', { connection, sql, maxRows: 0 });

      newRows = newRows.filter(r => r.index !== newRow.index);
      await refresh();
      updateMessage = { success: true, text: '插入成功' };
    } catch (err) {
      updateMessage = { success: false, text: '插入失败: ' + err };
    } finally {
      loading = false;
      editingCell = null;
    }
  }

  function handleKeydown(e) {
    if (e.key === 'Enter') {
      e.preventDefault();
      saveEdit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelEdit();
    } else if (e.key === 'Tab') {
      e.preventDefault();
      const { rowIndex, colIndex } = editingCell;
      const nextCol = colIndex + 1;
      if (nextCol < data.columns.length) {
        editingCell = {
          ...editingCell,
          colIndex: nextCol,
          value: ''
        };
      }
    }
  }

  function toggleRowSelection(rowIndex) {
    if (selectedRows.has(rowIndex)) {
      selectedRows.delete(rowIndex);
    } else {
      selectedRows.add(rowIndex);
    }
    selectedRows = new Set(selectedRows);
  }

  function toggleSelectAll() {
    if (selectedRows.size === data.rows.length) {
      selectedRows.clear();
    } else {
      selectedRows = new Set(data.rows.map((_, i) => i));
    }
  }

  // 获取所有行（包括新增的）
  $: allRows = [...data.rows, ...newRows.map(r => r.data)];

  function formatCellValue(value) {
    if (value === null) return '<span class="null-value">NULL</span>';
    if (typeof value === 'object') return '[Binary]';
    return String(value);
  }
</script>

<div class="datagrid-container">
  <!-- 工具栏 -->
  <div class="datagrid-toolbar">
    <div class="toolbar-left">
      <button class="btn-toolbar" on:click={refresh} disabled={loading} title="刷新">
        &#8635;
      </button>
      <button class="btn-toolbar btn-add" on:click={addRow} title="新增行">
        + 新增
      </button>
      <button
        class="btn-toolbar btn-delete"
        on:click={deleteSelectedRows}
        disabled={selectedRows.size === 0 || loading}
        title="删除选中行"
      >
        - 删除 ({selectedRows.size})
      </button>
    </div>
    <div class="toolbar-right">
      <input
        type="text"
        class="filter-input"
        bind:value={filterText}
        placeholder="筛选..."
        on:keydown={(e) => e.key === 'Enter' && applyFilter()}
      />
      <select class="filter-column" bind:value={filterColumn}>
        <option value="all">所有列</option>
        {#each data.columns as col}
          <option value={col.name}>{col.name}</option>
        {/each}
      </select>
      <button class="btn-filter" on:click={applyFilter}>筛选</button>
      {#if filterText}
        <button class="btn-clear" on:click={clearFilter}>清除</button>
      {/if}
    </div>
  </div>

  <!-- 消息提示 -->
  {#if updateMessage}
    <div class="message-banner" class:success={updateMessage.success} class:error={!updateMessage.success}>
      {updateMessage.text}
      <button on:click={() => updateMessage = null}>&times;</button>
    </div>
  {/if}

  <!-- 表格 -->
  <div class="datagrid-table-wrapper">
    {#if loading}
      <div class="datagrid-loading">加载中...</div>
    {:else if error}
      <div class="datagrid-error">{error}</div>
    {:else if data.columns.length === 0}
      <div class="datagrid-empty">无数据</div>
    {:else}
      <div class="datagrid-table">
        <table>
          <thead>
            <tr>
              <th class="row-num-header">#</th>
              <th class="checkbox-header">
                <input
                  type="checkbox"
                  checked={selectedRows.size === data.rows.length && data.rows.length > 0}
                  on:change={toggleSelectAll}
                />
              </th>
              {#each data.columns as col, i}
                <th class:primary-key={i === primaryKeyColumn}>
                  <span class="col-name">{col.name}</span>
                  <span class="col-type">{(col.type || col.column_type || '').split('::').pop()}</span>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#if data.rows.length === 0}
              <!-- 空表 - 显示占位行 -->
              <tr class="empty-row">
                <td colspan="{data.columns.length + 2}">
                  <div class="empty-table-message">
                    <span class="empty-icon">📭</span>
                    <span>此表当前没有数据</span>
                    <span class="table-name-hint">{tableName}</span>
                  </div>
                </td>
              </tr>
            {:else}
              {#each allRows as row, rowIndex}
                {@const isEditing = editingCell && editingCell.rowIndex === rowIndex}
                {@const isSelected = selectedRows.has(rowIndex)}
                {@const isNewRow = rowIndex >= data.rows.length}
                <tr
                  class:editing-row={isEditing}
                  class:selected-row={isSelected}
                  class:new-row={isNewRow}
                >
                  <td class="row-num">{rowIndex + 1}</td>
                  <td class="row-checkbox">
                    {#if !isNewRow}
                      <input
                        type="checkbox"
                        checked={isSelected}
                        on:change={() => toggleRowSelection(rowIndex)}
                      />
                    {/if}
                  </td>
                  {#each data.columns as col, colIndex}
                    {@const isEditingThisCell = isEditing && editingCell.colIndex === colIndex}
                    {@const isPrimaryKey = colIndex === primaryKeyColumn}
                    <td
                      class="datagrid-cell"
                      class:cell-primary-key={isPrimaryKey}
                      class:cell-editable={!isPrimaryKey}
                      class:cell-editing={isEditingThisCell}
                      on:dblclick={() => !isPrimaryKey && startEdit(rowIndex, colIndex, row[colIndex])}
                    >
                      {#if isEditingThisCell}
                        <input
                          type="text"
                          class="cell-input"
                          bind:value={editingCell.value}
                          on:keydown={handleKeydown}
                          autofocus
                        />
                      {:else}
                        {@html formatCellValue(row[colIndex])}
                      {/if}
                    </td>
                  {/each}
                  </tr>
                {/each}
            {/if}
          </tbody>
        </table>
      </div>

      <!-- 分页 -->
      <div class="datagrid-pagination">
        <span class="pagination-info">
          {data.rows.length === 0 ? '空表' : `第 ${currentPage} / ${totalPages || 1} 页，共 ${data.totalRows} 行`}
        </span>
        <div class="pagination-controls">
          <button on:click={prevPage} disabled={currentPage <= 1 || loading}>&lt;</button>
          <input
            type="number"
            min="1"
            max={totalPages}
            bind:value={currentPage}
            on:change={() => gotoPage(currentPage)}
            class="page-input"
          />
          <button on:click={nextPage} disabled={currentPage >= totalPages || loading}>&gt;</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .datagrid-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
  }

  .datagrid-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
    flex-wrap: wrap;
    gap: 8px;
  }

  .toolbar-left, .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .btn-toolbar {
    padding: 5px 10px;
    background: #3e3e3e;
    border: 1px solid #4e4e4e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .btn-toolbar:hover:not(:disabled) {
    background: #4e4e4e;
  }

  .btn-toolbar:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-add {
    background: #2da042;
    border-color: #238736;
  }

  .btn-add:hover:not(:disabled) {
    background: #238736;
  }

  .btn-delete {
    background: #f48771;
    border-color: #d9403a;
  }

  .btn-delete:hover:not(:disabled) {
    background: #d9403a;
  }

  .filter-input {
    padding: 5px 8px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    width: 150px;
  }

  .filter-input:focus {
    outline: none;
    border-color: #007acc;
  }

  .filter-column {
    padding: 5px 8px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
  }

  .btn-filter, .btn-clear {
    padding: 5px 10px;
    background: #3e3e3e;
    border: 1px solid #4e4e4e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    cursor: pointer;
  }

  .btn-filter:hover, .btn-clear:hover {
    background: #4e4e4e;
  }

  .message-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    font-size: 12px;
  }

  .message-banner.success {
    background: #1e3a1e;
    color: #4ec9b0;
  }

  .message-banner.error {
    background: #3c1f1e;
    color: #f48771;
  }

  .message-banner button {
    background: none;
    border: none;
    color: inherit;
    font-size: 16px;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .datagrid-table-wrapper {
    flex: 1;
    overflow: auto;
    position: relative;
  }

  .datagrid-loading, .datagrid-error, .datagrid-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: #888;
    font-size: 14px;
  }

  .datagrid-error {
    color: #f48771;
  }

  .datagrid-table {
    min-height: 100%;
  }

  .datagrid-table table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .datagrid-table thead {
    position: sticky;
    top: 0;
    background: #2d2d2d;
    z-index: 2;
  }

  .datagrid-table th {
    padding: 6px 8px;
    border-right: 1px solid #3e3e3e;
    border-bottom: 1px solid #3e3e3e;
    text-align: left;
    font-weight: 500;
    white-space: nowrap;
    background: #2d2d2d;
  }

  .datagrid-table th.row-num-header,
  .datagrid-table th.checkbox-header {
    width: 40px;
    text-align: center;
  }

  .datagrid-table th.primary-key {
    background: #2a2d3e;
    color: #c586c0;
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

  .datagrid-table td {
    padding: 4px 8px;
    border-right: 1px solid #2d2d2d;
    border-bottom: 1px solid #2d2d2d;
    white-space: nowrap;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .datagrid-table tr:hover td {
    background: #2a2d2e;
  }

  .datagrid-table tr.selected-row td {
    background: #1a3a2e;
  }

  .datagrid-table tr.editing-row td {
    background: #1a2d1e;
  }

  .datagrid-table tr.new-row td {
    background: #1e2d3e;
  }

  .datagrid-table tr.empty-row td {
    background: transparent;
  }

  .empty-table-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: #888;
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 16px;
    opacity: 0.5;
  }

  .empty-table-message > span:nth-child(2) {
    font-size: 16px;
    font-weight: 500;
  }

  .table-name-hint {
    font-size: 12px;
    color: #666;
    margin-top: 8px;
  }

  .row-num {
    text-align: center;
    color: #666;
    font-size: 11px;
    width: 40px;
  }

  .row-checkbox {
    text-align: center;
    width: 40px;
  }

  .datagrid-cell {
    cursor: default;
  }

  .datagrid-cell.cell-primary-key {
    background: #252835;
    color: #c586c0;
    font-weight: 500;
  }

  .datagrid-cell.cell-editable {
    cursor: cell;
  }

  .datagrid-cell.cell-editable:hover {
    background: #2a3a2e;
  }

  .datagrid-cell.cell-editing {
    padding: 0;
  }

  .cell-input {
    width: 100%;
    padding: 4px 6px;
    background: #3c3c3c;
    border: 1px solid #007acc;
    border-radius: 3px;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
  }

  .cell-input:focus {
    outline: none;
  }

  :global(.null-value) {
    color: #888;
    font-style: italic;
  }

  .datagrid-pagination {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: #2d2d2d;
    border-top: 1px solid #3e3e3e;
  }

  .pagination-info {
    font-size: 12px;
    color: #888;
  }

  .pagination-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pagination-controls button {
    padding: 4px 10px;
    background: #3e3e3e;
    border: 1px solid #4e4e4e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    cursor: pointer;
  }

  .pagination-controls button:hover:not(:disabled) {
    background: #4e4e4e;
  }

  .pagination-controls button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-input {
    width: 50px;
    padding: 4px 6px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    text-align: center;
  }

  .page-input:focus {
    outline: none;
    border-color: #007acc;
  }

  /* 空表结构视图 */
  .empty-table-view {
    padding: 16px;
  }

  .empty-table-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    margin-bottom: 16px;
    background: #2d2d2d;
    border-radius: 4px;
    border-left: 4px solid #007acc;
  }

  .empty-table-title {
    font-size: 14px;
    font-weight: 600;
    color: #d4d4d4;
  }

  .empty-table-note {
    font-size: 12px;
    color: #888;
  }

  .empty-table-view table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .empty-table-view thead {
    position: sticky;
    top: 0;
    background: #2d2d2d;
    z-index: 2;
  }

  .empty-table-view th {
    padding: 10px 12px;
    border-right: 1px solid #3e3e3e;
    border-bottom: 1px solid #3e3e3e;
    text-align: left;
    font-weight: 500;
    background: #252526;
    color: #888;
    font-size: 11px;
    text-transform: uppercase;
  }

  .empty-table-view th:last-child {
    border-right: none;
  }

  .empty-table-view td {
    padding: 8px 12px;
    border-right: 1px solid #2d2d2d;
    border-bottom: 1px solid #2d2d2d;
  }

  .empty-table-view td:last-child {
    border-right: none;
  }

  .empty-table-view tbody tr:hover td {
    background: #2a2d2e;
  }

  .col-name-cell {
    font-weight: 500;
    color: #d4d4d4;
  }

  .key-icon {
    margin-right: 6px;
  }

  .col-type-cell {
    color: #4ec9b0;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 12px;
  }

  .col-nullable-cell {
    color: #888;
    text-align: center;
    width: 80px;
  }

  .col-nullable-cell:not(:empty) {
    font-weight: 500;
  }

  .col-key-cell {
    color: #c586c0;
    font-weight: 500;
    width: 80px;
  }

  .col-default-cell {
    color: #ce9178;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 12px;
  }

  .col-extra-cell {
    color: #888;
    font-size: 11px;
  }

  .schema-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: #888;
    font-size: 14px;
  }
</style>
