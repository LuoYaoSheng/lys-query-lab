import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import ConnectionManager from './ConnectionManager.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('../lib/notifications', () => ({
  confirmAction: vi.fn(() => Promise.resolve(true)),
  notifyError: vi.fn(),
  notifySuccess: vi.fn(),
}));

const { invoke: invokeMock } = await import('@tauri-apps/api/core');
const { confirmAction } = await import('../lib/notifications');

describe('ConnectionManager', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    confirmAction.mockClear();
  });

  it('selects and tests a connection', async () => {
    const onConnect = vi.fn();

    invokeMock.mockImplementation(async (command, payload) => {
      if (command === 'conn_test') {
        expect(payload).toEqual({
          connection: {
            id: 'conn-1',
            name: 'Primary DB',
            host: 'db.internal',
            port: 3306,
            user: 'admin',
          },
        });
        return {
          latency_ms: 12,
          server_version: '8.0.0',
          user: 'admin',
          default_db: 'app',
        };
      }
      throw new Error(`unexpected invoke: ${command}`);
    });

    render(ConnectionManager, {
      connections: [
        {
          id: 'conn-1',
          name: 'Primary DB',
          host: 'db.internal',
          port: 3306,
          user: 'admin',
        },
      ],
      selectedConnection: null,
      onConnect,
    });

    await fireEvent.click(screen.getByRole('button', { name: /Primary DB/i }));
    expect(onConnect).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'conn-1', name: 'Primary DB' }),
    );

    await fireEvent.click(screen.getByTitle('测试连接'));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('conn_test', {
        connection: expect.objectContaining({ id: 'conn-1' }),
      });
    });

    expect(screen.getByText((content) => content.includes('连接成功!'))).toBeInTheDocument();
  });

  it('deletes a connection and refreshes the list', async () => {
    const onConnect = vi.fn();

    invokeMock.mockImplementation(async (command, payload) => {
      if (command === 'conn_delete') {
        expect(payload).toEqual({ id: 'conn-1' });
        return true;
      }
      if (command === 'conn_list') {
        return [];
      }
      throw new Error(`unexpected invoke: ${command}`);
    });

    render(ConnectionManager, {
      connections: [
        {
          id: 'conn-1',
          name: 'Primary DB',
          host: 'db.internal',
          port: 3306,
          user: 'admin',
        },
      ],
      selectedConnection: { id: 'conn-1' },
      onConnect,
    });

    await fireEvent.click(screen.getAllByTitle('删除连接')[0]);

    await waitFor(() => {
      expect(confirmAction).toHaveBeenCalled();
      expect(invokeMock).toHaveBeenCalledWith('conn_delete', { id: 'conn-1' });
      expect(invokeMock).toHaveBeenCalledWith('conn_list');
    });
  });

  it('opens the edit form with the current connection values', async () => {
    render(ConnectionManager, {
      connections: [
        {
          id: 'conn-1',
          name: 'Primary DB',
          host: 'db.internal',
          port: 3306,
          user: 'admin',
          defaultDb: 'app',
        },
      ],
      selectedConnection: null,
      onConnect: () => {},
    });

    await fireEvent.click(screen.getAllByTitle('编辑连接')[0]);

    expect(screen.getByRole('dialog', { name: '编辑连接' })).toBeInTheDocument();
    expect(screen.getByDisplayValue('Primary DB')).toBeInTheDocument();
    expect(screen.getByDisplayValue('db.internal')).toBeInTheDocument();
    expect(screen.getByDisplayValue('3306')).toBeInTheDocument();
    expect(screen.getByDisplayValue('admin')).toBeInTheDocument();
  });
});
