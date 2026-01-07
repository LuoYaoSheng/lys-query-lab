<script>
  export let result = null;
  export let loading = false;
  export let error = null;

  // 判断值的类型
  function getValueType(value) {
    if (value === null) return 'null';
    if (typeof value === 'number') return 'number';
    if (typeof value === 'boolean') return 'boolean';
    if (typeof value === 'string') return 'string';
    if (Array.isArray(value)) return 'bytes';
    return 'unknown';
  }

  // 格式化值显示
  function formatValue(value) {
    if (value === null) return 'NULL';
    if (typeof value === 'number') return value.toString();
    if (typeof value === 'boolean') return value ? 'true' : 'false';
    if (Array.isArray(value)) return `[${value.length} bytes]`;
    return value;
  }

  // 切换结果集
  let activeSetIndex = 0;

  $: activeSet = result?.sets[activeSetIndex] || null;
  $: totalSets = result?.sets.length || 0;
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
      <div class="results-tabs">
        {#each result.sets as set, idx}
          <div
            class="result-tab"
            class:active={idx === activeSetIndex}
            on:click={() => activeSetIndex = idx}
          >
            结果 {idx + 1}
            <span class="rows-count">({set.meta.affectedRows} 行)</span>
          </div>
        {/each}
      </div>
      <div class="results-info">
        <span>耗时: {result.elapsedMs}ms</span>
        <span>查询ID: {result.queryId.slice(0, 8)}</span>
      </div>
    </div>

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
                  {#each activeSet.columns as col}
                    <th>
                      <span class="col-name">{col.name}</span>
                      <span class="col-type">{(col.column_type || col.columnType || '').split('::').pop()}</span>
                    </th>
                  {/each}
                </tr>
              </thead>
              <tbody>
                {#each activeSet.chunks as chunk}
                  {#each chunk.rows as row}
                    <tr>
                      {#each row as cell, i}
                        {@const type = getValueType(cell)}
                        <td class="cell-{type}">
                          {formatValue(cell)}
                        </td>
                      {/each}
                    </tr>
                  {/each}
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
  }

  .results-tabs {
    display: flex;
    gap: 4px;
  }

  .result-tab {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: #3e3e3e;
    border-radius: 4px 4px 0 0;
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
  }

  td:last-child {
    border-right: none;
  }

  tr:hover td {
    background: #2a2d2e;
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
