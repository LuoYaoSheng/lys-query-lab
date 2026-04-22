<script>
  import { invoke } from '@tauri-apps/api/core';
  import { confirmAction, notifyError, notifySuccess } from '../lib/notifications';

  export let connections = [];
  export let selectedConnection = null;
  export let onConnect = () => {};

  let showForm = false;
  let testingConnection = '';
  let savingConnection = false;
  let testResult = null;
  let editingConnection = false;

  // 表单数据
  let formData = {
    id: '',
    name: '',
    driver: 'mysql',
    host: '',
    port: 3306,
    user: '',
    password: '',
    defaultDb: ''
  };

  function openNewForm() {
    testResult = null;
    editingConnection = false;
    formData = {
      id: '',
      name: '',
      driver: 'mysql',
      host: '',
      port: 3306,
      user: '',
      password: '',
      defaultDb: ''
    };
    showForm = true;
  }

  function openEditForm(conn) {
    testResult = null;
    editingConnection = true;
    formData = {
      id: conn.id || '',
      name: conn.name || '',
      driver: conn.driver || conn.driver_type || 'mysql',
      host: conn.host || '',
      port: conn.port || 3306,
      user: conn.user || '',
      password: conn.password || '',
      defaultDb: conn.defaultDb || conn.default_db || ''
    };
    showForm = true;
  }

  async function saveConnection() {
    savingConnection = true;
    testResult = null;
    try {
      const id = await invoke('conn_upsert', { connection: formData });
      connections = await invoke('conn_list');
      showForm = false;
    } catch (err) {
      console.error('Save error:', err);
      testResult = { success: false, message: '保存失败: ' + err };
    } finally {
      savingConnection = false;
    }
  }

  async function testConnection(conn) {
    testingConnection = conn.id || '__test__';
    testResult = null;
    try {
      const result = await invoke('conn_test', { connection: conn });
      testResult = {
        success: true,
        message: `连接成功!\n延迟: ${result.latency_ms}ms\n版本: ${result.server_version}\n用户: ${result.user}`
      };
    } catch (err) {
      console.error('Test error:', err);
      testResult = { success: false, message: '连接失败: ' + err };
    } finally {
      testingConnection = '';
    }
  }

  async function testFormConnection() {
    const testConn = { ...formData, id: '__test__' };
    await testConnection(testConn);
  }

  async function deleteConnection(conn) {
    const confirmed = await confirmAction({
      title: '删除连接',
      message: `确定要删除连接 “${conn.name || conn.host}” 吗？`,
      confirmLabel: '删除',
      cancelLabel: '取消',
      tone: 'danger',
    });
    if (!confirmed) return;

    testResult = null;
    try {
      await invoke('conn_delete', { id: conn.id });
      connections = await invoke('conn_list');
      if (selectedConnection?.id === conn.id) {
        selectedConnection = null;
        onConnect(null);
      }
      notifySuccess('连接已删除');
    } catch (err) {
      console.error('Delete error:', err);
      notifyError('删除失败: ' + err);
    }
  }

  function selectConnection(conn) {
    selectedConnection = conn;
    onConnect(conn);
  }

  function closeForm() {
    showForm = false;
    testResult = null;
    editingConnection = false;
  }

  function handleOverlayClick(event) {
    if (event.target === event.currentTarget) {
      closeForm();
    }
  }

  function handleOverlayKeydown(event) {
    if (event.key === 'Escape') {
      closeForm();
    }
  }
</script>

<div class="connection-manager">
  <div class="connection-header">
    <span>连接</span>
    <button class="btn-icon" on:click={openNewForm} title="新建连接">+</button>
  </div>

  {#if testResult && !showForm}
    <div class="test-result" class:success={testResult.success} class:error={!testResult.success}>
      <pre>{testResult.message}</pre>
    </div>
  {/if}

  <div class="connection-list">
    {#each connections as conn}
      <div
        class="connection-item"
        class:connected={selectedConnection?.id === conn.id}
        class:testing={testingConnection === conn.id}
      >
        <button type="button" class="conn-info" on:click={() => selectConnection(conn)}>
          <span class="conn-name">{conn.name || conn.host}</span>
          <span class="conn-host">{conn.host}:{conn.port}</span>
        </button>
        <div class="conn-actions">
          <button
            class="btn-icon"
            on:click|stopPropagation={() => openEditForm(conn)}
            title="编辑连接"
          >
            ✎
          </button>
          <button
            class="btn-icon"
            on:click|stopPropagation={() => testConnection(conn)}
            title="测试连接"
            disabled={testingConnection === conn.id}
          >
            {testingConnection === conn.id ? '...' : '⚡'}
          </button>
          <button
            class="btn-icon btn-danger"
            on:click|stopPropagation={() => deleteConnection(conn)}
            title="删除连接"
          >
            ✕
          </button>
        </div>
      </div>
    {:else}
      <div class="empty-state">暂无连接，点击 + 新建</div>
    {/each}
  </div>

  {#if showForm}
    <div
      class="connection-form-overlay"
      role="button"
      tabindex="0"
      aria-label="关闭连接表单"
      on:click={handleOverlayClick}
      on:keydown={handleOverlayKeydown}
    >
      <div class="connection-form" role="dialog" aria-modal="true" aria-label={editingConnection ? '编辑连接' : '新建连接'}>
        <h3>{editingConnection ? '编辑连接' : '新建连接'}</h3>

        {#if testResult}
          <div class="test-result" class:success={testResult.success} class:error={!testResult.success}>
            <pre>{testResult.message}</pre>
          </div>
        {/if}

        <form on:submit|preventDefault={saveConnection}>
          <label>
            <span>连接名称</span>
            <input type="text" bind:value={formData.name} placeholder="例如：生产 MySQL" required />
          </label>
          <label>
            <span>主机</span>
            <input type="text" bind:value={formData.host} placeholder="localhost" required />
          </label>
          <label>
            <span>端口</span>
            <input type="number" bind:value={formData.port} required />
          </label>
          <label>
            <span>用户</span>
            <input type="text" bind:value={formData.user} placeholder="root" required />
          </label>
          <label>
            <span>密码</span>
            <input type="password" bind:value={formData.password} />
          </label>
          <div class="form-actions">
            <button type="button" class="btn-secondary" on:click={closeForm}>取消</button>
            <button
              type="button"
              class="btn-secondary"
              on:click={testFormConnection}
              disabled={testingConnection}
            >
              {testingConnection ? '测试中...' : '测试'}
            </button>
            <button type="submit" class="btn-primary" disabled={savingConnection}>
              {savingConnection ? '保存中...' : '保存'}
            </button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  .connection-manager {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .connection-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: #888;
    border-bottom: 1px solid #3e3e3e;
  }

  .connection-list {
    flex: 1;
    overflow-y: auto;
  }

  .connection-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 16px;
    cursor: pointer;
    border-bottom: 1px solid #2d2d2d;
  }

  .connection-item:hover {
    background: #2a2d2e;
  }

  .connection-item.connected {
    background: #37373d;
    border-left: 3px solid #007acc;
  }

  .conn-info {
    flex: 1;
    cursor: pointer;
    background: transparent;
    border: none;
    padding: 0;
    text-align: left;
    color: inherit;
    font: inherit;
  }

  .conn-name {
    display: block;
    font-size: 13px;
    font-weight: 500;
  }

  .conn-host {
    display: block;
    font-size: 11px;
    color: #888;
  }

  .conn-actions {
    display: flex;
    gap: 4px;
    opacity: 0.5;
  }

  .connection-item:hover .conn-actions {
    opacity: 1;
  }

  .btn-icon {
    background: transparent;
    border: none;
    color: #d4d4d4;
    padding: 4px 6px;
    cursor: pointer;
    border-radius: 3px;
    font-size: 12px;
  }

  .btn-icon:hover:not(:disabled) {
    background: #3e3e3e;
  }

  .btn-icon:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-danger {
    color: #f48771;
  }

  .btn-danger:hover:not(:disabled) {
    background: #4a2a2a;
  }

  .empty-state {
    padding: 32px 16px;
    text-align: center;
    color: #666;
    font-size: 13px;
  }

  .connection-form-overlay {
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

  .connection-form {
    background: #252526;
    border: 1px solid #3e3e3e;
    border-radius: 8px;
    padding: 24px;
    width: 400px;
    max-width: 90vw;
  }

  .connection-form h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
  }

  .test-result {
    padding: 10px 12px;
    border-radius: 4px;
    margin-bottom: 12px;
    font-size: 13px;
  }

  .test-result pre {
    margin: 0;
    white-space: pre-wrap;
    font-family: inherit;
  }

  .test-result.success {
    background: #1e3a1e;
    color: #4ec9b0;
    border: 1px solid #2d5a2d;
  }

  .test-result.error {
    background: #3c1f1e;
    color: #f48771;
    border: 1px solid #5a2d2d;
  }

  .connection-form label {
    display: block;
    margin-bottom: 12px;
  }

  .connection-form label span {
    display: block;
    font-size: 12px;
    color: #888;
    margin-bottom: 4px;
  }

  .connection-form input {
    width: 100%;
    padding: 8px 12px;
    background: #3c3c3c;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #d4d4d4;
    font-size: 13px;
    box-sizing: border-box;
  }

  .connection-form input:focus {
    outline: none;
    border-color: #007acc;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }

  .btn-primary, .btn-secondary {
    padding: 8px 16px;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .btn-primary:disabled, .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #007acc;
    color: white;
    border: none;
  }

  .btn-primary:hover:not(:disabled) {
    background: #006bb3;
  }

  .btn-secondary {
    background: #3e3e3e;
    color: #d4d4d4;
    border: none;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4e4e4e;
  }
</style>
