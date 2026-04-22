<script>
  import { confirmStore, dismissToast, resolveConfirm, toastStore } from '../lib/notifications';
</script>

<div class="notification-center">
  <div class="toast-stack">
    {#each $toastStore as toast (toast.id)}
      <div class="toast {toast.level}">
        <span>{toast.message}</span>
        <button type="button" class="toast-close" on:click={() => dismissToast(toast.id)}>&times;</button>
      </div>
    {/each}
  </div>

  {#if $confirmStore}
    <div
      class="confirm-overlay"
      role="button"
      tabindex="0"
      aria-label="关闭确认对话框"
      on:click={(event) => event.target === event.currentTarget && resolveConfirm(false)}
      on:keydown={(event) => event.key === 'Escape' && resolveConfirm(false)}
    >
      <div class="confirm-dialog" role="dialog" aria-modal="true" aria-label={$confirmStore.title}>
        <div class="confirm-header">
          <h3>{$confirmStore.title}</h3>
        </div>
        <div class="confirm-body">
          <p>{$confirmStore.message}</p>
        </div>
        <div class="confirm-actions">
          <button type="button" class="btn-cancel" on:click={() => resolveConfirm(false)}>
            {$confirmStore.cancelLabel}
          </button>
          <button type="button" class="btn-confirm {$confirmStore.tone}" on:click={() => resolveConfirm(true)}>
            {$confirmStore.confirmLabel}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .notification-center {
    pointer-events: none;
  }

  .toast-stack {
    position: fixed;
    top: 16px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    z-index: 2000;
  }

  .toast {
    pointer-events: auto;
    min-width: 280px;
    max-width: 420px;
    padding: 12px 14px;
    border-radius: 8px;
    display: flex;
    justify-content: space-between;
    gap: 12px;
    align-items: flex-start;
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.28);
    color: #f4f4f4;
    font-size: 13px;
    line-height: 1.4;
  }

  .toast.success {
    background: #1f4b2a;
    border: 1px solid #2f7a45;
  }

  .toast.error {
    background: #4b2424;
    border: 1px solid #8a3a3a;
  }

  .toast.info {
    background: #1f364b;
    border: 1px solid #2f5f8a;
  }

  .toast-close {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    padding: 0;
  }

  .confirm-overlay {
    pointer-events: auto;
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
  }

  .confirm-dialog {
    width: 420px;
    max-width: calc(100vw - 32px);
    background: #252526;
    border: 1px solid #3e3e3e;
    border-radius: 10px;
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.35);
  }

  .confirm-header,
  .confirm-body,
  .confirm-actions {
    padding: 16px 18px;
  }

  .confirm-header {
    border-bottom: 1px solid #3e3e3e;
  }

  .confirm-header h3 {
    margin: 0;
    font-size: 16px;
    color: #f4f4f4;
  }

  .confirm-body {
    color: #d4d4d4;
    font-size: 13px;
    line-height: 1.5;
  }

  .confirm-body p {
    margin: 0;
  }

  .confirm-actions {
    border-top: 1px solid #3e3e3e;
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .btn-cancel,
  .btn-confirm {
    border: none;
    border-radius: 6px;
    padding: 8px 14px;
    font-size: 13px;
    cursor: pointer;
  }

  .btn-cancel {
    background: #3e3e3e;
    color: #d4d4d4;
  }

  .btn-confirm {
    color: white;
  }

  .btn-confirm.danger {
    background: #c73b3b;
  }

  .btn-confirm.info {
    background: #007acc;
  }
</style>
