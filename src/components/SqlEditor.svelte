<script>
  export let connection = null;
  export let onExecute = () => {};
  export let placeholder = '-- 输入 SQL 语句...';

  let sql = '';

  // 获取选中的 SQL
  function getSelectedSql() {
    const textarea = document.getElementById('sql-editor');
    if (!textarea) return sql;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    if (start !== end) {
      return sql.substring(start, end);
    }
    return sql;
  }

  // 执行 SQL
  async function execute() {
    const sqlToExecute = getSelectedSql().trim() || sql.trim();
    console.log('Execute called, sql:', sqlToExecute);
    console.log('Connection:', connection);
    if (!sqlToExecute) {
      console.log('No SQL to execute');
      return;
    }
    if (!connection) {
      alert('请先选择连接');
      return;
    }
    if (onExecute) {
      await onExecute(sqlToExecute);
    }
  }

  // 格式化 SQL
  function format() {
    sql = sql
      .replace(/\bselect\b/gi, 'SELECT\n  ')
      .replace(/\bfrom\b/gi, '\nFROM\n  ')
      .replace(/\bwhere\b/gi, '\nWHERE\n  ')
      .replace(/\border by\b/gi, '\nORDER BY ')
      .replace(/\blimit\b/gi, '\nLIMIT ');
  }

  // 清空
  function clear() {
    sql = '';
  }

  // 快捷键支持
  function handleKeydown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      execute();
    }
  }
</script>

<div class="sql-editor">
  <div class="editor-toolbar">
    <button class="btn-run" on:click={execute} title="执行 SQL (Ctrl+Enter)">
      ▶ 运行
    </button>
    <button class="btn-format" on:click={format} title="格式化 SQL">
      ⟡ 格式化
    </button>
    <button class="btn-clear" on:click={clear} title="清空编辑器">
      清空
    </button>
    <div class="spacer"></div>
    <span class="connection-info">
      {#if connection}
        {connection.name || connection.host}
      {:else}
        未连接
      {/if}
    </span>
  </div>
  <div class="editor-container">
    <textarea
      id="sql-editor"
      bind:value={sql}
      {placeholder}
      on:keydown={handleKeydown}
      spellcheck="false"
    ></textarea>
  </div>
</div>

<style>
  .sql-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
    gap: 8px;
  }

  .editor-toolbar button {
    background: #3e3e3e;
    border: none;
    color: #d4d4d4;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .editor-toolbar button:hover {
    background: #4e4e4e;
  }

  .btn-run {
    background: #2da042 !important;
    color: white !important;
  }

  .btn-run:hover {
    background: #238736 !important;
  }

  .spacer {
    flex: 1;
  }

  .connection-info {
    font-size: 12px;
    color: #888;
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  #sql-editor {
    width: 100%;
    height: 100%;
    background: #1e1e1e;
    color: #d4d4d4;
    border: none;
    padding: 16px;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
    font-size: 14px;
    line-height: 1.6;
    resize: none;
  }

  #sql-editor:focus {
    outline: none;
  }

  #sql-editor::placeholder {
    color: #666;
  }
</style>
