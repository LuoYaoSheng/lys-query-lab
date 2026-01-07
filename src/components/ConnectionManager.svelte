<script>
  import { invoke } from '@tauri-apps/api/core';

  export let connections = [];
  export let selectedConnection = null;
  export let onConnect = () => {};

  let showForm = false;
  let testingConnection = false;
  let savingConnection = false;
  let testResult = null;

  // 表单数据
  let formData = {
    id: '',
    name: '',
    driver: 'mysql',
    host: 'localhost',
    port: 33306,
    user: 'root',
    password: 'root123456',
    defaultDb: ''
  };

  function openNewForm() {
    testResult = null;
    formData = {
      id: '',
      name: '',
      driver: 'mysql',
      host: 'localhost',
      port: 33306,
      user: 'root',
      password: 'root123456',
      defaultDb: ''
    };
    showForm = true;
  }

  async function saveConnection() {
    savingConnection = true;
    testResult = null;
    try {
      const id = await invoke('conn_upsert', { connection: formData });
      console.log('Connection saved:', id);
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
    testingConnection = true;
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
      testingConnection = false;
    }
  }

  async function testFormConnection() {
    const testConn = { ...formData, id: '__test__' };
    await testConnection(testConn);
  }

  function selectConnection(conn) {
    selectedConnection = conn;
    onConnect(conn);
  }

  function closeForm() {
    showForm = false;
    testResult = null;
  }
</script>

<div class="connection-manager">
  <div class="connection-header">
    <span>连接</span>
    <button class="btn-icon" on:click={openNewForm} title="新建连接">+</button>
  </div>

  <div class="connection-list">
    {#each connections as conn}
      <div
        class="connection-item"
        class:connected={selectedConnection?.id === conn.id}
        class:testing={testingConnection === conn.id}
      >
        <div class="conn-info" on:click={() => selectConnection(conn)}>
          <span class="conn-name">{conn.name || conn.host}</span>
          <span class="conn-host">{conn.host}:{conn.port}</span>
        </div>
        <div class="conn-actions">
          <button
            class="btn-icon"
            on:click|stopPropagation={() => testConnection(conn)}
            title="测试连接"
            disabled={testingConnection === conn.id}
          >
            {testingConnection === conn.id ? '...' : '⚡'}
          </button>
        </div>
      </div>
    {:else}
      <div class="empty-state">暂无连接，点击 + 新建</div>
    {/each}
  </div>

  {#if showForm}
    <div class="connection-form-overlay" on:click={closeForm}>
      <div class="connection-form" on:click|stopPropagation>
        <h3>新建连接</h3>

        {#if testResult}
          <div class="test-result" class:success={testResult.success} class:error={!testResult.success}>
            <pre>{testResult.message}</pre>
          </div>
        {/if}

        <form on:submit|preventDefault={saveConnection}>
          <label>
            <span>连接名称</span>
            <input type="text" bind:value={formData.name} placeholder="HUB MySQL" required />
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
