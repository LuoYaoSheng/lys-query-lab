<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { sql, MySQL, PostgreSQL, StandardSQL } from '@codemirror/lang-sql';
  import { keymap, hoverTooltip } from '@codemirror/view';
  import { indentWithTab } from '@codemirror/commands';
  import { autocompletion } from '@codemirror/autocomplete';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';

  export let connection = null;
  export let onExecute = () => {};
  export let onBatchExecute = () => {};  // 批量执行回调
  export let placeholder = '-- 输入 SQL 语句...';

  let editorContainer;
  let editorView = null;
  let sql = '';
  let showHistory = false;
  let showSnippetDialog = false;
  let batchMode = false;  // 批量执行模式
  let useTransaction = false;  // 使用事务

  // SQL 历史记录
  let sqlHistory = [];
  const MAX_HISTORY = 100;

  // 数据库表名（用于自动补全）
  let tableNames = [];
  let columnNames = new Map(); // table -> [columns]

  // 常用 SQL 代码片段
  const snippets = [
    { name: 'SELECT 基础', sql: 'SELECT * FROM table_name WHERE condition LIMIT 100;' },
    { name: 'SELECT 指定列', sql: 'SELECT col1, col2, col3\nFROM table_name\nWHERE condition\nORDER BY col1 DESC\nLIMIT 100;' },
    { name: 'INSERT', sql: 'INSERT INTO table_name (col1, col2, col3)\nVALUES (val1, val2, val3);' },
    { name: 'INSERT 多行', sql: 'INSERT INTO table_name (col1, col2, col3)\nVALUES\n  (val1, val2, val3),\n  (val4, val5, val6),\n  (val7, val8, val9);' },
    { name: 'UPDATE', sql: 'UPDATE table_name\nSET col1 = val1, col2 = val2\nWHERE condition;' },
    { name: 'UPDATE 多列', sql: 'UPDATE table_name\nSET col1 = val1,\n    col2 = val2,\n    col3 = val3\nWHERE id = 1;' },
    { name: 'DELETE', sql: 'DELETE FROM table_name\nWHERE condition;' },
    { name: 'DELETE 多条件', sql: 'DELETE FROM table_name\nWHERE condition1 AND condition2;' },
    { name: 'CREATE TABLE', sql: 'CREATE TABLE table_name (\n  id INT PRIMARY KEY AUTO_INCREMENT,\n  col1 VARCHAR(255) NOT NULL,\n  col2 INT DEFAULT 0,\n  col3 DECIMAL(10,2),\n  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,\n  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,\n  INDEX idx_col1 (col1),\n  INDEX idx_col2_col3 (col2, col3)\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;' },
    { name: 'CREATE INDEX', sql: 'CREATE INDEX idx_col1 ON table_name (col1);\nCREATE INDEX idx_col2_col3 ON table_name (col2, col3);' },
    { name: 'ALTER TABLE', sql: 'ALTER TABLE table_name\nADD COLUMN new_col VARCHAR(255) AFTER existing_col;' },
    { name: 'DROP TABLE', sql: 'DROP TABLE IF EXISTS table_name;' },
    { name: 'TRUNCATE', sql: 'TRUNCATE TABLE table_name;' },
    { name: 'JOIN', sql: 'SELECT a.*, b.col2, c.col3\nFROM table_a a\nINNER JOIN table_b b ON a.id = b.a_id\nLEFT JOIN table_c c ON a.id = c.a_id\nWHERE a.condition;' },
    { name: 'LEFT JOIN', sql: 'SELECT a.*, b.col2\nFROM table_a a\nLEFT JOIN table_b b ON a.id = b.a_id\nWHERE a.condition;' },
    { name: 'RIGHT JOIN', sql: 'SELECT a.*, b.col2\nFROM table_a a\nRIGHT JOIN table_b b ON a.id = b.a_id\nWHERE a.condition;' },
    { name: 'GROUP BY', sql: 'SELECT col1, COUNT(*) as count, SUM(amount) as total\nFROM table_name\nWHERE condition\nGROUP BY col1\nHAVING count > 1\nORDER BY count DESC;' },
    { name: '子查询', sql: 'SELECT *\nFROM table_name\nWHERE id IN (\n  SELECT id\n  FROM other_table\n  WHERE status = 1\n);' },
    { name: 'EXISTS 子查询', sql: 'SELECT *\nFROM table_a a\nWHERE EXISTS (\n  SELECT 1\n  FROM table_b b\n  WHERE b.a_id = a.id\n);' },
    { name: 'UNION', sql: 'SELECT col1, col2 FROM table_a\nUNION ALL\nSELECT col1, col2 FROM table_b\nORDER BY col1;' },
    { name: '事务', sql: 'START TRANSACTION;\n\n-- 你的 SQL 语句\n\n-- 如果成功\nCOMMIT;\n\n-- 如果失败\n-- ROLLBACK;' },
    { name: '批量插入', sql: 'INSERT INTO table_name (col1, col2, col3)\nVALUES\n  (?, ?, ?),\n  (?, ?, ?),\n  (?, ?, ?);' },
    { name: '创建视图', sql: 'CREATE VIEW view_name AS\nSELECT col1, col2, col3\nFROM table_name\nWHERE condition;' },
    { name: 'WITH 递归', sql: 'WITH RECURSIVE cte_name AS (\n  SELECT base_column\n  FROM table_name\n  WHERE condition\n  \n  UNION ALL\n  \n  SELECT t.column\n  FROM table_name t\n  INNER JOIN cte_name c ON t.parent_id = c.id\n)\nSELECT * FROM cte_name;' }
  ];

  // 加载历史记录
  function loadHistory() {
    try {
      const saved = localStorage.getItem('querylab_sql_history');
      if (saved) {
        sqlHistory = JSON.parse(saved);
      }
    } catch (err) {
      console.error('Failed to load history:', err);
    }
  }

  // 保存历史记录
  function saveToHistory(sqlText) {
    if (!sqlText || !sqlText.trim()) return;

    sqlHistory = sqlHistory.filter(h => h.sql !== sqlText);
    sqlHistory.unshift({
      sql: sqlText,
      timestamp: Date.now(),
      date: new Date().toLocaleString()
    });

    if (sqlHistory.length > MAX_HISTORY) {
      sqlHistory = sqlHistory.slice(0, MAX_HISTORY);
    }

    try {
      localStorage.setItem('querylab_sql_history', JSON.stringify(sqlHistory));
    } catch (err) {
      console.error('Failed to save history:', err);
    }
  }

  function clearHistory() {
    sqlHistory = [];
    localStorage.removeItem('querylab_sql_history');
  }

  // SQL 关键字列表（用于自动补全）
  const sqlKeywords = [
    'SELECT', 'FROM', 'WHERE', 'INSERT', 'INTO', 'VALUES', 'UPDATE', 'SET', 'DELETE',
    'CREATE', 'TABLE', 'INDEX', 'VIEW', 'DROP', 'ALTER', 'ADD', 'COLUMN', 'PRIMARY', 'KEY',
    'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'FULL', 'ON', 'AND', 'OR', 'NOT', 'IN',
    'EXISTS', 'BETWEEN', 'LIKE', 'IS', 'NULL', 'ORDER', 'BY', 'GROUP', 'HAVING', 'LIMIT',
    'OFFSET', 'ASC', 'DESC', 'DISTINCT', 'AS', 'UNION', 'ALL', 'WITH', 'RECURSIVE',
    'CASE', 'WHEN', 'THEN', 'ELSE', 'END', 'IF', 'IFNULL', 'COALESCE', 'NULLIF',
    'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'CONCAT', 'SUBSTRING', 'LENGTH', 'TRIM',
    'DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'NOW', 'CURDATE', 'CURTIME',
    'START', 'TRANSACTION', 'COMMIT', 'ROLLBACK', 'LOCK', 'UNLOCK',
    'GRANT', 'REVOKE', 'SHOW', 'DESCRIBE', 'EXPLAIN', 'USE', 'DATABASE',
    'ENGINE', 'CHARSET', 'COLLATE', 'AUTO_INCREMENT', 'DEFAULT', 'REFERENCES',
    'FOREIGN', 'CONSTRAINT', 'UNIQUE', 'CHECK', 'CASCADE', 'RESTRICT', 'SET', 'NULL'
  ];

  // 数据库类型（用于自动补全）
  const dataTypes = [
    'INT', 'INTEGER', 'BIGINT', 'SMALLINT', 'TINYINT',
    'DECIMAL', 'NUMERIC', 'FLOAT', 'DOUBLE', 'REAL',
    'VARCHAR', 'CHAR', 'TEXT', 'LONGTEXT', 'MEDIUMTEXT', 'TINYTEXT',
    'DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'YEAR',
    'BOOLEAN', 'BOOL', 'ENUM', 'SET',
    'JSON', 'BLOB', 'LONGBLOB', 'MEDIUMBLOB', 'TINYBLOB',
    'BINARY', 'VARBINARY'
  ];

  // 创建自动补全源
  function createCompletionSource() {
    return (context) => {
      const word = context.matchBefore(/\w*/);
      if (!word || word.from === word.to) return null;

      // 获取上下文中的表名
      const beforeCursor = context.state.doc.sliceString(0, word.from);
      const suggestFrom = word.from;

      let options = [];

      // 添加关键字
      options.push(...sqlKeywords.map(k => ({ label: k, type: 'keyword' })));

      // 添加数据类型（在 CREATE 或 ALTER 后）
      if (/CREATE|ALTER|COLUMN/i.test(beforeCursor)) {
        options.push(...dataTypes.map(d => ({ label: d, type: 'type' })));
      }

      // 添加表名
      if (/FROM|JOIN|TABLE|INTO|UPDATE/i.test(beforeCursor)) {
        options.push(...tableNames.map(t => ({ label: t, type: 'table' })));
      }

      // 如果有列名信息，添加列名
      // 这里可以进一步优化，根据当前表来过滤列名

      return {
        from: suggestFrom,
        options: options.map(o => ({ ...o, boost: 1 }))
      };
    };
  }

  // 创建编辑器
  function createEditor() {
    if (editorView) {
      editorView.destroy();
    }

    // 根据连接类型选择 SQL 方言
    let sqlDialect = StandardSQL;
    if (connection) {
      const dialect = connection.dialect?.toLowerCase() || '';
      if (dialect.includes('mysql') || dialect.includes('mariadb')) {
        sqlDialect = MySQL;
      } else if (dialect.includes('postgres')) {
        sqlDialect = PostgreSQL;
      }
    }

    const extensions = [
      basicSetup,
      sqlDialect,
      oneDark,
      keymap.of([
        { key: 'Tab', run: indentWithTab },
        { key: 'Mod-Enter', run: () => { execute(); return true; } },
        { key: 'Mod-s', run: () => { format(); return true; } },
        { key: 'Mod-k', run: () => { clear(); return true; } },
        { key: 'Mod-h', run: () => { showHistory = !showHistory; return true; } },
        { key: 'F1', run: () => { showSnippetDialog = true; return true; } }
      ]),
      autocompletion({ override: [createCompletionSource()] }),
      searchKeymap,
      highlightSelectionMatches(),
      EditorView.theme({
        '&': { fontSize: '14px', fontFamily: "'SF Mono', Monaco, 'Cascadia Code', monospace' },
        '.cm-scroller': { overflow: 'auto' },
        '.cm-content': { padding: '12px 0', minHeight: '100%' },
        '.cm-focused': { outline: 'none' },
        '.cm-line': { padding: '0 12px' },
        '.cm-placeholder': { color: '#666', fontStyle: 'italic' }
      })
    ];

    const state = EditorState.create({
      doc: sql,
      extensions: [
        ...extensions,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            sql = update.state.doc.toString();
          }
        }),
        EditorView.lineWrapping
      ]
    });

    editorView = new EditorView({
      state,
      parent: editorContainer
    });
  }

  onMount(() => {
    loadHistory();
    createEditor();
  });

  onDestroy(() => {
    if (editorView) {
      editorView.destroy();
    }
  });

  // 更新编辑器内容
  export async function setSql(newSql) {
    sql = newSql;
    if (editorView) {
      editorView.dispatch({
        changes: { from: 0, to: editorView.state.doc.length, insert: newSql }
      });
    }
  }

  export async function getSql() {
    return editorView ? editorView.state.doc.toString() : sql;
  }

  export async function insertText(text) {
    if (editorView) {
      const transaction = editorView.state.update({
        changes: {
          from: editorView.state.selection.main.head,
          to: editorView.state.selection.main.head,
          insert: text
        }
      });
      editorView.dispatch(transaction);
      editorView.focus();
    }
  }

  // 获取选中的 SQL 或全部
  async function getSelectedSql() {
    if (!editorView) return sql;
    const selection = editorView.state.selection.main;
    if (selection.from !== selection.to) {
      return editorView.state.sliceDoc(selection.from, selection.to);
    }
    return editorView.state.doc.toString();
  }

  // 执行 SQL
  async function execute() {
    const sqlToExecute = (await getSelectedSql()).trim() || (await getSql()).trim();
    if (!sqlToExecute) return;
    if (!connection) {
      alert('请先选择连接');
      return;
    }

    saveToHistory(sqlToExecute);

    if (batchMode && onBatchExecute) {
      // 批量执行模式
      await onBatchExecute(sqlToExecute, useTransaction);
    } else if (onExecute) {
      // 普通执行模式
      await onExecute(sqlToExecute);
    }
  }

  // 解析 SQL 语句（智能分割，处理字符串和存储过程）
  function parseStatements(sqlText) {
    const statements = [];
    let current = '';
    let inString = false;
    let stringChar = '';
    let inComment = false;
    let inLineComment = false;
    let inDelimiter = false;  // 自定义分隔符（如 DELIMITER //）

    for (let i = 0; i < sqlText.length; i++) {
      const char = sqlText[i];
      const next = sqlText[i + 1] || '';

      // 处理行注释
      if (!inString && !inComment && char === '-' && next === '-') {
        inLineComment = true;
      }

      // 处理块注释开始
      if (!inString && !inLineComment && char === '/' && next === '*') {
        inComment = true;
        current += char;
        i++;
        current += next;
        continue;
      }

      // 处理块注释结束
      if (inComment && char === '*' && next === '/') {
        inComment = false;
        current += char;
        i++;
        current += next;
        continue;
      }

      // 处理字符串
      if (!inComment && !inLineComment && (char === '"' || char === '\'' || char === '`')) {
        if (!inString) {
          inString = true;
          stringChar = char;
        } else if (char === stringChar) {
          // 检查是否转义
          const prevCount = (current.match(/\\+$/)?.[0] || '').length;
          if (prevCount % 2 === 0) {
            inString = false;
            stringChar = '';
          }
        }
      }

      // 换行结束行注释
      if (inLineComment && (char === '\n' || char === '\r')) {
        inLineComment = false;
      }

      // 处理自定义分隔符（MySQL 存储过程）
      if (!inString && !inComment && !inLineComment) {
        const remaining = sqlText.slice(i).toUpperCase();
        if (remaining.startsWith('DELIMITER')) {
          inDelimiter = true;
          const endIdx = sqlText.indexOf('\n', i);
          if (endIdx > 0) {
            const delimiterLine = sqlText.slice(i, endIdx);
            const parts = delimiterLine.split(/\s+/);
            if (parts.length >= 2) {
              // 自定义分隔符，这里简化处理
              // 实际实现需要保存新分隔符并使用它来分割语句
            }
            i = endIdx;
            continue;
          }
        }
      }

      // 处理分号（只在非字符串、非注释中）
      if (char === ';' && !inString && !inComment && !inLineComment && !inDelimiter) {
        current += char;
        const trimmed = current.trim();
        if (trimmed && !trimmed.match(/^--/)) {
          statements.push(trimmed);
        }
        current = '';
        continue;
      }

      current += char;
    }

    // 添加最后一条语句
    const trimmed = current.trim();
    if (trimmed && !trimmed.match(/^--/)) {
      statements.push(trimmed);
    }

    return statements;
  }

  // 格式化 SQL
  async function format() {
    const currentSql = await getSql();
    let formatted = currentSql.trim();

    // 首先标准化空白
    formatted = formatted.replace(/\s+/g, ' ');
    formatted = formatted.replace(/\s*,\s*/g, ', ');
    formatted = formatted.replace(/\s*\(\s*/g, '(');
    formatted = formatted.replace(/\s*\)\s*/g, ')');
    formatted = formatted.replace(/\s*=\s*/g, ' = ');
    formatted = formatted.replace(/\s*<>\s*/g, ' <> ');
    formatted = formatted.replace(/\s*!=\s*/g, ' != ');
    formatted = formatted.replace(/\s*>\s*/g, ' > ');
    formatted = formatted.replace(/\s*<\s*/g, ' < ');
    formatted = formatted.replace(/\s*>=\s*/g, ' >= ');
    formatted = formatted.replace(/\s*<=\s*/g, ' <= ');

    // 关键字大写并换行
    const keywords = [
      'SELECT', 'FROM', 'WHERE', 'INSERT INTO', 'VALUES', 'UPDATE', 'SET', 'DELETE FROM',
      'JOIN', 'LEFT JOIN', 'RIGHT JOIN', 'INNER JOIN', 'OUTER JOIN', 'CROSS JOIN', 'ON',
      'AND', 'OR', 'NOT', 'ORDER BY', 'GROUP BY', 'HAVING', 'LIMIT', 'OFFSET',
      'CREATE TABLE', 'ALTER TABLE', 'DROP TABLE', 'PRIMARY KEY', 'FOREIGN KEY',
      'UNION', 'UNION ALL', 'INTERSECT', 'EXCEPT', 'EXISTS', 'IN', 'BETWEEN',
      'LIKE', 'IS NULL', 'IS NOT NULL', 'CASE', 'WHEN', 'THEN', 'ELSE', 'END',
      'LEFT OUTER JOIN', 'RIGHT OUTER JOIN', 'FULL OUTER JOIN', 'FULL JOIN'
    ];

    keywords.sort((a, b) => b.length - a.length);

    for (const keyword of keywords) {
      const regex = new RegExp(`\\b${keyword}\\b`, 'gi');
      formatted = formatted.replace(regex, '\n' + keyword);
    }

    // 清理多余换行和添加缩进
    formatted = formatted.replace(/\n+/g, '\n');
    formatted = formatted.replace(/^\n+/, '');
    formatted = formatted.replace(/\n\s+/g, '\n  ');
    formatted = formatted.replace(/\(\s+/g, '(\n    ');
    formatted = formatted.replace(/\s+\)/g, '\n  )');

    await setSql(formatted);
  }

  // 插入代码片段
  async function insertSnippet(snippet) {
    await insertText(snippet.sql + '\n');
    showSnippetDialog = false;
  }

  // 从历史加载
  async function loadFromHistory(item) {
    await setSql(item.sql);
    showHistory = false;
  }

  // 清空
  async function clear() {
    await setSql('');
  }

  // 更新表列表（用于自动补全）
  export function updateTableNames(tables) {
    tableNames = tables;
    // 重新创建编辑器以更新自动补全
    // createEditor();  // 可选：如果需要立即更新补全
  }

  // 获取当前内容
  function getCurrentContent() {
    return editorView ? editorView.state.doc.toString() : '';
  }

  // 导出解析函数供外部使用
  export { parseStatements };
</script>

<div class="sql-editor">
  <div class="editor-toolbar">
    <button
      class="btn-run"
      class:batch-mode={batchMode}
      on:click={execute}
      title="执行 SQL (Ctrl+Enter)"
    >
      ▶ {batchMode ? '批量运行' : '运行'}
    </button>
    <button
      class="btn-batch"
      class:active={batchMode}
      on:click={() => batchMode = !batchMode}
      title="批量执行模式"
    >
      ⚡ 批量模式
    </button>
    {#if batchMode}
      <button
        class="btn-transaction"
        class:active={useTransaction}
        on:click={() => useTransaction = !useTransaction}
        title="使用事务包装"
      >
        🔒 事务
      </button>
    {/if}
    <button class="btn-format" on:click={format} title="格式化 SQL (Ctrl+S)">
      ⟡ 格式化
    </button>
    <button class="btn-snippet" on:click={() => showSnippetDialog = true} title="代码片段 (F1)">
      📋 片段
    </button>
    <button class="btn-history" on:click={() => showHistory = !showHistory} title="历史记录 (Ctrl+H)">
      🕒 历史 ({sqlHistory.length})
    </button>
    <button class="btn-clear" on:click={clear} title="清空 (Ctrl+K)">
      清空
    </button>
    <div class="spacer"></div>
    <span class="connection-info">
      {#if connection}
        {connection.name || connection.host} ({connection.dialect || 'SQL'})
      {:else}
        未连接
      {/if}
    </span>
    <span class="shortcuts-hint" title="快捷键提示">
      Ctrl+Enter:执行 | Ctrl+S:格式化 | Ctrl+H:历史 | Ctrl+K:清空 | F1:片段
    </span>
  </div>

  <div class="editor-main">
    <!-- 历史记录面板 -->
    {#if showHistory}
      <div class="history-panel">
        <div class="history-header">
          <span>历史记录</span>
          <button class="btn-close-history" on:click={() => showHistory = false}>&times;</button>
        </div>
        <div class="history-actions">
          <button class="btn-clear-history" on:click={clearHistory}>清空历史</button>
        </div>
        <div class="history-search">
          <input
            type="text"
            placeholder="搜索历史..."
            class="history-search-input"
          />
        </div>
        <div class="history-list">
          {#if sqlHistory.length === 0}
            <div class="history-empty">暂无历史记录</div>
          {:else}
            {#each sqlHistory as item}
              <div class="history-item" on:click={() => loadFromHistory(item)}>
                <div class="history-sql">{item.sql.slice(0, 150)}{item.sql.length > 150 ? '...' : ''}</div>
                <div class="history-meta">
                  <span class="history-date">{item.date}</span>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    {/if}

    <!-- 编辑器容器 -->
    <div class="editor-container" bind:this={editorContainer}></div>
  </div>
</div>

<!-- 代码片段对话框 -->
{#if showSnippetDialog}
  <div class="dialog-overlay" on:click={() => showSnippetDialog = false}>
    <div class="dialog snippet-dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h3>SQL 代码片段 ({snippets.length})</h3>
        <button class="dialog-close" on:click={() => showSnippetDialog = false}>&times;</button>
      </div>
      <div class="dialog-body">
        <div class="snippet-list">
          {#each snippets as snippet}
            <div class="snippet-item" on:click={() => insertSnippet(snippet)}>
              <span class="snippet-name">{snippet.name}</span>
              <span class="snippet-preview">{snippet.sql.slice(0, 50)}...</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(.cm-editor) {
    height: 100%;
  }

  :global(.cm-scroller) {
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace !important;
  }

  :global(.cm-line) {
    font-size: 14px;
    line-height: 1.6;
  }

  :global(.cm-gutters) {
    background: #1e1e1e !important;
    border-right: 1px solid #3e3e3e !important;
  }

  :global(.cm-lineNumbers) {
    color: #666 !important;
  }

  :global(.cm-activeLineGutter) {
    color: #d4d4d4 !important;
    background: #2a2d2e !important;
  }

  :global(.cm-activeLine) {
    background: #2a2d2e !important;
  }

  :global(.cm-selectionBackground) {
    background: #264f78 !important;
  }

  :global(.cm-tooltip) {
    background: #2d2d2d !important;
    border: 1px solid #3e3e3e !important;
    border-radius: 4px !important;
    color: #d4d4d4 !important;
  }

  :global(.cm-tooltip-autocomplete) {
    background: #252526 !important;
    border: 1px solid #3e3e3e !important;
    max-width: 300px;
  }

  :global(.cm-tooltip-autocomplete ul) {
    font-family: 'SF Mono', Monaco, monospace;
    max-height: 200px;
  }

  :global(.cm-tooltip-autocomplete ul li) {
    padding: 4px 8px;
    font-size: 13px;
  }

  :global(.cm-tooltip-autocomplete ul li[aria-selected]) {
    background: #094771 !important;
  }

  :global(.cm-completionIcon) {
    width: 16px;
    font-size: 11px;
  }

  :global(.cm-completionIcon-keyword) {
    color: #c586c0;
  }

  :global(.cm-completionIcon-type) {
    color: #4ec9b0;
  }

  :global(.cm-completionIcon-table) {
    color: #dcdcaa;
  }

  /* 搜索面板 */
  :global(.cm-search) {
    background: #252526 !important;
    border: 1px solid #3e3e3e !important;
  }

  :global(.cm-search input) {
    background: #3c3c3c !important;
    border: 1px solid #3e3e3e !important;
    color: #d4d4d4 !important;
  }

  .sql-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    padding: 6px 12px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
    gap: 4px;
    flex-wrap: wrap;
  }

  .editor-toolbar button {
    background: #3e3e3e;
    border: none;
    color: #d4d4d4;
    padding: 5px 10px;
    border-radius: 4px;
    font-size: 11px;
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

  .btn-run.batch-mode {
    background: #9b46c8 !important;
  }

  .btn-run.batch-mode:hover {
    background: #7a35a0 !important;
  }

  .btn-batch, .btn-transaction {
    background: #3e3e3e;
  }

  .btn-batch.active, .btn-transaction.active {
    background: #007acc !important;
    color: white !important;
  }

  .btn-snippet, .btn-history {
    background: #3e3e3e;
  }

  .spacer {
    flex: 1;
  }

  .connection-info {
    font-size: 10px;
    color: #888;
  }

  .shortcuts-hint {
    font-size: 9px;
    color: #666;
  }

  .editor-main {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* 历史记录面板 */
  .history-panel {
    width: 320px;
    background: #252526;
    border-right: 1px solid #3e3e3e;
    display: flex;
    flex-direction: column;
  }

  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid #3e3e3e;
    font-weight: 600;
    font-size: 12px;
  }

  .btn-close-history {
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

  .btn-close-history:hover {
    color: #d4d4d4;
  }

  .history-actions {
    padding: 6px 12px;
    border-bottom: 1px solid #3e3e3e;
  }

  .btn-clear-history {
    padding: 3px 8px;
    background: #3e3e3e;
    border: none;
    border-radius: 4px;
    color: #f48771;
    font-size: 10px;
    cursor: pointer;
  }

  .btn-clear-history:hover {
    background: #4a2a2a;
  }

  .history-search {
    padding: 8px 12px;
    border-bottom: 1px solid #3e3e3e;
  }

  .history-search-input {
    width: 100%;
    padding: 6px 8px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 11px;
  }

  .history-search-input:focus {
    outline: none;
    border-color: #007acc;
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
  }

  .history-empty {
    padding: 32px 12px;
    text-align: center;
    color: #666;
    font-size: 12px;
  }

  .history-item {
    padding: 10px 12px;
    border-bottom: 1px solid #2d2d2d;
    cursor: pointer;
  }

  .history-item:hover {
    background: #2a2d2e;
  }

  .history-sql {
    font-size: 11px;
    color: #d4d4d4;
    font-family: 'SF Mono', Monaco, monospace;
    margin-bottom: 4px;
    word-break: break-all;
    white-space: pre-wrap;
  }

  .history-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .history-date {
    font-size: 9px;
    color: #666;
  }

  /* 编辑器容器 */
  .editor-container {
    flex: 1;
    overflow: hidden;
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
    max-width: 90vw;
  }

  .snippet-dialog {
    width: 500px;
    max-height: 80vh;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #3e3e3e;
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
  }

  .dialog-close {
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

  .dialog-close:hover {
    color: #d4d4d4;
  }

  .dialog-body {
    padding: 12px 16px;
    max-height: 60vh;
    overflow-y: auto;
  }

  .snippet-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
  }

  .snippet-item {
    padding: 10px 12px;
    background: #3e3e3e;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.15s;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .snippet-item:hover {
    background: #4e4e4e;
  }

  .snippet-name {
    font-size: 12px;
    font-weight: 600;
    color: #d4d4d4;
  }

  .snippet-preview {
    font-size: 10px;
    color: #888;
    font-family: 'SF Mono', Monaco, monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* 滚动条样式 */
  .history-list::-webkit-scrollbar,
  .dialog-body::-webkit-scrollbar {
    width: 8px;
  }

  .history-list::-webkit-scrollbar-track,
  .dialog-body::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .history-list::-webkit-scrollbar-thumb,
  .dialog-body::-webkit-scrollbar-thumb {
    background: #424242;
    border-radius: 4px;
  }

  .history-list::-webkit-scrollbar-thumb:hover,
  .dialog-body::-webkit-scrollbar-thumb:hover {
    background: #4e4e4e;
  }
</style>
