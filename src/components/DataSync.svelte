<script>
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  import { confirmAction, notifyError, notifyInfo, notifySuccess } from '../lib/notifications';

  export let connection = null;
  export let databases = [];

  const dispatch = createEventDispatcher();

  // 同步配置
  let sourceDatabase = '';
  let targetDatabase = '';
  let syncMode = 'structure'; // 当前仅支持结构对比与结构 SQL 预演

  // 同步结果
  let comparing = false;
  let syncResult = null;
  let syncError = null;

  // 选中的表
  let selectedTables = new Set();

  // 详情面板
  let showDetailPanel = false;
  let detailTable = null;

  // 表差异
  let tableDifferences = [];

  // 执行同步
  let syncing = false;

  // 开始比较
  async function startCompare() {
    if (!sourceDatabase || !targetDatabase) {
      notifyError('请选择源数据库和目标数据库');
      return;
    }

    if (sourceDatabase === targetDatabase) {
      notifyError('源数据库和目标数据库不能相同');
      return;
    }

    comparing = true;
    syncResult = null;
    syncError = null;
    tableDifferences = [];
    selectedTables.clear();

    try {
      // 获取两个数据库的表列表
      const [sourceTables, targetTables] = await Promise.all([
        invoke('meta_list_tables', {
          connection,
          database: sourceDatabase,
          includeViews: false
        }),
        invoke('meta_list_tables', {
          connection,
          database: targetDatabase,
          includeViews: false
        })
      ]);

      // 比较表差异
      const sourceTableNames = new Set(sourceTables.map(t => t.name));
      const targetTableNames = new Set(targetTables.map(t => t.name));

      // 在源中存在但目标中不存在的表（新增）
      const tablesToAdd = sourceTables.filter(t => !targetTableNames.has(t.name));

      // 在目标中存在但源中不存在的表（删除）
      const tablesToRemove = targetTables.filter(t => !sourceTableNames.has(t.name));

      // 两边都存在的表，需要比较结构
      const tablesToCompare = sourceTables.filter(t => targetTableNames.has(t.name));

      const differences = [];

      // 检查共同表的结构差异
      for (const table of tablesToCompare) {
        try {
          const [sourceSchema, targetSchema] = await Promise.all([
            invoke('meta_get_table_schema', {
              connection,
              database: sourceDatabase,
              table: table.name
            }),
            invoke('meta_get_table_schema', {
              connection,
              database: targetDatabase,
              table: table.name
            })
          ]);

          // 比较列差异
          const sourceCols = new Map(sourceSchema.columns.map(c => [c.name, c]));
          const targetCols = new Map(targetSchema.columns.map(c => [c.name, c]));

          const columnsToAdd = [];
          const columnsToRemove = [];
          const columnsToModify = [];

          for (const [name, col] of sourceCols) {
            if (!targetCols.has(name)) {
              columnsToAdd.push(col);
            } else {
              const targetCol = targetCols.get(name);
              // 比较列类型
              const sourceType = (col.column_type || col.type || '').toUpperCase();
              const targetType = (targetCol.column_type || targetCol.type || '').toUpperCase();
              if (sourceType !== targetType) {
                columnsToModify.push({ column: col, oldType: targetType, newType: sourceType });
              }
            }
          }

          for (const [name, col] of targetCols) {
            if (!sourceCols.has(name)) {
              columnsToRemove.push(col);
            }
          }

          // 比较索引差异
          const sourceIndexes = new Set(
            (sourceSchema.indexes || []).map(i => i.name + ':' + i.columns.join(','))
          );
          const targetIndexes = new Set(
            (targetSchema.indexes || []).map(i => i.name + ':' + i.columns.join(','))
          );

          const hasIndexDiff =
            ![...sourceIndexes].every(x => targetIndexes.has(x)) ||
            ![...targetIndexes].every(x => sourceIndexes.has(x));

          if (
            columnsToAdd.length > 0 ||
            columnsToRemove.length > 0 ||
            columnsToModify.length > 0 ||
            hasIndexDiff
          ) {
            differences.push({
              table: table.name,
              status: 'modified',
              columnsToAdd,
              columnsToRemove,
              columnsToModify,
              hasIndexDiff
            });
          }
        } catch (err) {
          console.error(`Failed to compare table ${table.name}:`, err);
        }
      }

      // 添加新增的表
      for (const table of tablesToAdd) {
        differences.push({
          table: table.name,
          status: 'add',
          columnsToAdd: [],
          columnsToRemove: [],
          columnsToModify: [],
          hasIndexDiff: false
        });
      }

      // 添加删除的表
      for (const table of tablesToRemove) {
        differences.push({
          table: table.name,
          status: 'remove',
          columnsToAdd: [],
          columnsToRemove: [],
          columnsToModify: [],
          hasIndexDiff: false
        });
      }

      tableDifferences = differences;

      syncResult = {
        sourceTables: sourceTableNames,
        targetTables: targetTableNames,
        tablesToAdd: tablesToAdd.map(t => t.name),
        tablesToRemove: tablesToRemove.map(t => t.name),
        commonTables: tablesToCompare.map(t => t.name),
        differences
      };

      // 默认选中所有需要同步的表
      for (const diff of differences) {
        if (diff.status !== 'unchanged') {
          selectedTables.add(diff.table);
        }
      }
      selectedTables = new Set(selectedTables);
    } catch (err) {
      syncError = String(err);
    } finally {
      comparing = false;
    }
  }

  // 生成同步 SQL
  function generateSyncSQL() {
    if (!syncResult || selectedTables.size === 0) return '';

    let sql = `-- 数据同步脚本\n`;
    sql += `-- 源数据库: ${sourceDatabase}\n`;
    sql += `-- 目标数据库: ${targetDatabase}\n`;
    sql += `-- 生成时间: ${new Date().toLocaleString()}\n\n`;

    for (const diff of tableDifferences) {
      if (!selectedTables.has(diff.table)) continue;

      if (diff.status === 'add') {
        // 新增表 - 需要 CREATE TABLE
        sql += `-- 表 ${diff.table} 需要手动创建\n`;
        sql += `-- 请从源数据库导出 CREATE TABLE 语句\n\n`;
      } else if (diff.status === 'remove') {
        // 删除表
        if (syncMode === 'structure' || syncMode === 'both') {
          sql += `-- 删除表 ${diff.table}\n`;
          sql += `DROP TABLE IF EXISTS \`${targetDatabase}\`.\`${diff.table}\`;\n\n`;
        }
      } else if (diff.status === 'modified') {
        // 修改表结构
        if (syncMode === 'structure' || syncMode === 'both') {
          if (diff.columnsToAdd.length > 0) {
            sql += `-- 为表 ${diff.table} 添加列\n`;
            sql += `ALTER TABLE \`${targetDatabase}\`.\`${diff.table}\`\n`;
            sql += diff.columnsToAdd.map(c =>
              `  ADD COLUMN \`${c.name}\` ${c.column_type || c.type}${c.nullable === 'NO' ? ' NOT NULL' : ''}${c.default ? ` DEFAULT ${c.default}` : ''}`
            ).join(',\n');
            sql += `;\n\n`;
          }

          if (diff.columnsToRemove.length > 0) {
            sql += `-- 从表 ${diff.table} 删除列\n`;
            sql += `ALTER TABLE \`${targetDatabase}\`.\`${diff.table}\`\n`;
            sql += diff.columnsToRemove.map(c => `  DROP COLUMN \`${c.name}\``).join(',\n');
            sql += `;\n\n`;
          }

          if (diff.columnsToModify.length > 0) {
            sql += `-- 修改表 ${diff.table} 的列类型\n`;
            for (const col of diff.columnsToModify) {
              sql += `ALTER TABLE \`${targetDatabase}\`.\`${diff.table}\`\n`;
              sql += `  MODIFY COLUMN \`${col.column.name}\` ${col.newType};\n\n`;
            }
          }
        }

      }
    }

    return sql;
  }

  // 执行同步
  async function executeSync() {
    const sql = generateSyncSQL();
    if (!sql) {
      notifyInfo('没有选中的表或没有需要同步的内容');
      return;
    }

    const confirmed = await confirmAction({
      title: '确认执行结构变更',
      message: '此操作会直接修改目标数据库结构，且不可撤销。建议先完成 SQL 备份。',
      confirmLabel: '执行变更',
      cancelLabel: '取消',
      tone: 'danger',
    });

    if (!confirmed) {
      return;
    }

    syncing = true;

    try {
      // 按语句分割执行
      const statements = sql
        .split(';')
        .map(s => s.trim())
        .filter(s => s && !s.startsWith('--'))
        .map(s => {
          // 移除行内注释
          return s.replace(/--.*$/gm, '').trim();
        })
        .filter(s => s);

      for (const stmt of statements) {
        await invoke('query_execute', {
          connection,
          sql: stmt,
          maxRows: 0
        });
      }

      notifySuccess('结构变更已执行完成');
      dispatch('syncComplete');
    } catch (err) {
      notifyError('结构变更执行失败: ' + err);
    } finally {
      syncing = false;
    }
  }

  // 复制 SQL 到剪贴板
  function copySQL() {
    const sql = generateSyncSQL();
    if (!sql) return;

    navigator.clipboard.writeText(sql).then(() => {
      notifySuccess('SQL 已复制到剪贴板');
    }).catch(err => {
      notifyError('复制失败: ' + err);
    });
  }

  // 导出 SQL 文件
  async function exportSQL() {
    const sql = generateSyncSQL();
    if (!sql) return;

    const blob = new Blob([sql], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `sync_${sourceDatabase}_to_${targetDatabase}_${Date.now()}.sql`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function toggleTableSelection(tableName) {
    if (selectedTables.has(tableName)) {
      selectedTables.delete(tableName);
    } else {
      selectedTables.add(tableName);
    }
    selectedTables = new Set(selectedTables);
  }

  function toggleSelectAll() {
    if (selectedTables.size === tableDifferences.length) {
      selectedTables.clear();
    } else {
      selectedTables = new Set(tableDifferences.map(d => d.table));
    }
    selectedTables = new Set(selectedTables);
  }

  // 显示表详情
  function showTableDetail(tableName) {
    detailTable = tableDifferences.find(d => d.table === tableName);
    showDetailPanel = true;
  }

  // 获取详情面板的完整数据（包括列详情）
  function getDetailData() {
    if (!detailTable) return null;

    // 构建完整的列对比数据
    const columnsDiff = [];

    // 添加的新列
    for (const col of detailTable.columnsToAdd) {
      columnsDiff.push({
        name: col.name,
        status: 'add',
        source: col,
        target: null
      });
    }

    // 删除的列
    for (const col of detailTable.columnsToRemove) {
      columnsDiff.push({
        name: col.name,
        status: 'remove',
        source: null,
        target: col
      });
    }

    // 修改的列
    for (const col of detailTable.columnsToModify) {
      columnsDiff.push({
        name: col.column.name,
        status: 'modify',
        source: col.column,
        target: { ...col.column, column_type: col.oldType },
        change: `类型: ${col.oldType} → ${col.newType}`
      });
    }

    return {
      table: detailTable.table,
      status: detailTable.status,
      columns: columnsDiff,
      hasIndexDiff: detailTable.hasIndexDiff
    };
  }
</script>

<div class="datasync-container">
  <div class="datasync-header">
    <h2>结构对比</h2>
    <button class="btn-close" on:click={() => dispatch('close')}>&times;</button>
  </div>

  <div class="datasync-body">
    <!-- 配置区域 -->
    <div class="sync-config">
      <div class="mode-note">
        当前页面用于比较两个数据库的表结构差异，并生成/执行结构变更 SQL。
        暂不提供真实数据同步。
      </div>
      <div class="config-row">
        <div class="config-item">
          <label for="sync-source-database">源数据库</label>
          <select id="sync-source-database" bind:value={sourceDatabase}>
            <option value="">-- 选择数据库 --</option>
            {#each databases as db}
              <option value={db}>{db}</option>
            {/each}
          </select>
        </div>

        <div class="config-arrow">→</div>

        <div class="config-item">
          <label for="sync-target-database">目标数据库</label>
          <select id="sync-target-database" bind:value={targetDatabase}>
            <option value="">-- 选择数据库 --</option>
            {#each databases as db}
              <option value={db}>{db}</option>
            {/each}
          </select>
        </div>

        <div class="config-item">
          <label for="sync-mode">同步模式</label>
          <select id="sync-mode" bind:value={syncMode} disabled>
            <option value="structure">仅结构（当前支持）</option>
          </select>
        </div>

        <button class="btn-compare" on:click={startCompare} disabled={comparing}>
          {comparing ? '比较中...' : '开始比较'}
        </button>
      </div>
    </div>

    <!-- 错误显示 -->
    {#if syncError}
      <div class="sync-error">
        <span class="error-icon">⚠️</span>
        <pre>{syncError}</pre>
      </div>
    {/if}

    <!-- 比较结果 -->
    {#if syncResult}
      <div class="sync-result">
        <!-- 统计信息 -->
        <div class="sync-summary">
          <div class="summary-item">
            <span class="summary-label">源表数:</span>
            <span class="summary-value">{syncResult.sourceTables.size}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">目标表数:</span>
            <span class="summary-value">{syncResult.targetTables.size}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">新增表:</span>
            <span class="summary-value add">{syncResult.tablesToAdd.length}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">删除表:</span>
            <span class="summary-value remove">{syncResult.tablesToRemove.length}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">差异表:</span>
            <span class="summary-value modify">{tableDifferences.filter(d => d.status === 'modified').length}</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">已选中:</span>
            <span class="summary-value">{selectedTables.size}</span>
          </div>
        </div>

        <!-- 差异列表 -->
        <div class="diff-list">
          <div class="diff-list-header">
            <label class="select-all">
              <input
                type="checkbox"
                checked={selectedTables.size === tableDifferences.length && tableDifferences.length > 0}
                on:change={toggleSelectAll}
              />
              全选
            </label>
          </div>

          <div class="diff-items">
            {#if tableDifferences.length === 0}
              <div class="diff-empty">没有发现结构差异，两个数据库结构相同</div>
            {:else}
              {#each tableDifferences as diff}
                {@const isSelected = selectedTables.has(diff.table)}
                <div
                  class="diff-item"
                  class:selected={isSelected}
                  class:add={diff.status === 'add'}
                  class:remove={diff.status === 'remove'}
                  class:modify={diff.status === 'modified'}
                >
                  <label class="diff-checkbox">
                    <input
                      type="checkbox"
                      checked={isSelected}
                      on:change={() => toggleTableSelection(diff.table)}
                    />
                  </label>
                  <div class="diff-info" on:click={() => showTableDetail(diff.table)} on:keydown={(e) => e.key === 'Enter' && showTableDetail(diff.table)} role="button" tabindex="0">
                    <div class="diff-table-name">{diff.table}</div>
                    <div class="diff-status">
                      {#if diff.status === 'add'}
                        <span class="status-badge add">新增</span>
                      {:else if diff.status === 'remove'}
                        <span class="status-badge remove">删除</span>
                      {:else}
                        <span class="status-badge modify">有差异</span>
                      {/if}

                      {#if diff.columnsToAdd.length > 0}
                        <span class="diff-detail">+{diff.columnsToAdd.length} 列</span>
                      {/if}
                      {#if diff.columnsToRemove.length > 0}
                        <span class="diff-detail">-{diff.columnsToRemove.length} 列</span>
                      {/if}
                      {#if diff.columnsToModify.length > 0}
                        <span class="diff-detail">~{diff.columnsToModify.length} 列</span>
                      {/if}
                      {#if diff.hasIndexDiff}
                        <span class="diff-detail">索引差异</span>
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- 详情面板 -->
        {#if showDetailPanel && detailTable}
          {@const detailData = getDetailData()}
          <div class="detail-panel">
            <div class="detail-header">
              <div class="detail-title">
                <span>表结构详情: {detailData.table}</span>
                <span class="detail-status-badge {detailData.status}">
                  {detailData.status === 'add' ? '新增表' : detailData.status === 'remove' ? '删除表' : '结构差异'}
                </span>
              </div>
              <button class="btn-close-detail" on:click={() => showDetailPanel = false}>&times;</button>
            </div>

            {#if detailData.status === 'modified' && detailData.columns.length > 0}
              <div class="detail-section">
                <h4>列差异</h4>
                <div class="columns-diff-table">
                  <div class="diff-table-header">
                    <div class="diff-cell col-name">列名</div>
                    <div class="diff-cell col-status">状态</div>
                    <div class="diff-cell col-source">源类型</div>
                    <div class="diff-cell col-target">目标类型</div>
                    <div class="diff-cell col-nullable">可空</div>
                    <div class="diff-cell col-default">默认值</div>
                  </div>
                  {#each detailData.columns as col}
                    <div class="diff-table-row" class:{col.status}={true}>
                      <div class="diff-cell col-name">{col.name}</div>
                      <div class="diff-cell col-status">
                        {#if col.status === 'add'}
                          <span class="status-dot add"></span> 新增
                        {:else if col.status === 'remove'}
                          <span class="status-dot remove"></span> 删除
                        {:else}
                          <span class="status-dot modify"></span> 修改
                        {/if}
                      </div>
                      <div class="diff-cell col-source">
                        {col.source ? (col.source.column_type || col.source.type || '-') : '-'}
                      </div>
                      <div class="diff-cell col-target">
                        {col.target ? (col.target.column_type || col.target.type || '-') : '-'}
                      </div>
                      <div class="diff-cell col-nullable">
                        {#if col.source}
                          {col.source.nullable === 'NO' ? 'NOT NULL' : 'NULL'}
                        {:else}
                          -
                        {/if}
                      </div>
                      <div class="diff-cell col-default">
                        {(col.source?.default || col.target?.default || '-')}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {:else}
              <div class="detail-section">
                {#if detailData.status === 'add'}
                  <p class="detail-info">这是源数据库中的新表，需要创建完整表结构。</p>
                {:else if detailData.status === 'remove'}
                  <p class="detail-info">这是目标数据库中存在但源数据库中不存在的表。</p>
                {:else}
                  <p class="detail-info">没有发现列结构差异。</p>
                {/if}
              </div>
            {/if}

            {#if detailData.hasIndexDiff}
              <div class="detail-section">
                <h4>索引差异</h4>
                <p class="detail-info">检测到索引结构存在差异，请同步后检查索引定义。</p>
              </div>
            {/if}

            <!-- 生成预览 SQL -->
            {#if detailData.status === 'modified'}
              <div class="detail-section">
                <h4>同步 SQL 预览</h4>
                <pre class="sql-preview">{#if detailData.columnsToAdd.length > 0}
-- 为表 {detailData.table} 添加列
ALTER TABLE `{targetDatabase}`.`{detailData.table}`
{detailData.columnsToAdd.map((c, i) =>
  `  ADD COLUMN \`${c.name}\` ${c.column_type || c.type}${c.nullable === 'NO' ? ' NOT NULL' : ''}${c.default ? ` DEFAULT ${c.default}` : ''}${i < detailData.columnsToAdd.length - 1 ? ',' : ';'}`
).join('\n')}
{/if}
{#if detailData.columnsToRemove.length > 0}
-- 从表 {detailData.table} 删除列
ALTER TABLE `{targetDatabase}`.`{detailData.table}`
{detailData.columnsToRemove.map((c, i) =>
  `  DROP COLUMN \`${c.name}\`${i < detailData.columnsToRemove.length - 1 ? ',' : ';'}`
).join('\n')}
{/if}
{#if detailData.columnsToModify.length > 0}
-- 修改表 {detailData.table} 的列类型
{detailData.columnsToModify.map(c =>
  `ALTER TABLE \`{targetDatabase}\`.\`{detailData.table}\`\n  MODIFY COLUMN \`${c.column.name}\` ${c.newType};`
).join('\n')}
{/if}</pre>
              </div>
            {/if}
          </div>
        {/if}

        <!-- 操作按钮 -->
        <div class="sync-actions">
          <button class="btn-action btn-copy" on:click={copySQL} disabled={selectedTables.size === 0}>
            📋 复制 SQL
          </button>
          <button class="btn-action btn-export" on:click={exportSQL} disabled={selectedTables.size === 0}>
            💾 导出 SQL
          </button>
          <button class="btn-action btn-sync" on:click={executeSync} disabled={selectedTables.size === 0 || syncing}>
            {syncing ? '同步中...' : '▶ 执行同步'}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .datasync-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    color: #d4d4d4;
  }

  .datasync-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
  }

  .datasync-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .btn-close {
    background: none;
    border: none;
    color: #888;
    font-size: 24px;
    cursor: pointer;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-close:hover {
    color: #d4d4d4;
  }

  .datasync-body {
    flex: 1;
    padding: 20px;
    overflow-y: auto;
  }

  /* 配置区域 */
  .sync-config {
    background: #252526;
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 20px;
  }

  .config-row {
    display: flex;
    align-items: flex-end;
    gap: 16px;
    flex-wrap: wrap;
  }

  .config-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .config-item label {
    font-size: 12px;
    color: #888;
  }

  .config-item select {
    min-width: 150px;
    padding: 8px 12px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 13px;
  }

  .config-item select:focus {
    outline: none;
    border-color: #007acc;
  }

  .config-arrow {
    font-size: 20px;
    color: #888;
    padding-bottom: 8px;
  }

  .btn-compare {
    padding: 8px 20px;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .btn-compare:hover:not(:disabled) {
    background: #006cbd;
  }

  .btn-compare:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 错误显示 */
  .sync-error {
    background: #3c1f1e;
    border: 1px solid #f48771;
    border-radius: 4px;
    padding: 12px 16px;
    margin-bottom: 20px;
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .error-icon {
    font-size: 18px;
  }

  .sync-error pre {
    margin: 0;
    font-size: 12px;
    white-space: pre-wrap;
  }

  /* 同步结果 */
  .sync-result {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .sync-summary {
    display: flex;
    gap: 20px;
    flex-wrap: wrap;
    background: #252526;
    border-radius: 8px;
    padding: 16px;
  }

  .summary-item {
    display: flex;
    gap: 8px;
    font-size: 13px;
  }

  .summary-label {
    color: #888;
  }

  .summary-value {
    font-weight: 600;
  }

  .summary-value.add {
    color: #4ec9b0;
  }

  .summary-value.remove {
    color: #f48771;
  }

  .summary-value.modify {
    color: #dcdcaa;
  }

  /* 差异列表 */
  .diff-list {
    background: #252526;
    border-radius: 8px;
    overflow: hidden;
  }

  .diff-list-header {
    padding: 12px 16px;
    border-bottom: 1px solid #3e3e3e;
  }

  .select-all {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }

  .diff-items {
    max-height: 400px;
    overflow-y: auto;
  }

  .diff-empty {
    padding: 40px;
    text-align: center;
    color: #888;
  }

  .diff-item {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #2d2d2d;
    cursor: pointer;
  }

  .diff-item:hover {
    background: #2a2d2e;
  }

  .diff-item.selected {
    background: #1a3a2e;
  }

  .diff-item.add {
    border-left: 3px solid #4ec9b0;
  }

  .diff-item.remove {
    border-left: 3px solid #f48771;
  }

  .diff-item.modify {
    border-left: 3px solid #dcdcaa;
  }

  .diff-checkbox {
    margin-right: 12px;
  }

  .diff-checkbox input {
    cursor: pointer;
  }

  .diff-info {
    flex: 1;
    cursor: pointer;
  }

  .diff-table-name {
    font-weight: 600;
    margin-bottom: 4px;
  }

  .diff-table-name::after {
    content: ' ›';
    color: #888;
    font-size: 12px;
    margin-left: 6px;
  }

  .diff-status {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .status-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
  }

  .status-badge.add {
    background: #1e3a2e;
    color: #4ec9b0;
  }

  .status-badge.remove {
    background: #3c1f1e;
    color: #f48771;
  }

  .status-badge.modify {
    background: #3a3a2e;
    color: #dcdcaa;
  }

  .diff-detail {
    font-size: 11px;
    color: #888;
  }

  /* 操作按钮 */
  .sync-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }

  .btn-action {
    padding: 10px 20px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .btn-copy {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .btn-copy:hover:not(:disabled) {
    background: #4e4e4e;
  }

  .btn-export {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .btn-export:hover:not(:disabled) {
    background: #4e4e4e;
  }

  .btn-sync {
    background: #2da042;
    color: white;
  }

  .btn-sync:hover:not(:disabled) {
    background: #238736;
  }

  .btn-action:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* 滚动条 */
  .diff-items::-webkit-scrollbar {
    width: 10px;
  }

  .diff-items::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .diff-items::-webkit-scrollbar-thumb {
    background: #424242;
    border-radius: 5px;
  }

  /* 详情面板 */
  .detail-panel {
    background: #252526;
    border-radius: 8px;
    overflow: hidden;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
  }

  .detail-title {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    font-weight: 600;
  }

  .detail-status-badge {
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
  }

  .detail-status-badge.add {
    background: #1e3a2e;
    color: #4ec9b0;
  }

  .detail-status-badge.remove {
    background: #3c1f1e;
    color: #f48771;
  }

  .detail-status-badge.modified {
    background: #3a3a2e;
    color: #dcdcaa;
  }

  .btn-close-detail {
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

  .btn-close-detail:hover {
    color: #d4d4d4;
  }

  .detail-section {
    padding: 16px;
    border-bottom: 1px solid #2d2d2d;
  }

  .detail-section:last-child {
    border-bottom: none;
  }

  .detail-section h4 {
    margin: 0 0 12px 0;
    font-size: 12px;
    font-weight: 600;
    color: #888;
    text-transform: uppercase;
  }

  .detail-info {
    margin: 0;
    font-size: 13px;
    color: #888;
  }

  /* 列差异表格 */
  .columns-diff-table {
    font-size: 12px;
  }

  .diff-table-header {
    display: grid;
    grid-template-columns: 1.5fr 1fr 2fr 2fr 1fr 1.5fr;
    gap: 8px;
    padding: 8px;
    background: #1e1e1e;
    border-radius: 4px 4px 0 0;
    font-weight: 600;
    color: #888;
  }

  .diff-table-row {
    display: grid;
    grid-template-columns: 1.5fr 1fr 2fr 2fr 1fr 1.5fr;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid #2d2d2d;
  }

  .diff-table-row:last-child {
    border-bottom: none;
  }

  .diff-table-row:hover {
    background: #2a2d2e;
  }

  .diff-table-row.add {
    background: rgba(78, 201, 176, 0.1);
  }

  .diff-table-row.remove {
    background: rgba(244, 135, 113, 0.1);
  }

  .diff-table-row.modify {
    background: rgba(220, 220, 170, 0.1);
  }

  .diff-cell {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-name {
    font-weight: 600;
  }

  .status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 6px;
  }

  .status-dot.add {
    background: #4ec9b0;
  }

  .status-dot.remove {
    background: #f48771;
  }

  .status-dot.modify {
    background: #dcdcaa;
  }

  /* SQL 预览 */
  .sql-preview {
    margin: 0;
    padding: 12px;
    background: #1e1e1e;
    border-radius: 4px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    font-size: 11px;
    color: #d4d4d4;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }

  @media (max-width: 900px) {
    .diff-table-header,
    .diff-table-row {
      grid-template-columns: 1fr 0.8fr 1.5fr 1.5fr 0.8fr 1.2fr;
      gap: 6px;
      padding: 6px;
      font-size: 11px;
    }
  }
</style>
