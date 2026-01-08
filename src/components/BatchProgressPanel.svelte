<script>
  export let statements = [];
  export let currentIndex = 0;
  export let results = [];
  export let error = null;
  export let show = false;

  $: completedCount = results.length;
  $: totalCount = statements.length;
  $: progress = totalCount > 0 ? (completedCount / totalCount) * 100 : 0;
  $: hasError = error !== null;

  function getStatementType(sql) {
    const trimmed = sql.trim().toUpperCase();
    if (trimmed.startsWith('SELECT')) return 'select';
    if (trimmed.startsWith('INSERT')) return 'insert';
    if (trimmed.startsWith('UPDATE')) return 'update';
    if (trimmed.startsWith('DELETE')) return 'delete';
    if (trimmed.startsWith('CREATE')) return 'create';
    if (trimmed.startsWith('ALTER')) return 'alter';
    if (trimmed.startsWith('DROP')) return 'drop';
    if (trimmed.startsWith('START TRANSACTION') || trimmed.startsWith('COMMIT') || trimmed.startsWith('ROLLBACK')) return 'transaction';
    return 'other';
  }

  function getStatementIcon(type) {
    switch (type) {
      case 'select': return '🔍';
      case 'insert': return '➕';
      case 'update': return '✏️';
      case 'delete': return '🗑️';
      case 'create': return '🆕';
      case 'alter': return '🔧';
      case 'drop': return '💣';
      case 'transaction': return '🔒';
      default: return '📄';
    }
  }

  function getStatementPreview(sql) {
    return sql.slice(0, 60) + (sql.length > 60 ? '...' : '');
  }
</script>

{#if show}
  <div class="batch-progress-overlay">
    <div class="batch-progress-panel">
      <div class="batch-header">
        <h3>⚡ 批量执行进度</h3>
        <button class="btn-close" on:click={() => show = false}>&times;</button>
      </div>

      <div class="batch-progress-bar">
        <div class="progress-track">
          <div class="progress-fill" style="width: {progress}%"></div>
        </div>
        <div class="progress-text">
          {completedCount} / {totalCount} 语句执行完成
          {#if hasError}
            <span class="error-badge">有错误</span>
          {/if}
        </div>
      </div>

      <div class="batch-stats">
        <div class="stat-item">
          <span class="stat-label">总计</span>
          <span class="stat-value">{totalCount}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">成功</span>
          <span class="stat-value success">{results.filter(r => r.success !== false).length}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">失败</span>
          <span class="stat-value error">{results.filter(r => r.success === false).length}</span>
        </div>
      </div>

      <div class="batch-statements">
        {#each statements as stmt, idx}
          {@const result = results[idx]}
          {@const isCurrent = idx === currentIndex}
          {@const type = getStatementType(stmt)}
          {@const icon = getStatementIcon(type)}
          {@const hasResult = result !== undefined}
          {@const isSuccess = hasResult && result.success !== false}
          {@const isFailed = hasResult && result.success === false}

          <div
            class="statement-item"
            class:current={isCurrent}
            class:success={isSuccess}
            class:failed={isFailed}
            class:pending={!hasResult && !isCurrent}
          >
            <div class="statement-number">{idx + 1}</div>
            <div class="statement-icon">{icon}</div>
            <div class="statement-content">
              <div class="statement-preview" title={stmt}>{getStatementPreview(stmt)}</div>
              {#if isCurrent}
                <div class="statement-status">执行中...</div>
              {:else if isSuccess}
                <div class="statement-status success">
                  ✓ {result.rows !== undefined ? `${result.rows} 行` : '完成'}
                  {result.elapsedMs ? `(${result.elapsedMs}ms)` : ''}
                </div>
              {:else if isFailed}
                <div class="statement-status error">✗ {result.error || '失败'}</div>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      {#if error}
        <div class="batch-error">
          <div class="error-header">错误详情</div>
          <pre class="error-message">{error}</pre>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .batch-progress-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .batch-progress-panel {
    background: #252526;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    width: 90vw;
    max-width: 700px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .batch-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #3e3e3e;
  }

  .batch-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
  }

  .btn-close {
    background: none;
    border: none;
    color: #888;
    font-size: 18px;
    cursor: pointer;
    padding: 0;
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-close:hover {
    color: #d4d4d4;
  }

  .batch-progress-bar {
    padding: 16px;
    border-bottom: 1px solid #3e3e3e;
  }

  .progress-track {
    height: 8px;
    background: #3e3e3e;
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #007acc, #00a8ff);
    transition: width 0.3s ease;
  }

  .progress-text {
    margin-top: 8px;
    font-size: 12px;
    color: #d4d4d4;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .error-badge {
    background: #f48771;
    color: #1e1e1e;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
  }

  .batch-stats {
    display: flex;
    padding: 12px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
    gap: 24px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .stat-label {
    font-size: 10px;
    color: #888;
    text-transform: uppercase;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 600;
  }

  .stat-value.success {
    color: #4ec9b0;
  }

  .stat-value.error {
    color: #f48771;
  }

  .batch-statements {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .statement-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 4px;
    margin-bottom: 4px;
    transition: background 0.15s;
  }

  .statement-item.current {
    background: #1e3a5e;
    border-left: 3px solid #007acc;
  }

  .statement-item.success {
    background: rgba(78, 201, 176, 0.1);
  }

  .statement-item.failed {
    background: rgba(244, 135, 113, 0.15);
  }

  .statement-item.pending {
    opacity: 0.6;
  }

  .statement-number {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #3e3e3e;
    border-radius: 50%;
    font-size: 10px;
    font-weight: 600;
    color: #888;
  }

  .statement-item.current .statement-number {
    background: #007acc;
    color: white;
  }

  .statement-item.success .statement-number {
    background: #4ec9b0;
    color: #1e1e1e;
  }

  .statement-item.failed .statement-number {
    background: #f48771;
    color: #1e1e1e;
  }

  .statement-icon {
    font-size: 14px;
  }

  .statement-content {
    flex: 1;
    min-width: 0;
  }

  .statement-preview {
    font-family: 'SF Mono', Monaco, monospace;
    font-size: 11px;
    color: #d4d4d4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .statement-status {
    font-size: 10px;
    margin-top: 2px;
  }

  .statement-status.success {
    color: #4ec9b0;
  }

  .statement-status.error {
    color: #f48771;
  }

  .batch-error {
    padding: 12px 16px;
    border-top: 1px solid #3e3e3e;
    background: #3c1f1e;
  }

  .error-header {
    font-size: 11px;
    font-weight: 600;
    color: #f48771;
    margin-bottom: 8px;
  }

  .error-message {
    margin: 0;
    font-size: 11px;
    color: #f48771;
    white-space: pre-wrap;
    overflow: auto;
    max-height: 100px;
  }

  .batch-statements::-webkit-scrollbar {
    width: 8px;
  }

  .batch-statements::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .batch-statements::-webkit-scrollbar-thumb {
    background: #424242;
    border-radius: 4px;
  }

  .batch-statements::-webkit-scrollbar-thumb:hover {
    background: #4e4e4e;
  }
</style>
