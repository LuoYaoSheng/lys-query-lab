import { writable } from 'svelte/store';

export const toastStore = writable([]);
export const confirmStore = writable(null);

let toastId = 0;

function pushToast(level, message, timeout = 3200) {
  const id = ++toastId;
  toastStore.update((items) => [...items, { id, level, message }]);

  if (timeout > 0) {
    setTimeout(() => dismissToast(id), timeout);
  }

  return id;
}

export function notifySuccess(message, timeout) {
  return pushToast('success', message, timeout);
}

export function notifyError(message, timeout = 4500) {
  return pushToast('error', message, timeout);
}

export function notifyInfo(message, timeout) {
  return pushToast('info', message, timeout);
}

export function dismissToast(id) {
  toastStore.update((items) => items.filter((item) => item.id !== id));
}

export function confirmAction({
  title = '请确认',
  message = '',
  confirmLabel = '确认',
  cancelLabel = '取消',
  tone = 'danger',
} = {}) {
  return new Promise((resolve) => {
    confirmStore.update((current) => {
      if (current?.resolve) {
        current.resolve(false);
      }
      return {
        title,
        message,
        confirmLabel,
        cancelLabel,
        tone,
        resolve,
      };
    });
  });
}

export function resolveConfirm(result) {
  confirmStore.update((current) => {
    if (current?.resolve) {
      current.resolve(result);
    }
    return null;
  });
}
