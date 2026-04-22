<script>
  import { invoke } from '@tauri-apps/api/core';
  import { notifySuccess } from '../lib/notifications';

  export let connection = null;
  export let tableName = '';
  export let targetDatabase = '';
  export let isCreatingNewTable = false;
  export let onClose = () => {};
  export let onRefresh = () => {};

  // 新建表的表名
  let newTableName = '';

  // 表结构数据
  const DEFAULT_TABLE_INFO = {
    engine: 'InnoDB',
    charset: 'utf8mb4',
    collation: 'utf8mb4_unicode_ci',
    comment: ''
  };

  let columns = [];
  let originalColumns = [];
  let indexes = [];
  let tableInfo = { ...DEFAULT_TABLE_INFO };
  let originalTableInfo = { ...DEFAULT_TABLE_INFO };

  // 状态
  let loading = false;
  let saving = false;
  let error = null;
  let hasChanges = false;

  // 重置为新建表模式
  function resetToNewTableMode() {
    columns = []; // 暂时为空，让用户手动添加列
    originalColumns = [];
    indexes = [];
    newTableName = '';
    tableInfo = { ...DEFAULT_TABLE_INFO };
    originalTableInfo = { ...DEFAULT_TABLE_INFO };
    hasChanges = false;
    error = null;
  }

  // 重置为编辑模式
  function resetToEditMode() {
    columns = [];
    originalColumns = [];
    indexes = [];
    newTableName = '';
    tableInfo = { ...DEFAULT_TABLE_INFO };
    originalTableInfo = { ...DEFAULT_TABLE_INFO };
    hasChanges = false;
    error = null;
  }

  // 跟踪上一次的模式，避免重复触发
  let lastMode = 'none'; // 'none', 'new', 'edit'

  // 监听模式变化
  $: {
    const currentMode = isCreatingNewTable ? 'new' : (tableName ? 'edit' : 'none');

    if (currentMode !== lastMode) {
      lastMode = currentMode;
      if (currentMode === 'new') {
        resetToNewTableMode();
      } else if (currentMode === 'edit') {
        resetToEditMode();
      }
    }
  }

  // 可用的数据类型
  const dataTypes = [
    { group: '整数', types: ['TINYINT', 'SMALLINT', 'MEDIUMINT', 'INT', 'BIGINT'] },
    { group: '浮点数', types: ['FLOAT', 'DOUBLE', 'DECIMAL', 'NUMERIC'] },
    { group: '字符串', types: ['CHAR', 'VARCHAR', 'TEXT', 'TINYTEXT', 'MEDIUMTEXT', 'LONGTEXT'] },
    { group: '二进制', types: ['BINARY', 'VARBINARY', 'BLOB', 'TINYBLOB', 'MEDIUMBLOB', 'LONGBLOB'] },
    { group: '日期时间', types: ['DATE', 'TIME', 'DATETIME', 'TIMESTAMP', 'YEAR'] },
    { group: '其他', types: ['BOOLEAN', 'JSON', 'ENUM', 'SET'] }
  ];

  const storageEngines = ['InnoDB', 'MyISAM', 'MEMORY', 'ARCHIVE', 'CSV'];
  const charsets = ['utf8mb4', 'utf8', 'latin1', 'gbk', 'big5'];
  const collations = [
    'utf8mb4_unicode_ci', 'utf8mb4_general_ci',
    'utf8_unicode_ci', 'utf8_general_ci',
    'latin1_swedish_ci', 'gbk_chinese_ci', 'big5_chinese_ci'
  ];

  function cloneColumn(column) {
    return { ...column };
  }

  // 加载表结构
  async function loadTableSchema() {
    if (!connection || !tableName) return;
    // 新建表模式不需要加载现有表结构
    if (isCreatingNewTable) return;

    loading = true;
    error = null;

    try {
      const [database, table] = tableName.split('.');

      // 获取列信息
      const schema = await invoke('meta_get_table_schema', {
        connection,
        database,
        table
      });

      // 转换列信息为编辑格式
      columns = schema.columns.map((col, idx) => {
        // Rust serde rename: column_type -> type
        const colType = col.type || col.column_type || 'VARCHAR';
        return {
          id: idx,
          originalName: col.name,
          name: col.name,
          type: colType,
          length: extractLength(colType),
          nullable: col.nullable !== false,
          primaryKey: false, // 稍后从索引中设置
          autoIncrement: col.extra?.includes('auto_increment') || false,
          defaultValue: col.default || '',
          comment: col.comment || '',
          isOriginal: true // 标记为原始列
        };
      });

      // 从索引中获取主键信息
      if (schema.indexes) {
        const pkIndex = schema.indexes.find(idx => idx.name === 'PRIMARY');
        if (pkIndex) {
          pkIndex.columns.forEach(pkCol => {
            const col = columns.find(c => c.name === pkCol);
            if (col) col.primaryKey = true;
          });
        }
        indexes = schema.indexes;
      }

      // 获取表信息
      const nextTableInfo = schema.create_sql
        ? (extractTableInfo(schema.create_sql) || { ...DEFAULT_TABLE_INFO })
        : { ...DEFAULT_TABLE_INFO };
      tableInfo = nextTableInfo;
      originalColumns = columns.map(cloneColumn);
      originalTableInfo = { ...nextTableInfo };

      hasChanges = false;
    } catch (err) {
      error = String(err);
      console.error('Load schema error:', err);
    } finally {
      loading = false;
    }
  }

  // 从类型字符串中提取长度
  function extractLength(typeStr) {
    if (!typeStr) return '';
    const match = typeStr.match(/\(([^)]+)\)/);
    return match ? match[1] : '';
  }

  // 从建表 SQL 中提取表信息
  function extractTableInfo(sql) {
    const engineMatch = sql.match(/ENGINE\s*=\s*(\w+)/i);
    const charsetMatch = sql.match(/CHARSET\s*=\s*(\w+)/i);
    const collationMatch = sql.match(/COLLATE\s*=\s*([\w_]+)/i);
    const commentMatch = sql.match(/COMMENT\s*=\s*'([^']*)'/i);

    return {
      engine: engineMatch ? engineMatch[1] : 'InnoDB',
      charset: charsetMatch ? charsetMatch[1] : 'utf8mb4',
      collation: collationMatch ? collationMatch[1] : 'utf8mb4_unicode_ci',
      comment: commentMatch ? commentMatch[1] : ''
    };
  }

  // 添加新列
  function addColumn() {
    columns = [...columns, {
      id: Date.now(),
      name: '',
      type: 'VARCHAR',
      length: '255',
      nullable: true,
      primaryKey: false,
      autoIncrement: false,
      defaultValue: '',
      comment: '',
      isOriginal: false,
      originalName: null
    }];
    hasChanges = true;
  }

  // 删除列
  function deleteColumn(colId) {
    columns = columns.filter(c => c.id !== colId);
    hasChanges = true;
  }

  // 更新列属性
  function updateColumn(colId, field, value) {
    columns = columns.map(c => {
      if (c.id === colId) {
        const updated = { ...c, [field]: value };
        // 设置主键时自动设置 NOT NULL
        if (field === 'primaryKey' && value === true) {
          updated.nullable = false;
        }
        // 设置自增时自动设置主键（MySQL 要求自增列必须是键）
        if (field === 'autoIncrement' && value === true) {
          updated.primaryKey = true;
          updated.nullable = false;
        }
        return updated;
      }
      return c;
    });
    hasChanges = true;
  }

  // 生成完整的类型字符串
  function getTypeString(col) {
    let type = col.type;
    if (col.length) {
      type += `(${col.length})`;
    }
    return type;
  }

  // 保存表结构
  async function saveSchema() {
    saving = true;
    error = null;

    try {
      // 验证
      const validationError = validateSchema();
      if (validationError) {
        error = validationError;
        return;
      }

      if (isCreatingNewTable) {
        // 新建表模式
        if (!newTableName.trim()) {
          error = '请输入表名';
          return;
        }

        // 准备列定义
        const columnDefs = columns.map(col => ({
          name: col.name,
          type: col.type,
          length: col.length || undefined,
          nullable: col.nullable,
          primary_key: col.primaryKey,
          auto_increment: col.autoIncrement,
          default_value: col.defaultValue || undefined,
          comment: col.comment || ''
        }));

        const result = await invoke('meta_create_table', {
          params: {
            connection,
            database: targetDatabase,
            table: newTableName.trim(),
            columns: columnDefs,
            engine: tableInfo.engine,
            charset: tableInfo.charset,
            collation: tableInfo.collation,
            comment: tableInfo.comment
          }
        });

        notifySuccess(result);
        onRefresh();
      } else {
        // 编辑现有表模式
        // 生成 ALTER TABLE SQL
        const sqlStatements = generateAlterSQL();

        if (sqlStatements.length === 0) {
          hasChanges = false;
          notifySuccess('没有需要保存的结构变更');
          return;
        }

        for (const sql of sqlStatements) {
          await invoke('query_execute', {
            connection,
            sql,
            maxRows: 0
          });
        }

        columns = columns.map((col) => ({
          ...col,
          isOriginal: true,
          originalName: col.name
        }));
        originalColumns = columns.map(cloneColumn);
        originalTableInfo = { ...tableInfo };
        hasChanges = false;
        onRefresh();
        notifySuccess('表结构保存成功');
      }
    } catch (err) {
      error = '保存失败: ' + err;
    } finally {
      saving = false;
    }
  }

  // 验证表结构
  function validateSchema() {
    if (!columns.length) return '至少需要一列';

    for (const col of columns) {
      if (!col.name.trim()) return '列名不能为空';
      if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(col.name)) {
        return `列名 "${col.name}" 不符合命名规则`;
      }
      if (col.autoIncrement && !col.primaryKey) {
        return `列 "${col.name}" 设置了自增必须是主键`;
      }
    }

    const primaryKeys = columns.filter(c => c.primaryKey);
    if (primaryKeys.length > 1) {
      return '目前只支持单列主键';
    }

    return null;
  }

  // 生成 ALTER TABLE SQL
  function generateAlterSQL() {
    const [database, table] = tableName.split('.');
    const statements = [];

    const currentOriginalNames = new Set(
      columns
        .filter((col) => col.isOriginal)
        .map((col) => col.originalName || col.name)
    );
    const originalByName = new Map(
      originalColumns.map((col) => [col.originalName || col.name, col])
    );
    const deletedCols = originalColumns.filter(
      (col) => !currentOriginalNames.has(col.originalName || col.name)
    );
    const newCols = columns.filter((col) => !col.isOriginal);
    const modifiedCols = columns.filter((col) => {
      if (!col.isOriginal) return false;
      const originalCol = originalByName.get(col.originalName || col.name);
      return !columnsEqual(col, originalCol);
    });

    const originalPrimaryKey = originalColumns.find((col) => col.primaryKey)?.originalName
      || originalColumns.find((col) => col.primaryKey)?.name
      || '';
    const currentPrimaryKey = columns.find((col) => col.primaryKey)?.name || '';
    const primaryKeyChanged = originalPrimaryKey !== currentPrimaryKey;
    let primaryKeyHandled = false;

    if (primaryKeyChanged && originalPrimaryKey) {
      statements.push(`ALTER TABLE \`${database}\`.\`${table}\` DROP PRIMARY KEY;`);
    }

    for (const col of deletedCols) {
      statements.push(`ALTER TABLE \`${database}\`.\`${table}\` DROP COLUMN \`${col.name}\`;`);
    }

    for (const col of newCols) {
      const clauses = [`ADD COLUMN ${buildColumnDefinition(col)}`];
      if (col.primaryKey && primaryKeyChanged) {
        clauses.push(`ADD PRIMARY KEY (\`${col.name}\`)`);
        primaryKeyHandled = true;
      }
      statements.push(`ALTER TABLE \`${database}\`.\`${table}\` ${clauses.join(', ')};`);
    }

    for (const col of modifiedCols) {
      const originalCol = originalByName.get(col.originalName || col.name);
      const columnClause = originalCol && originalCol.name !== col.name
        ? `CHANGE COLUMN \`${originalCol.name}\` ${buildColumnDefinition(col)}`
        : `MODIFY COLUMN ${buildColumnDefinition(col)}`;

      const clauses = [columnClause];
      if (col.primaryKey && primaryKeyChanged && !primaryKeyHandled) {
        clauses.push(`ADD PRIMARY KEY (\`${col.name}\`)`);
        primaryKeyHandled = true;
      }

      statements.push(`ALTER TABLE \`${database}\`.\`${table}\` ${clauses.join(', ')};`);
    }

    if (currentPrimaryKey && primaryKeyChanged && !primaryKeyHandled) {
      statements.push(`ALTER TABLE \`${database}\`.\`${table}\` ADD PRIMARY KEY (\`${currentPrimaryKey}\`);`);
    }

    if (!tableInfoEquals(tableInfo, originalTableInfo)) {
      statements.push(generateTableOptionsSQL(database, table));
    }

    return statements;
  }

  function columnsEqual(current, original) {
    if (!original) return false;
    return (
      current.name === original.name &&
      current.type === original.type &&
      (current.length || '') === (original.length || '') &&
      current.nullable === original.nullable &&
      current.primaryKey === original.primaryKey &&
      current.autoIncrement === original.autoIncrement &&
      (current.defaultValue || '') === (original.defaultValue || '') &&
      (current.comment || '') === (original.comment || '')
    );
  }

  function tableInfoEquals(current, original) {
    return (
      (current.engine || '') === (original?.engine || '') &&
      (current.charset || '') === (original?.charset || '') &&
      (current.collation || '') === (original?.collation || '') &&
      (current.comment || '') === (original?.comment || '')
    );
  }

  function buildColumnDefinition(col) {
    let sql = `\`${col.name}\` ${getTypeString(col)}`;
    if (!col.nullable || col.primaryKey || col.autoIncrement) sql += ' NOT NULL';
    if (col.defaultValue) sql += ` DEFAULT ${quoteValue(col.defaultValue)}`;
    if (col.autoIncrement) sql += ' AUTO_INCREMENT';
    if (col.comment) sql += ` COMMENT '${col.comment.replace(/'/g, "''")}'`;
    return sql;
  }

  function generateTableOptionsSQL(database, table) {
    const comment = tableInfo.comment ? tableInfo.comment.replace(/'/g, "''") : '';
    return `ALTER TABLE \`${database}\`.\`${table}\` ENGINE=${tableInfo.engine}, DEFAULT CHARACTER SET=${tableInfo.charset}, COLLATE=${tableInfo.collation}, COMMENT='${comment}';`;
  }

  // 引用值
  function quoteValue(value) {
    if (!value) return "''";
    if (value === 'NULL') return 'NULL';
    if (/^CURRENT_TIMESTAMP(?:\(\))?$/i.test(value)) return value.toUpperCase();
    if (!isNaN(value)) return value;
    return `'${value.replace(/'/g, "''")}'`;
  }

  // 获取类型的长度输入框
  function needsLength(type) {
    const needsLen = ['CHAR', 'VARCHAR', 'BINARY', 'VARBINARY', 'DECIMAL', 'NUMERIC'];
    return needsLen.some(t => type?.includes(t));
  }

  // 获取类型的枚举值输入
  function needsEnumValues(type) {
    return type === 'ENUM' || type === 'SET';
  }

  // 监听 tableName 变化
  $: if (tableName && !isCreatingNewTable) {
    loadTableSchema();
  }
</script>

<div class="table-designer">
  <!-- 头部工具栏 -->
  <div class="designer-header">
    <div class="designer-title">
      <span class="icon">{isCreatingNewTable ? '📝' : '📋'}</span>
      {#if isCreatingNewTable}
        <span>新建表 - {targetDatabase}.</span>
        <input
          type="text"
          class="table-name-input"
          bind:value={newTableName}
          placeholder="表名"
          on:input={() => hasChanges = true}
        />
      {:else}
        <span>表设计 - {tableName}</span>
      {/if}
    </div>
    <div class="designer-actions">
      {#if hasChanges}
        <span class="unsaved-indicator">有未保存的更改</span>
      {/if}
      <button class="btn btn-secondary" on:click={onClose}>关闭</button>
      <button class="btn btn-primary" on:click={saveSchema} disabled={saving}>
        {saving ? '保存中...' : '保存'}
      </button>
    </div>
  </div>

  {#if error}
    <div class="error-banner">{error}</div>
  {/if}

  {#if loading && !isCreatingNewTable}
    <div class="loading-state">加载中...</div>
  {:else}
    <div class="designer-content">
      <!-- 列列表 -->
      <div class="columns-section">
        <div class="section-header">
          <h3>列</h3>
          <button class="btn-add" on:click={addColumn}>+ 添加列</button>
        </div>

        <div class="columns-table-wrapper">
          <table class="columns-table">
            <thead>
              <tr>
                <th class="col-pk">主键</th>
                <th class="col-auto">自增</th>
                <th class="col-name">列名</th>
                <th class="col-type">类型</th>
                <th class="col-length">长度</th>
                <th class="col-nullable">NULL</th>
                <th class="col-default">默认值</th>
                <th class="col-comment">注释</th>
                <th class="col-actions">操作</th>
              </tr>
            </thead>
            <tbody>
              {#each columns as col, idx}
                <!-- {@debug col} -->
                <tr class:original-col={col.isOriginal}>
                  <!-- 主键 -->
                  <td class="col-pk">
                    <input
                      type="checkbox"
                      checked={col.primaryKey}
                      on:change={(e) => updateColumn(col.id, 'primaryKey', e.target.checked)}
                    />
                  </td>

                  <!-- 自增 -->
                  <td class="col-auto">
                    <input
                      type="checkbox"
                      checked={col.autoIncrement}
                      on:change={(e) => updateColumn(col.id, 'autoIncrement', e.target.checked)}
                      disabled={!col.primaryKey}
                    />
                  </td>

                  <!-- 列名 -->
                  <td class="col-name">
                    <input
                      type="text"
                      class="input-name"
                      bind:value={col.name}
                      on:change={(e) => updateColumn(col.id, 'name', e.target.value)}
                      placeholder="列名"
                    />
                  </td>

                  <!-- 类型 -->
                  <td class="col-type">
                    <select
                      class="select-type"
                      on:change={(e) => updateColumn(col.id, 'type', e.target.value)}
                    >
                      {#each dataTypes as group}
                        <optgroup label={group.group}>
                          {#each group.types as type}
                            <option value={type} selected={col.type === type}>{type}</option>
                          {/each}
                        </optgroup>
                      {/each}
                    </select>
                  </td>

                  <!-- 长度 -->
                  <td class="col-length">
                    {#if needsLength(col.type)}
                      <input
                        type="text"
                        class="input-length"
                        bind:value={col.length}
                        on:change={(e) => updateColumn(col.id, 'length', e.target.value)}
                        placeholder="长度"
                      />
                    {:else}
                      <span class="text-muted">-</span>
                    {/if}
                  </td>

                  <!-- NULL -->
                  <td class="col-nullable">
                    <input
                      type="checkbox"
                      checked={col.nullable}
                      disabled={col.primaryKey}
                      on:change={(e) => updateColumn(col.id, 'nullable', e.target.checked)}
                    />
                  </td>

                  <!-- 默认值 -->
                  <td class="col-default">
                    <input
                      type="text"
                      class="input-default"
                      bind:value={col.defaultValue}
                      on:change={(e) => updateColumn(col.id, 'defaultValue', e.target.value)}
                      placeholder="默认值"
                    />
                  </td>

                  <!-- 注释 -->
                  <td class="col-comment">
                    <input
                      type="text"
                      class="input-comment"
                      bind:value={col.comment}
                      on:change={(e) => updateColumn(col.id, 'comment', e.target.value)}
                      placeholder="注释"
                    />
                  </td>

                  <!-- 操作 -->
                  <td class="col-actions">
                    <button
                      class="btn-delete"
                      on:click={() => deleteColumn(col.id)}
                      title="删除列"
                    >
                      🗑️
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>

      <!-- 表选项 -->
      <div class="options-section">
        <div class="section-header">
          <h3>表选项</h3>
        </div>

        <div class="options-grid">
          <div class="option-item">
            <label for="table-engine">存储引擎</label>
            <select id="table-engine" bind:value={tableInfo.engine} on:change={() => hasChanges = true}>
              {#each storageEngines as engine}
                <option value={engine}>{engine}</option>
              {/each}
            </select>
          </div>

          <div class="option-item">
            <label for="table-charset">字符集</label>
            <select id="table-charset" bind:value={tableInfo.charset} on:change={() => hasChanges = true}>
              {#each charsets as charset}
                <option value={charset}>{charset}</option>
              {/each}
            </select>
          </div>

          <div class="option-item">
            <label for="table-collation">排序规则</label>
            <select id="table-collation" bind:value={tableInfo.collation} on:change={() => hasChanges = true}>
              {#each collations as collation}
                <option value={collation}>{collation}</option>
              {/each}
            </select>
          </div>

          <div class="option-item">
            <label for="table-comment">表注释</label>
            <input
              id="table-comment"
              type="text"
              class="input-table-comment"
              bind:value={tableInfo.comment}
              placeholder="表注释"
              on:input={() => hasChanges = true}
            />
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .table-designer {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    color: #d4d4d4;
  }

  .designer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
  }

  .designer-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
  }

  .designer-title .icon {
    font-size: 16px;
  }

  .table-name-input {
    padding: 4px 10px;
    background: #3c3c3c;
    border: 1px solid #007acc;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 13px;
    font-family: inherit;
    font-weight: 600;
    min-width: 150px;
  }

  .table-name-input:focus {
    outline: none;
    border-color: #0098ff;
  }

  .designer-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .unsaved-indicator {
    font-size: 12px;
    color: #e8c16a;
    padding: 4px 8px;
    background: #3a3a1e;
    border-radius: 4px;
  }

  .btn {
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #007acc;
    color: white;
    border-color: #005a9e;
  }

  .btn-primary:hover:not(:disabled) {
    background: #006cbd;
  }

  .btn-secondary {
    background: #3e3e3e;
    color: #d4d4d4;
    border-color: #4e4e4e;
  }

  .btn-secondary:hover {
    background: #4e4e4e;
  }

  .error-banner {
    padding: 10px 16px;
    background: #3c1f1e;
    color: #f48771;
    font-size: 12px;
    border-bottom: 1px solid #3e3e3e;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: #888;
  }

  .designer-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .columns-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #252526;
    border-bottom: 1px solid #3e3e3e;
  }

  .section-header h3 {
    margin: 0;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: #888;
  }

  .btn-add {
    padding: 4px 10px;
    background: #2da042;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
  }

  .btn-add:hover {
    background: #238736;
  }

  .columns-table-wrapper {
    flex: 1;
    overflow: auto;
  }

  .columns-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .columns-table thead {
    position: sticky;
    top: 0;
    background: #2d2d2d;
    z-index: 1;
  }

  .columns-table th {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 1px solid #3e3e3e;
    border-right: 1px solid #2d2d2d;
    font-weight: 500;
    color: #888;
    font-size: 11px;
    text-transform: uppercase;
  }

  .columns-table th:last-child {
    border-right: none;
  }

  .columns-table td {
    padding: 6px 8px;
    border-bottom: 1px solid #2d2d2d;
    border-right: 1px solid #2d2d2d;
    background: #1e1e1e;
  }

  .columns-table td:last-child {
    border-right: none;
  }

  .columns-table tbody tr:hover td {
    background: #252526;
  }

  .columns-table tbody tr.original-col td {
    background: #1e1e1e;
  }

  .columns-table tbody tr:not(.original-col) td {
    background: #1a2e1e;
  }

  .col-pk {
    width: 50px;
    text-align: center;
  }

  .col-auto {
    width: 55px;
    text-align: center;
  }

  .col-name {
    width: 150px;
  }

  .col-type {
    width: 140px;
  }

  .col-length {
    width: 80px;
  }

  .col-nullable {
    width: 50px;
    text-align: center;
  }

  .col-default {
    width: 120px;
  }

  .col-comment {
    min-width: 150px;
  }

  .col-actions {
    width: 50px;
    text-align: center;
  }

  .input-name,
  .input-length,
  .input-default,
  .input-comment,
  .input-table-comment {
    width: 100%;
    padding: 4px 8px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 3px;
    color: #d4d4d4;
    font-size: 12px;
    font-family: inherit;
  }

  .input-name:focus,
  .input-length:focus,
  .input-default:focus,
  .input-comment:focus,
  .select-type:focus,
  .input-table-comment:focus {
    outline: none;
    border-color: #007acc;
  }

  .select-type {
    width: 100%;
    padding: 4px 6px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 3px;
    color: #d4d4d4;
    font-size: 12px;
    font-family: inherit;
  }

  .select-type optgroup {
    color: #888;
    font-style: italic;
  }

  .text-muted {
    color: #666;
  }

  .btn-delete {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 14px;
    padding: 4px;
    opacity: 0.7;
  }

  .btn-delete:hover {
    opacity: 1;
  }

  .options-section {
    border-top: 1px solid #3e3e3e;
    background: #252526;
  }

  .options-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 16px;
    padding: 16px;
  }

  .option-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .option-item label {
    font-size: 11px;
    color: #888;
    text-transform: uppercase;
  }

  .option-item select,
  .option-item input {
    padding: 6px 10px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
  }

  .option-item select:focus,
  .option-item input:focus {
    outline: none;
    border-color: #007acc;
  }

  /* 滚动条 */
  .columns-table-wrapper::-webkit-scrollbar {
    width: 14px;
    height: 14px;
  }

  .columns-table-wrapper::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .columns-table-wrapper::-webkit-scrollbar-thumb {
    background: #424242;
    border-radius: 7px;
    border: 3px solid #1e1e1e;
  }

  .columns-table-wrapper::-webkit-scrollbar-thumb:hover {
    background: #4e4e4e;
  }
</style>
