import { describe, it, expect } from 'vitest';
import type { Task } from './types';

// Standalone fractional sort order calculation for testing
function computeNewSortOrder(tasks: Task[], fromIndex: number, toIndex: number): number {
  if (toIndex === 0) {
    return (tasks[0]?.sort_order ?? 1) - 1;
  } else if (toIndex >= tasks.length - 1) {
    return (tasks[tasks.length - 1]?.sort_order ?? 0) + 1;
  } else {
    const before = tasks[toIndex < fromIndex ? toIndex - 1 : toIndex];
    const after = tasks[toIndex < fromIndex ? toIndex : toIndex + 1];
    if (before && after) {
      return (before.sort_order + after.sort_order) / 2;
    }
    return toIndex;
  }
}

function mockTask(id: string, sort_order: number): Task {
  return {
    id,
    text: `Task ${id}`,
    raw_input: null,
    link: null,
    status: 'now',
    context: null,
    priority: 0,
    due_at: null,
    created_at: '2026-08-27T00:00:00.000Z',
    updated_at: '2026-08-27T00:00:00.000Z',
    completed_at: null,
    sort_order,
  };
}

describe('Task Reorder Calculation', () => {
  it('moves task to top of list with decrementing order', () => {
    const tasks = [mockTask('a', 1), mockTask('b', 2), mockTask('c', 3)];
    const newOrder = computeNewSortOrder(tasks, 2, 0); // move c to top
    expect(newOrder).toBe(0);
  });

  it('moves task to bottom of list with incrementing order', () => {
    const tasks = [mockTask('a', 1), mockTask('b', 2), mockTask('c', 3)];
    const newOrder = computeNewSortOrder(tasks, 0, 2); // move a to bottom
    expect(newOrder).toBe(4);
  });

  it('moves task between two neighbors with average order', () => {
    const tasks = [mockTask('a', 1), mockTask('b', 2), mockTask('c', 4), mockTask('d', 5)];
    const newOrder = computeNewSortOrder(tasks, 3, 2); // move d before c (between b and c)
    expect(newOrder).toBe(3); // (2 + 4) / 2 = 3
  });
});
