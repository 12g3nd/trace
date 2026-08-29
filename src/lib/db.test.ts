import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Task } from './types';

const sql = vi.hoisted(() => ({
  execute: vi.fn(),
  select: vi.fn(),
  load: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-sql', () => ({
  default: {
    load: sql.load,
  },
}));

import { getAllTasks, initDb, restoreTask } from './db';

const linkedTask: Task = {
  id: 'linked-1',
  text: 'Review release candidate',
  raw_input: 'Review release candidate ~rc **',
  link: 'https://example.com/rc',
  status: 'now',
  context: 'rc',
  priority: 2,
  due_at: null,
  created_at: '2026-08-29T12:00:00.000Z',
  updated_at: '2026-08-29T12:00:00.000Z',
  completed_at: null,
  sort_order: 4,
};

describe('task link schema persistence', () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    sql.load.mockResolvedValue(sql);
    sql.execute.mockResolvedValue({ rowsAffected: 1 });
    sql.select.mockImplementation(async (query: string) => {
      if (query.includes('SELECT version FROM migrations')) return [];
      return [];
    });
    await initDb();
  });

  it('applies a backward-compatible nullable link migration', () => {
    const statements = sql.execute.mock.calls.map(([statement]) => statement);
    expect(statements).toContain('ALTER TABLE tasks ADD COLUMN link TEXT');
    expect(statements).toContain('INSERT INTO migrations (version) VALUES ($1)');
  });

  it('maps link values returned by SQLite', async () => {
    sql.select.mockResolvedValueOnce([linkedTask]);
    await expect(getAllTasks()).resolves.toEqual([linkedTask]);
  });

  it('restores the link with the complete task snapshot', async () => {
    sql.execute.mockClear();
    await restoreTask(linkedTask);

    expect(sql.execute).toHaveBeenCalledTimes(1);
    const [statement, values] = sql.execute.mock.calls[0];
    expect(statement).toContain('(id, text, raw_input, link, status');
    expect(values[3]).toBe('https://example.com/rc');
  });
});
