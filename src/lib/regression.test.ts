import { describe, it, expect } from 'vitest';
import { parseInput } from './parser';
import type { Task, Status } from './types';

// Mock in-memory task store simulating the DB and service layer logic
class MockTaskStore {
  tasks: Task[] = [];

  private nextSortOrder(status: Status): number {
    const matching = this.tasks.filter((t) => t.status === status);
    if (matching.length === 0) return 1;
    const max = Math.max(...matching.map((t) => t.sort_order));
    return max + 1;
  }

  createTask(rawInput: string, status: Status = 'now'): Task {
    const trimmed = rawInput.trim();
    const parsed = parseInput(trimmed);
    const now = new Date().toISOString();
    const order = this.nextSortOrder(status);

    const task: Task = {
      id: Math.random().toString(36).slice(2),
      text: parsed.text || trimmed,
      raw_input: trimmed,
      status,
      context: parsed.context,
      priority: parsed.priority,
      due_at: parsed.due_at ?? null,
      created_at: now,
      updated_at: now,
      completed_at: status === 'done' ? now : null,
      sort_order: order,
    };

    this.tasks.push(task);
    return task;
  }

  editTask(id: string, newRawInput: string): Task {
    const task = this.tasks.find((t) => t.id === id);
    if (!task) throw new Error('Task not found');

    const trimmed = newRawInput.trim();
    const parsed = parseInput(trimmed);
    const now = new Date().toISOString();

    task.raw_input = trimmed;
    task.text = parsed.text || trimmed;
    task.context = parsed.context;
    task.priority = parsed.priority;
    task.due_at = parsed.due_at ?? null;
    task.updated_at = now;

    return task;
  }

  moveTask(id: string, newStatus: Status): Task {
    const task = this.tasks.find((t) => t.id === id);
    if (!task) throw new Error('Task not found');

    const now = new Date().toISOString();
    const order = this.nextSortOrder(newStatus);
    task.status = newStatus;
    task.sort_order = order;
    task.completed_at = newStatus === 'done' ? now : null;
    task.updated_at = now;

    return task;
  }

  reorder(status: Status, fromIndex: number, toIndex: number): void {
    const group = this.tasks
      .filter((t) => t.status === status)
      .sort((a, b) => a.sort_order - b.sort_order);

    if (fromIndex === toIndex || fromIndex < 0 || toIndex < 0) return;
    if (fromIndex >= group.length || toIndex >= group.length) return;

    const task = group[fromIndex];
    let newOrder: number;

    if (toIndex === 0) {
      newOrder = (group[0]?.sort_order ?? 1) - 1;
    } else if (toIndex >= group.length - 1) {
      newOrder = (group[group.length - 1]?.sort_order ?? 0) + 1;
    } else {
      const before = group[toIndex < fromIndex ? toIndex - 1 : toIndex];
      const after = group[toIndex < fromIndex ? toIndex : toIndex + 1];
      newOrder = before && after ? (before.sort_order + after.sort_order) / 2 : toIndex;
    }

    task.sort_order = newOrder;
    task.updated_at = new Date().toISOString();
  }
}

describe('Raw Input and Edit Consistency (Regression)', () => {
  it('updates raw_input and parsed fields when editing a task', () => {
    const store = new MockTaskStore();
    const task = store.createTask('Message Michael ~ rc **');

    expect(task.text).toBe('Message Michael');
    expect(task.context).toBe('rc');
    expect(task.priority).toBe(2);
    expect(task.raw_input).toBe('Message Michael ~ rc **');

    // Perform edit
    const updated = store.editTask(task.id, 'Message Edwin ~ work ***');

    expect(updated.raw_input).toBe('Message Edwin ~ work ***');
    expect(updated.text).toBe('Message Edwin');
    expect(updated.context).toBe('work');
    expect(updated.priority).toBe(3);

    // Subsequent edit check
    const secondEdit = store.editTask(task.id, 'Plain task without meta');
    expect(secondEdit.raw_input).toBe('Plain task without meta');
    expect(secondEdit.text).toBe('Plain task without meta');
    expect(secondEdit.context).toBeNull();
    expect(secondEdit.priority).toBe(0);
  });

  it('preserves raw_input when editing with special characters', () => {
    const store = new MockTaskStore();
    const task = store.createTask('Ship v1.0.0-rc.1 ~ release *');

    const updated = store.editTask(task.id, 'Ship v1.0.0-final ~ release *****');
    expect(updated.raw_input).toBe('Ship v1.0.0-final ~ release *****');
    expect(updated.priority).toBe(5);
    expect(updated.context).toBe('release');
  });
});

describe('Status Movement and Destination Ordering (Regression)', () => {
  it('assigns valid sort_order when moving between groups', () => {
    const store = new MockTaskStore();
    const task1 = store.createTask('Task 1', 'now');
    const task2 = store.createTask('Task 2', 'now');
    const later1 = store.createTask('Later 1', 'later');

    expect(later1.sort_order).toBe(1);

    // Move task1 from 'now' to 'later'
    const moved = store.moveTask(task1.id, 'later');
    expect(moved.status).toBe('later');
    expect(moved.sort_order).toBe(2);
    expect(moved.completed_at).toBeNull();

    // Move task2 to 'done' (done group is currently empty, next sort_order is 1)
    const completed = store.moveTask(task2.id, 'done');
    expect(completed.status).toBe('done');
    expect(completed.completed_at).not.toBeNull();
    expect(completed.sort_order).toBe(1);

    // Uncomplete task2 back to 'now'
    const restored = store.moveTask(task2.id, 'now');
    expect(restored.status).toBe('now');
    expect(restored.completed_at).toBeNull();
    expect(restored.sort_order).toBeGreaterThan(0);
  });

  it('clears completed_at when moving out of done status', () => {
    const store = new MockTaskStore();
    const task = store.createTask('Finish report', 'now');

    store.moveTask(task.id, 'done');
    expect(task.completed_at).not.toBeNull();

    store.moveTask(task.id, 'someday');
    expect(task.status).toBe('someday');
    expect(task.completed_at).toBeNull();
  });
});

describe('Reorder Calculations', () => {
  it('handles reordering in a 3-item list accurately', () => {
    const store = new MockTaskStore();
    const t1 = store.createTask('Item 1', 'now'); // sort_order: 1
    const t2 = store.createTask('Item 2', 'now'); // sort_order: 2
    const t3 = store.createTask('Item 3', 'now'); // sort_order: 3

    // Move t3 to top (index 2 -> 0)
    store.reorder('now', 2, 0);
    expect(t3.sort_order).toBe(0);

    // After reorder, list in order is [t3 (0), t1 (1), t2 (2)]
    // Move t3 back to bottom (index 0 -> 2)
    store.reorder('now', 0, 2);
    expect(t3.sort_order).toBe(3); // (t2.sort_order + 1 = 3)
  });

  it('safely ignores out of bounds reorder requests', () => {
    const store = new MockTaskStore();
    const t1 = store.createTask('Only Item', 'now');
    const originalOrder = t1.sort_order;

    store.reorder('now', 0, 5);
    expect(t1.sort_order).toBe(originalOrder);

    store.reorder('now', -1, 0);
    expect(t1.sort_order).toBe(originalOrder);
  });
});
