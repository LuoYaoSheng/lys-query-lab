import { render, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import DataGrid from './DataGrid.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
}));

const { invoke: invokeMock } = await import('@tauri-apps/api/core');

describe('DataGrid', () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it('loads schema and data with the expected Tauri command payloads', async () => {
    invokeMock.mockImplementation(async (command, payload) => {
      if (command === 'meta_get_table_schema') {
        return {
          columns: [{ name: 'id', extra: 'auto_increment' }],
          indexes: [{ name: 'PRIMARY', columns: ['id'] }],
        };
      }

      if (command === 'query_execute' && payload.sql.includes('COUNT(*)')) {
        return {
          sets: [
            {
              columns: [{ name: 'total_count' }],
              chunks: [{ rows: [[3]] }],
              meta: { affectedRows: 1 },
            },
          ],
        };
      }

      if (command === 'query_execute' && payload.sql.includes('SELECT *')) {
        return {
          sets: [
            {
              columns: [{ name: 'id' }, { name: 'name' }],
              chunks: [{ rows: [[1, 'alpha'], [2, 'beta']] }],
              meta: { affectedRows: 2 },
            },
          ],
        };
      }

      throw new Error(`unexpected invoke: ${command}`);
    });

    render(DataGrid, {
      connection: { id: 'conn-1', host: 'localhost', port: 3306 },
      tableName: 'demo.users',
      onRefresh: () => {},
    });

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith('meta_get_table_schema', {
        connection: { id: 'conn-1', host: 'localhost', port: 3306 },
        database: 'demo',
        table: 'users',
      });
    });

    await waitFor(() => {
      expect(
        invokeMock.mock.calls.some(
          ([command, payload]) =>
            command === 'query_execute' &&
            payload.connection.id === 'conn-1' &&
            payload.maxRows === 1,
        ),
      ).toBe(true);

      expect(
        invokeMock.mock.calls.some(
          ([command, payload]) =>
            command === 'query_execute' &&
            payload.connection.id === 'conn-1' &&
            payload.maxRows === 50,
        ),
      ).toBe(true);
    });
  });
});
