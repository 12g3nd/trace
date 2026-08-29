import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as db from './db';
import * as taskService from './task-service';
import type { Task } from './types';

vi.mock('./db', () => ({
  createTask: vi.fn(),
  updateTask: vi.fn(),
  moveTask: vi.fn(),
  completeTask: vi.fn(),
  uncompleteTask: vi.fn(),
  deleteTask: vi.fn(),
  reorderTask: vi.fn(),
  restoreTask: vi.fn(),
  getAllTasks: vi.fn(),
  getTasksByStatus: vi.fn(),
  searchTasks: vi.fn(),
}));

describe('taskService (Production Hardening Regressions)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('captureTask', () => {
    it('returns null for blank input without calling database', async () => {
      const result = await taskService.captureTask('   ');
      expect(result).toBeNull();
      expect(db.createTask).not.toHaveBeenCalled();
    });

    it('parses input and calls db.createTask with structured fields and raw_input', async () => {
      const mockCreatedTask: Task = {
        id: 'abc123',
        text: 'Deploy service',
        raw_input: 'Deploy service ~ infra **',
        link: null,
        status: 'now',
        context: 'infra',
        priority: 2,
        due_at: null,
        created_at: '2026-08-28T00:00:00.000Z',
        updated_at: '2026-08-28T00:00:00.000Z',
        completed_at: null,
        sort_order: 1,
      };
      vi.mocked(db.createTask).mockResolvedValue(mockCreatedTask);

      const res = await taskService.captureTask('Deploy service ~ infra **', 'now');

      expect(db.createTask).toHaveBeenCalledWith(
        'Deploy service',
        'Deploy service ~ infra **',
        'infra',
        2,
        'now',
        null,
        null
      );
      expect(res).toEqual(mockCreatedTask);
    });

    it('handles input where parsing produces no distinct text by using trimmed raw input', async () => {
      await taskService.captureTask('~infra', 'later');
      expect(db.createTask).toHaveBeenCalledWith('~infra', '~infra', null, 0, 'later', null, null);
    });
  });

  describe('editTask', () => {
    it('updates raw_input, parsed text, context, priority, and due_at in db', async () => {
      await taskService.editTask('task-1', 'Updated task name ~ work ***', 'docs.example.com/rc');

      expect(db.updateTask).toHaveBeenCalledTimes(1);
      expect(db.updateTask).toHaveBeenCalledWith('task-1', {
        raw_input: 'Updated task name ~ work ***',
        text: 'Updated task name',
        context: 'work',
        priority: 3,
        due_at: null,
        link: 'https://docs.example.com/rc',
      });
    });

    it('clears metadata fields if edited to plain text', async () => {
      await taskService.editTask('task-1', 'Simple plain text');

      expect(db.updateTask).toHaveBeenCalledWith('task-1', {
        raw_input: 'Simple plain text',
        text: 'Simple plain text',
        context: null,
        priority: 0,
        due_at: null,
        link: null,
      });
    });
  });

  describe('restoreTask', () => {
    it('passes original task snapshot directly to db.restoreTask', async () => {
      const originalTask: Task = {
        id: 'orig-id-99',
        text: 'Important task',
        raw_input: 'Important task ~p1',
        link: 'https://example.com/important',
        status: 'now',
        context: 'p1',
        priority: 1,
        due_at: '2026-08-30T12:00:00.000Z',
        created_at: '2026-08-27T10:00:00.000Z',
        updated_at: '2026-08-27T10:00:00.000Z',
        completed_at: null,
        sort_order: 3.5,
      };

      await taskService.restoreTask(originalTask);

      expect(db.restoreTask).toHaveBeenCalledTimes(1);
      expect(db.restoreTask).toHaveBeenCalledWith(originalTask);
    });
  });

  describe('moveTask and completeTask', () => {
    it('delegates moveTask to db.moveTask with new status', async () => {
      await taskService.moveTask('task-1', 'someday');
      expect(db.moveTask).toHaveBeenCalledWith('task-1', 'someday');
    });

    it('delegates completeTask and uncompleteTask to db', async () => {
      await taskService.completeTask('task-1');
      expect(db.completeTask).toHaveBeenCalledWith('task-1');

      await taskService.uncompleteTask('task-1', 'later');
      expect(db.uncompleteTask).toHaveBeenCalledWith('task-1', 'later');
    });
  });

  describe('reorderTasks', () => {
    const makeTask = (id: string, sort_order: number): Task => ({
      id,
      text: `Task ${id}`,
      raw_input: `Task ${id}`,
      link: null,
      status: 'now',
      context: null,
      priority: 0,
      due_at: null,
      created_at: '2026-08-28T00:00:00.000Z',
      updated_at: '2026-08-28T00:00:00.000Z',
      completed_at: null,
      sort_order,
    });

    it('does nothing if fromIndex equals toIndex or indices are out of bounds', async () => {
      const list = [makeTask('1', 1), makeTask('2', 2)];

      await taskService.reorderTasks(list, 0, 0);
      expect(db.reorderTask).not.toHaveBeenCalled();

      await taskService.reorderTasks(list, -1, 1);
      expect(db.reorderTask).not.toHaveBeenCalled();

      await taskService.reorderTasks(list, 0, 5);
      expect(db.reorderTask).not.toHaveBeenCalled();
    });

    it('computes new sort_order when moving to top (toIndex = 0)', async () => {
      const list = [makeTask('1', 10), makeTask('2', 20), makeTask('3', 30)];
      // Moving task 3 to index 0: new order should be first.sort_order - 1 = 9
      await taskService.reorderTasks(list, 2, 0);
      expect(db.reorderTask).toHaveBeenCalledWith('3', 9);
    });

    it('computes new sort_order when moving to bottom (toIndex = last)', async () => {
      const list = [makeTask('1', 10), makeTask('2', 20), makeTask('3', 30)];
      // Moving task 1 to index 2: new order should be last.sort_order + 1 = 31
      await taskService.reorderTasks(list, 0, 2);
      expect(db.reorderTask).toHaveBeenCalledWith('1', 31);
    });

    it('computes average of neighbor sort_orders when moving between items', async () => {
      const list = [makeTask('1', 10), makeTask('2', 20), makeTask('3', 30), makeTask('4', 40)];
      // Moving task 4 (fromIndex 3) up to toIndex 1 (between task 1 and task 2)
      // before = list[0] (10), after = list[1] (20) -> (10 + 20) / 2 = 15
      await taskService.reorderTasks(list, 3, 1);
      expect(db.reorderTask).toHaveBeenCalledWith('4', 15);
    });
  });
});
