<script>
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';

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

  // 删除确认对话框状态
  let showDeleteConfirm = false;

  // 筛选状态
  let filterText = '';
  let filterColumn = 'all';

  // 主键列
  $: singlePrimaryKeyName = getSinglePrimaryKeyName();
  $: primaryKeyColumn = getPrimaryKeyColumn();

  function getSinglePrimaryKeyName() {
    const primaryIndex = tableSchema?.indexes?.find((index) => index.name === 'PRIMARY');
    if (!primaryIndex || primaryIndex.columns.length !== 1) {
      return null;
    }
    return primaryIndex.columns[0];
  }

  function getPrimaryKeyColumn() {
    if (!data.columns.length || !singlePrimaryKeyName) return null;
    const primaryIndex = data.columns.findIndex((column) => column.name === singlePrimaryKeyName);
    return primaryIndex >= 0 ? primaryIndex : null;
  }

  // 检查列是否是主键
  function isPrimaryKeyColumn(colName) {
    if (!tableSchema || !tableSchema.indexes) return false;
    const primaryIndex = tableSchema.indexes.find(idx => idx.name === 'PRIMARY');
    if (!primaryIndex) return false;
    return primaryIndex.columns.includes(colName);
  }

  function supportsRowMutation() {
    return Boolean(singlePrimaryKeyName && primaryKeyColumn !== null);
  }

  function escapeSqlString(value) {
    return String(value)
      .replace(/\\/g, '\\\\')
      .replace(/'/g, "''");
  }

  function buildWhereClause() {
    const escapedFilter = escapeSqlString(filterText);

    if (filterText && filterColumn !== 'all') {
      return ` WHERE \`${filterColumn}\` LIKE '%${escapedFilter}%'`;
    }

    if (filterText && data.columns.length > 0) {
      const conditions = data.columns.map(col =>
        `\`${col.name}\` LIKE '%${escapedFilter}%'`
      ).join(' OR ');
      return conditions ? ` WHERE ${conditions}` : '';
    }

    return '';
  }

  async function loadTotalCount(database, table, whereClause) {
    const countSql = `SELECT COUNT(*) AS total_count FROM \`${database}\`.\`${table}\`${whereClause}`;
    const result = await invoke('query_execute', {
      connection,
      sql: countSql,
      maxRows: 1
    });

    const set = result?.sets?.[0];
    const row = set?.chunks?.[0]?.rows?.[0];
    const raw = row?.[0];

    if (typeof raw === 'number') return raw;
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : 0;
  }

  // 加载数据
  async function loadData() {
    if (!connection || !tableName) return;

    loading = true;
    error = null;
    isEmptyTable = false;
    editingCell = null;
    selectedRows = new Set();
    newRows = [];

    try {
      const [database, table] = tableName.split('.');
      const offset = (currentPage - 1) * pageSize;

      // 总是加载表结构，以便获取主键和自增信息
      await loadTableSchema(database, table);

      const whereClause = buildWhereClause();
      const totalRows = await loadTotalCount(database, table, whereClause);

      let sql = `SELECT * FROM \`${database}\`.\`${table}\`${whereClause}`;
      sql += ` LIMIT ${pageSize} OFFSET ${offset}`;

      const result = await invoke('query_execute', {
        connection,
        sql,
        maxRows: pageSize
      });

      if (result.sets && result.sets[0]) {
        const set = result.sets[0];
        const rows = getAllRowsFromSet(set);
        // 触发响应式更新
        data = {
          columns: set.columns || [],
          rows: rows,
          totalRows
        };
        totalPages = Math.max(1, Math.ceil(data.totalRows / pageSize));

        // 如果没有数据，标记为空表
        if (rows.length === 0) {
          isEmptyTable = true;
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
    if (data.columns.length === 0) {
      updateMessage = { success: false, text: '无法新增：未加载表结构' };
      return;
    }

    const newRow = new Array(data.columns.length).fill(null);
    if (primaryKeyColumn !== null) {
      newRow[primaryKeyColumn] = '';
    }
    const newRowIndex = data.rows.length + newRows.length;

    // 触发响应式更新
    newRows = [...newRows, { data: newRow, index: newRowIndex }];
    editingCell = {
      rowIndex: newRowIndex,
      colIndex: 0,
      value: '',
      isNew: true
    };
  }

  // 删除选中行 - 显示确认对话框
  function deleteSelectedRows() {
    if (selectedRows.size === 0) return;
    if (!supportsRowMutation()) {
      updateMessage = { success: false, text: '当前表缺少单列主键，暂不支持网格删除' };
      return;
    }
    showDeleteConfirm = true;
  }

  // 执行删除
  async function executeDelete() {
    if (!supportsRowMutation()) {
      updateMessage = { success: false, text: '当前表缺少单列主键，暂不支持网格删除' };
      showDeleteConfirm = false;
      return;
    }

    showDeleteConfirm = false;
    loading = true;

    try {
      const [database, table] = tableName.split('.');
      const pkCol = data.columns[primaryKeyColumn].name;
      const count = selectedRows.size;

      for (const rowIndex of selectedRows) {
        const row = data.rows[rowIndex];
        const pkValue = row[primaryKeyColumn];

        const sql = `DELETE FROM \`${database}\`.\`${table}\` WHERE \`${pkCol}\` = ${quoteValue(String(pkValue))} LIMIT 1`;
        await invoke('query_execute', { connection, sql, maxRows: 0 });
      }

      selectedRows.clear();
      await refresh();
      updateMessage = { success: true, text: `成功删除 ${count} 行` };
    } catch (err) {
      updateMessage = { success: false, text: '删除失败: ' + err };
    } finally {
      loading = false;
    }
  }

  // 取消删除
  function cancelDelete() {
    showDeleteConfirm = false;
  }

  function handleDeleteOverlayClick(event) {
    if (event.target === event.currentTarget) {
      cancelDelete();
    }
  }

  function handleDeleteOverlayKeydown(event) {
    if (event.key === 'Escape') {
      cancelDelete();
    }
  }

  function quoteValue(value) {
    if (value === null || value === '') return 'NULL';
    if (typeof value === 'number') return String(value);
    if (/^-?\d+(\.\d+)?$/.test(String(value))) return String(value);
    return `'${value.replace(/'/g, "''").replace(/\\/g, '\\\\')}'`;
  }

  // 编辑单元格
  function startEdit(rowIndex, colIndex, currentValue) {
    if (!supportsRowMutation()) {
      updateMessage = { success: false, text: '当前表缺少单列主键，暂不支持网格编辑' };
      return;
    }

    const isNewRow = rowIndex >= data.rows.length;
    editingCell = {
      rowIndex,
      colIndex,
      value: currentValue === null ? '' : String(currentValue),
      isNew: isNewRow
    };
  }

  function cancelEdit() {
    const currentEdit = editingCell;
    editingCell = null;
    // 如果是新增行取消编辑，移除该行
    if (currentEdit?.isNew) {
      newRows = newRows.filter(r => r.index !== currentEdit.rowIndex);
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
    if (!supportsRowMutation()) {
      updateMessage = { success: false, text: '当前表缺少单列主键，暂不支持网格编辑' };
      editingCell = null;
      return;
    }

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

      // 找出自增主键列，需要从 INSERT 语句中排除
      const autoIncrementCols = [];
      if (tableSchema && tableSchema.columns) {
        tableSchema.columns.forEach(col => {
          if (col.extra && col.extra.includes('auto_increment')) {
            autoIncrementCols.push(col.name);
          }
        });
      }
      // 备用：通过索引检查主键列
      if (autoIncrementCols.length === 0 && tableSchema && tableSchema.indexes) {
        const pkIndex = tableSchema.indexes.find(idx => idx.name === 'PRIMARY');
        if (pkIndex && pkIndex.columns.length === 1) {
          // 单列主键，可能是自增的
          const pkCol = pkIndex.columns[0];
          // 检查列类型是否支持自增
          const pkColDef = tableSchema.columns.find(c => c.name === pkCol);
          if (pkColDef) {
            const type = pkColDef.column_type || pkColDef.type || '';
            if (/int|bigint|smallint|tinyint/i.test(type)) {
              autoIncrementCols.push(pkCol);
            }
          }
        }
      }

      console.log('Auto increment columns:', autoIncrementCols);

      // 构建列列表和值列表（排除自增列）
      const columns = [];
      const values = [];
      for (let i = 0; i < data.columns.length; i++) {
        const col = data.columns[i];
        // 跳过自增主键列
        if (autoIncrementCols.includes(col.name)) {
          console.log(`Skipping auto-increment column: ${col.name}`);
          continue;
        }
        columns.push(`\`${col.name}\``);
        values.push(quoteValue(newRow.data[i] === '' ? null : newRow.data[i]));
      }

      const sql = `INSERT INTO \`${database}\`.\`${table}\` (${columns.join(', ')}) VALUES (${values.join(', ')})`;
      console.log('Insert SQL:', sql);

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
    if (!supportsRowMutation()) return;
    if (selectedRows.has(rowIndex)) {
      selectedRows.delete(rowIndex);
    } else {
      selectedRows.add(rowIndex);
    }
    selectedRows = new Set(selectedRows);
  }

  function toggleSelectAll() {
    if (!supportsRowMutation()) return;
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
    return escapeHtml(String(value));
  }

  function escapeHtml(value) {
    return value
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  // 导出为 CSV
  async function exportCSV() {
    if (data.columns.length === 0) return;

    const rows = allRows;
    const columns = data.columns;
    const [database, table] = tableName.split('.');

    // 构建 CSV 内容
    let csv = '';

    // 表头
    csv += columns.map(col => escapeCSV(col.name)).join(',') + '\n';

    // 数据行
    for (const row of rows) {
      csv += row.map(cell => escapeCSV(cell === null ? 'NULL' : String(cell))).join(',') + '\n';
    }

    await downloadFile(csv, `${table}_export.csv`, 'text/csv', ['csv']);
  }

  // 导出为 JSON
  async function exportJSON() {
    if (data.columns.length === 0) return;

    const rows = allRows;
    const columns = data.columns;
    const [database, table] = tableName.split('.');

    // 构建 JSON 内容
    const data = rows.map(row => {
      const obj = {};
      columns.forEach((col, i) => {
        obj[col.name] = row[i] === null ? null : row[i];
      });
      return obj;
    });

    const json = JSON.stringify(data, null, 2);
    await downloadFile(json, `${table}_export.json`, 'application/json', ['json']);
  }

  // 导出为 SQL INSERT
  async function exportSQL() {
    if (data.columns.length === 0) return;

    const rows = allRows;
    const columns = data.columns;
    const [database, table] = tableName.split('.');

    // 构建 SQL INSERT 语句
    let sql = '';
    sql += `-- Data export from ${database}.${table}\n`;
    sql += `-- Generated at ${new Date().toISOString()}\n\n`;

    const colNames = columns.map(c => `\`${c.name}\``).join(', ');

    for (const row of rows) {
      const values = row.map(v => {
        if (v === null) return 'NULL';
        if (typeof v === 'number') return String(v);
        return `'${String(v).replace(/'/g, "''").replace(/\\/g, '\\\\')}'`;
      }).join(', ');

      sql += `INSERT INTO \`${database}\`.\`${table}\` (${colNames}) VALUES (${values});\n`;
    }

    await downloadFile(sql, `${table}_export.sql`, 'text/plain', ['sql']);
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
  async function downloadFile(content, filename, mimeType, extensions) {
    try {
      const path = await save({
        title: '导出数据',
        defaultPath: filename,
        filters: [
          {
            name: extensions[0].toUpperCase(),
            extensions: extensions
          }
        ]
      });

      if (path) {
        // 使用 Tauri API 写入文件
        try {
          await invoke('fs_write_file', {
            path: path,
            contents: content
          });
          updateMessage = { success: true, text: `导出成功: ${filename}` };
        } catch (err) {
          console.error('Write error:', err);
          // 降级使用浏览器下载
          browserDownload(content, filename, mimeType);
        }
      }
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
    updateMessage = { success: true, text: `导出成功: ${filename}` };
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
        disabled={selectedRows.size === 0 || loading || !supportsRowMutation()}
        title="删除选中行"
      >
        - 删除 ({selectedRows.size})
      </button>
      <div class="toolbar-divider"></div>
      <button class="btn-toolbar btn-export" on:click={exportCSV} disabled={data.columns.length === 0} title="导出为 CSV">
        &#128196; CSV
      </button>
      <button class="btn-toolbar btn-export" on:click={exportJSON} disabled={data.columns.length === 0} title="导出为 JSON">
        &#128193; JSON
      </button>
      <button class="btn-toolbar btn-export" on:click={exportSQL} disabled={data.columns.length === 0} title="导出为 SQL">
        &#128196; SQL
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

  {#if tableSchema && !supportsRowMutation()}
    <div class="readonly-banner">
      当前表未检测到单列主键，网格视图仅支持浏览、筛选、导出和插入；更新与删除已禁用。
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
                  disabled={!supportsRowMutation()}
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
            {#if data.rows.length === 0 && newRows.length === 0}
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
                  <td class="row-checkbox" on:click|stopPropagation>
                    {#if !isNewRow}
                      <input
                        type="checkbox"
                        checked={isSelected}
                        disabled={!supportsRowMutation()}
                        on:click|stopPropagation={() => toggleRowSelection(rowIndex)}
                      />
                    {/if}
                  </td>
                  {#each data.columns as col, colIndex}
                    {@const isEditingThisCell = isEditing && editingCell.colIndex === colIndex}
                    {@const isPrimaryKey = colIndex === primaryKeyColumn}
                    <td
                      class="datagrid-cell"
                      class:cell-primary-key={isPrimaryKey}
                      class:cell-editable={!isPrimaryKey && supportsRowMutation()}
                      class:cell-editing={isEditingThisCell}
                      on:dblclick={() => !isPrimaryKey && supportsRowMutation() && startEdit(rowIndex, colIndex, row[colIndex])}
                    >
                      {#if isEditingThisCell}
                        <input
                          type="text"
                          class="cell-input"
                          bind:value={editingCell.value}
                          on:keydown={handleKeydown}
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

<!-- 删除确认对话框 -->
{#if showDeleteConfirm}
  <div
    class="confirm-dialog-overlay"
    role="button"
    tabindex="0"
    aria-label="关闭删除确认对话框"
    on:click={handleDeleteOverlayClick}
    on:keydown={handleDeleteOverlayKeydown}
  >
    <div class="confirm-dialog" role="dialog" aria-modal="true" aria-label="确认删除数据">
      <div class="confirm-dialog-header">
        <h3>确认删除</h3>
      </div>
      <div class="confirm-dialog-body">
        <p>确定要删除选中的 <strong>{selectedRows.size}</strong> 行数据吗？</p>
        <p class="confirm-warning">此操作不可撤销！</p>
      </div>
      <div class="confirm-dialog-footer">
        <button class="btn-confirm btn-confirm-cancel" on:click={cancelDelete}>取消</button>
        <button class="btn-confirm btn-confirm-delete" on:click={executeDelete}>删除</button>
      </div>
    </div>
  </div>
{/if}

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

  .toolbar-divider {
    width: 1px;
    height: 20px;
    background: #3e3e3e;
    margin: 0 4px;
  }

  .btn-export {
    background: #3e3e3e;
    border-color: #4e4e4e;
  }

  .btn-export:hover:not(:disabled) {
    background: #4e4e4e;
    border-color: #007acc;
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

  .readonly-banner {
    padding: 8px 12px;
    background: #1f364b;
    border-top: 1px solid #2f5f8a;
    border-bottom: 1px solid #2f5f8a;
    color: #9cdcfe;
    font-size: 12px;
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

  /* 删除确认对话框样式 */
  .confirm-dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .confirm-dialog {
    background: #2d2d2d;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    width: 400px;
    max-width: 90vw;
    animation: dialogFadeIn 0.15s ease-out;
  }

  @keyframes dialogFadeIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .confirm-dialog-header {
    padding: 16px;
    border-bottom: 1px solid #3e3e3e;
  }

  .confirm-dialog-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 500;
    color: #f48771;
  }

  .confirm-dialog-body {
    padding: 20px 16px;
  }

  .confirm-dialog-body p {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #d4d4d4;
  }

  .confirm-dialog-body strong {
    color: #f48771;
    font-size: 16px;
  }

  .confirm-warning {
    color: #f48771 !important;
    font-size: 13px !important;
    margin-top: 12px !important;
  }

  .confirm-dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 16px;
    border-top: 1px solid #3e3e3e;
  }

  .btn-confirm {
    padding: 8px 20px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-confirm-cancel {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .btn-confirm-cancel:hover {
    background: #4e4e4e;
  }

  .btn-confirm-delete {
    background: #f48771;
    color: white;
  }

  .btn-confirm-delete:hover {
    background: #d9403a;
  }
</style>
