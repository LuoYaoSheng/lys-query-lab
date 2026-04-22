import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import DatabaseBackup from './DatabaseBackup.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
  open: vi.fn(),
}));

const { invoke: invokeMock } = await import('@tauri-apps/api/core');
const { save: saveMock, open: openMock } = await import('@tauri-apps/plugin-dialog');

describe('DatabaseBackup', () => {
  beforeEach(() => {
    invokeMock.mockReset();
    saveMock.mockReset();
    openMock.mockReset();
  });

  it('wraps db_export payload in params and uses backend field names', async () => {
    invokeMock.mockImplementation(async (command, payload) => {
      if (command === 'meta_list_tables') {
        expect(payload).toEqual({
          connection: { id: 'conn-1', host: 'localhost', port: 3306 },
          database: 'demo',
          includeViews: false,
        });
        return [{ name: 'users' }, { name: 'orders' }];
      }

      if (command === 'db_export') {
        expect(payload).toEqual({
          params: {
            connection: { id: 'conn-1', host: 'localhost', port: 3306 },
            database: 'demo',
            tables: ['users', 'orders'],
            export_type: 'both',
            format: 'sql',
            file_path: '/tmp/demo-backup.sql',
          },
        });
        return { size: 128, tables: 2 };
      }

      throw new Error(`unexpected invoke: ${command}`);
    });

    saveMock.mockResolvedValue('/tmp/demo-backup.sql');

    render(DatabaseBackup, {
      connection: { id: 'conn-1', host: 'localhost', port: 3306 },
      databases: ['demo'],
      onClose: () => {},
    });

    await fireEvent.change(screen.getByLabelText('选择数据库'), {
      target: { value: 'demo' },
    });

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('meta_list_tables', {
        connection: { id: 'conn-1', host: 'localhost', port: 3306 },
        database: 'demo',
        includeViews: false,
      });
    });

    await fireEvent.click(screen.getByRole('button', { name: /开始导出/ }));

    await waitFor(() => {
      expect(saveMock).toHaveBeenCalled();
      expect(invokeMock).toHaveBeenCalledWith(
        'db_export',
        expect.objectContaining({
          params: expect.any(Object),
        }),
      );
    });
  });
});
