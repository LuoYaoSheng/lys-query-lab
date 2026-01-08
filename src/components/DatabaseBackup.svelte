<script>
  import { invoke } from '@tauri-apps/api/core';
  import { save, open } from '@tauri-apps/plugin-dialog';

  export let connection = null;
  export let databases = [];
  export let onClose = () => {};

  let selectedDatabase = '';
  let selectedTables = [];
  let exportType = 'structure'; // 'structure', 'data', 'both'
  let exportFormat = 'sql'; // 'sql', 'json', 'csv'
  let isExporting = false;
  let exportProgress = 0;
  let exportStatus = '';
  let showResult = false;
  let exportResult = null;

  let importMode = false;
  let importFile = '';
  let importDropExisting = false;
  let isImporting = false;
  let importProgress = 0;
  let importStatus = '';
  let importResult = null;

  $: availableTables = [];

  // 获取数据库表列表
  async function loadTables() {
    if (!selectedDatabase || !connection) return;

    try {
      const tables = await invoke('meta_list_tables', {
        connection,
        database: selectedDatabase
      });
      availableTables = tables || [];
      // 默认全选
      selectedTables = [...availableTables];
    } catch (err) {
      console.error('Failed to load tables:', err);
      availableTables = [];
      selectedTables = [];
    }
  }

  // 切换表选择
  function toggleTable(table) {
    if (selectedTables.includes(table)) {
      selectedTables = selectedTables.filter(t => t !== table);
    } else {
      selectedTables = [...selectedTables, table];
    }
  }

  // 全选/取消全选
  function toggleSelectAll() {
    if (selectedTables.length === availableTables.length) {
      selectedTables = [];
    } else {
      selectedTables = [...availableTables];
    }
  }

  // 导出数据库
  async function exportDatabase() {
    if (!connection || !selectedDatabase) {
      exportStatus = '请选择数据库';
      return;
    }

    if (selectedTables.length === 0) {
      exportStatus = '请选择至少一个表';
      return;
    }

    isExporting = true;
    exportProgress = 0;
    exportStatus = '准备导出...';
    showResult = false;
    exportResult = null;

    try {
      // 选择保存路径
      const ext = exportFormat === 'json' ? 'json' : exportFormat === 'csv' ? 'csv' : 'sql';
      const fileName = `${selectedDatabase}_backup_${Date.now()}.${ext}`;

      const filePath = await save({
        title: '保存数据库备份',
        defaultPath: fileName,
        filters: [
          {
            name: ext.toUpperCase(),
            extensions: [ext]
          },
          {
            name: 'All Files',
            extensions: ['*']
          }
        ]
      });

      if (!filePath) {
        exportStatus = '已取消';
        isExporting = false;
        return;
      }

      exportStatus = '正在导出...';

      // 调用后端导出
      const result = await invoke('db_export', {
        connection,
        database: selectedDatabase,
        tables: selectedTables,
        exportType,
        format: exportFormat,
        filePath
      });

      exportProgress = 100;
      exportStatus = '导出完成！';
      exportResult = {
        success: true,
        file: filePath,
        tables: selectedTables.length,
        size: result.size || 0
      };
      showResult = true;
    } catch (err) {
      exportStatus = '导出失败: ' + err;
      exportResult = {
        success: false,
        error: String(err)
      };
      showResult = true;
    } finally {
      isExporting = false;
    }
  }

  // 导入数据库
  async function importDatabase() {
    if (!connection || !selectedDatabase) {
      importStatus = '请选择数据库';
      return;
    }

    if (!importFile) {
      importStatus = '请选择导入文件';
      return;
    }

    isImporting = true;
    importProgress = 0;
    importStatus = '准备导入...';
    importResult = null;

    try {
      importStatus = '正在导入...';

      // 调用后端导入
      const result = await invoke('db_import', {
        connection,
        database: selectedDatabase,
        filePath: importFile,
        dropExisting: importDropExisting
      });

      importProgress = 100;
      importStatus = '导入完成！';
      importResult = {
        success: true,
        tables: result.tables || 0,
        rows: result.rows || 0
      };
    } catch (err) {
      importStatus = '导入失败: ' + err;
      importResult = {
        success: false,
        error: String(err)
      };
    } finally {
      isImporting = false;
    }
  }

  // 选择导入文件
  async function selectImportFile() {
    try {
      const filePath = await open({
        title: '选择备份文件',
        multiple: false,
        filters: [
          {
            name: 'SQL Files',
            extensions: ['sql']
          },
          {
            name: 'JSON Files',
            extensions: ['json']
          },
          {
            name: 'All Files',
            extensions: ['*']
          }
        ]
      });

      if (filePath) {
        importFile = filePath;
      }
    } catch (err) {
      console.error('File select error:', err);
    }
  }

  // 切换导出/导入模式
  function switchToExport() {
    importMode = false;
  }

  function switchToImport() {
    importMode = true;
  }

  // 关闭
  function close() {
    onClose();
  }
</script>

<div class="backup-container">
  <div class="backup-header">
    <div class="backup-title">
      <button
        class="mode-tab"
        class:active={!importMode}
        on:click={switchToExport}
      >
        📤 导出备份
      </button>
      <button
        class="mode-tab"
        class:active={importMode}
        on:click={switchToImport}
      >
        📥 导入还原
      </button>
    </div>
    <button class="btn-close" on:click={close}>&times;</button>
  </div>

  <div class="backup-content">
    {#if !importMode}
      <!-- 导出模式 -->
      <div class="export-panel">
        <div class="form-section">
          <div class="form-group">
            <label>选择数据库</label>
            <select bind:value={selectedDatabase} on:change={loadTables}>
              <option value="">-- 请选择 --</option>
              {#each databases as db}
                <option value={db}>{db}</option>
              {/each}
            </select>
          </div>
        </div>

        {#if selectedDatabase}
          <div class="form-section">
            <div class="section-header">
              <span>选择表 ({selectedTables.length}/{availableTables.length})</span>
              <button class="btn-link" on:click={toggleSelectAll}>
                {selectedTables.length === availableTables.length ? '取消全选' : '全选'}
              </button>
            </div>
            <div class="tables-grid">
              {#each availableTables as table}
                <label class="table-checkbox">
                  <input
                    type="checkbox"
                    checked={selectedTables.includes(table)}
                    on:change={() => toggleTable(table)}
                  />
                  <span class="table-name">{table}</span>
                </label>
              {/each}
            </div>
          </div>

          <div class="form-section">
            <div class="form-group">
              <label>导出类型</label>
              <div class="radio-group">
                <label class="radio-item">
                  <input type="radio" bind:group={exportType} value="structure" />
                  <span>仅结构</span>
                </label>
                <label class="radio-item">
                  <input type="radio" bind:group={exportType} value="data" />
                  <span>仅数据</span>
                </label>
                <label class="radio-item">
                  <input type="radio" bind:group={exportType} value="both" />
                  <span>结构和数据</span>
                </label>
              </div>
            </div>

            <div class="form-group">
              <label>导出格式</label>
              <div class="radio-group">
                <label class="radio-item">
                  <input type="radio" bind:group={exportFormat} value="sql" />
                  <span>SQL (.sql)</span>
                </label>
                <label class="radio-item">
                  <input type="radio" bind:group={exportFormat} value="json" />
                  <span>JSON (.json)</span>
                </label>
                <label class="radio-item">
                  <input type="radio" bind:group={exportFormat} value="csv" />
                  <span>CSV (.csv)</span>
                </label>
              </div>
            </div>
          </div>

          <div class="action-section">
            <button
              class="btn-export"
              on:click={exportDatabase}
              disabled={isExporting || selectedTables.length === 0}
            >
              {isExporting ? '导出中...' : '📤 开始导出'}
            </button>

            {#if isExporting}
              <div class="progress-bar">
                <div class="progress-fill" style="width: {exportProgress}%"></div>
              </div>
              <div class="progress-status">{exportStatus}</div>
            {/if}

            {#if showResult && exportResult}
              <div class="result-message" class:success={exportResult.success} class:error={!exportResult.success}>
                {#if exportResult.success}
                  <div>✓ 导出成功</div>
                  <div class="result-details">
                    保存位置: {exportResult.file}<br>
                    表数量: {exportResult.tables}
                  </div>
                {:else}
                  <div>✗ 导出失败</div>
                  <div class="result-details">{exportResult.error}</div>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      </div>

    {:else}
      <!-- 导入模式 -->
      <div class="import-panel">
        <div class="form-section">
          <div class="form-group">
            <label>目标数据库</label>
            <select bind:value={selectedDatabase}>
              <option value="">-- 请选择 --</option>
              {#each databases as db}
                <option value={db}>{db}</option>
              {/each}
            </select>
          </div>
        </div>

        <div class="form-section">
          <div class="form-group">
            <label>备份文件</label>
            <div class="file-selector">
              <input type="text" bind:value={importFile} placeholder="选择备份文件..." readonly />
              <button class="btn-browse" on:click={selectImportFile}>浏览</button>
            </div>
          </div>

          <div class="form-group">
            <label class="checkbox-item">
              <input type="checkbox" bind:checked={importDropExisting} />
              <span>导入前删除现有表（谨慎使用）</span>
            </label>
          </div>
        </div>

        <div class="action-section">
          <button
            class="btn-import"
            on:click={importDatabase}
            disabled={isImporting || !importFile || !selectedDatabase}
          >
            {isImporting ? '导入中...' : '📥 开始导入'}
          </button>

          {#if isImporting}
            <div class="progress-bar">
              <div class="progress-fill" style="width: {importProgress}%"></div>
            </div>
            <div class="progress-status">{importStatus}</div>
          {/if}

          {#if importResult}
            <div class="result-message" class:success={importResult.success} class:error={!importResult.success}>
              {#if importResult.success}
                <div>✓ 导入成功</div>
                <div class="result-details">
                  导入表数: {importResult.tables}<br>
                  导入行数: {importResult.rows}
                </div>
              {:else}
                <div>✗ 导入失败</div>
                <div class="result-details">{importResult.error}</div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .backup-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
  }

  .backup-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #2d2d2d;
    border-bottom: 1px solid #3e3e3e;
  }

  .backup-title {
    display: flex;
    gap: 4px;
  }

  .mode-tab {
    padding: 6px 16px;
    background: transparent;
    border: none;
    color: #888;
    font-size: 12px;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.2s;
  }

  .mode-tab:hover {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .mode-tab.active {
    background: #007acc;
    color: white;
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

  .backup-content {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }

  .form-section {
    margin-bottom: 20px;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 600;
    color: #d4d4d4;
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

  .form-group select,
  .form-group input[type="text"] {
    width: 100%;
    padding: 8px 12px;
    background: #3e3e3e;
    border: 1px solid #4e4e4e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 13px;
  }

  .form-group select:focus,
  .form-group input[type="text"]:focus {
    outline: none;
    border-color: #007acc;
  }

  .tables-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 6px;
    max-height: 200px;
    overflow-y: auto;
    padding: 8px;
    background: #252526;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
  }

  .table-checkbox {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: #3e3e3e;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .table-checkbox:hover {
    background: #4e4e4e;
  }

  .table-checkbox input {
    cursor: pointer;
  }

  .table-name {
    color: #d4d4d4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .radio-group {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
  }

  .radio-item {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-size: 12px;
    color: #d4d4d4;
  }

  .radio-item input {
    cursor: pointer;
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .checkbox-item input {
    cursor: pointer;
  }

  .checkbox-item span {
    font-size: 12px;
    color: #d4d4d4;
  }

  .file-selector {
    display: flex;
    gap: 8px;
  }

  .file-selector input {
    flex: 1;
  }

  .btn-browse {
    padding: 8px 16px;
    background: #3e3e3e;
    border: 1px solid #4e4e4e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn-browse:hover {
    background: #4e4e4e;
    border-color: #007acc;
  }

  .action-section {
    padding: 16px;
    background: #252526;
    border-radius: 4px;
  }

  .btn-export,
  .btn-import {
    width: 100%;
    padding: 10px 16px;
    background: #2da042;
    border: none;
    border-radius: 4px;
    color: white;
    font-size: 13px;
    cursor: pointer;
    font-weight: 500;
  }

  .btn-export:hover:not(:disabled),
  .btn-import:hover:not(:disabled) {
    background: #238736;
  }

  .btn-export:disabled,
  .btn-import:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-import {
    background: #007acc;
  }

  .btn-import:hover:not(:disabled) {
    background: #005a9e;
  }

  .progress-bar {
    height: 6px;
    background: #3e3e3e;
    border-radius: 3px;
    margin-top: 12px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #007acc, #00a8ff);
    transition: width 0.3s ease;
  }

  .progress-status {
    margin-top: 8px;
    font-size: 11px;
    color: #888;
    text-align: center;
  }

  .result-message {
    margin-top: 12px;
    padding: 12px;
    border-radius: 4px;
    font-size: 12px;
  }

  .result-message.success {
    background: rgba(78, 201, 176, 0.15);
    color: #4ec9b0;
  }

  .result-message.error {
    background: rgba(244, 135, 113, 0.15);
    color: #f48771;
  }

  .result-details {
    margin-top: 6px;
    font-size: 11px;
    opacity: 0.8;
    line-height: 1.5;
  }

  .btn-link {
    background: none;
    border: none;
    color: #007acc;
    font-size: 11px;
    cursor: pointer;
    padding: 0;
  }

  .btn-link:hover {
    text-decoration: underline;
  }

  .backup-content::-webkit-scrollbar,
  .tables-grid::-webkit-scrollbar {
    width: 8px;
  }

  .backup-content::-webkit-scrollbar-track,
  .tables-grid::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .backup-content::-webkit-scrollbar-thumb,
  .tables-grid::-webkit-scrollbar-thumb {
    background: #424242;
    border-radius: 4px;
  }

  .backup-content::-webkit-scrollbar-thumb:hover,
  .tables-grid::-webkit-scrollbar-thumb:hover {
    background: #4e4e4e;
  }
</style>
