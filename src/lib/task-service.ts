import type { Task, Status } from './types';
import { parseInput } from './parser';
import * as db from './db';

/**
 * Task service — the single entry point for all task operations.
 * GUI, CLI, and future integrations all call through here.
 */

export async function captureTask(rawInput: string, status: Status = 'now'): Promise<Task | null> {
  const trimmed = rawInput.trim();
  if (!trimmed) return null;

  const parsed = parseInput(trimmed);
  if (!parsed.text) {
    // Even if parsing produced no structured text, store the raw input as-is.
    return db.createTask(trimmed, trimmed, null, 0, status, null);
  }

  return db.createTask(parsed.text, trimmed, parsed.context, parsed.priority, status, parsed.due_at ?? null);
}

export async function editTask(
  id: string,
  newRawInput: string
): Promise<void> {
  const parsed = parseInput(newRawInput.trim());
  await db.updateTask(id, {
    text: parsed.text || newRawInput.trim(),
    context: parsed.context,
    priority: parsed.priority,
    due_at: parsed.due_at ?? null,
  });
}

export async function completeTask(id: string): Promise<void> {
  await db.completeTask(id);
}

export async function uncompleteTask(id: string, restoreTo: Status = 'now'): Promise<void> {
  await db.uncompleteTask(id, restoreTo);
}

export async function moveTask(id: string, newStatus: Status): Promise<void> {
  if (newStatus === 'done') {
    await db.completeTask(id);
  } else {
    await db.updateTask(id, { status: newStatus });
  }
}

export async function deleteTask(id: string): Promise<void> {
  await db.deleteTask(id);
}

export async function loadAllTasks(): Promise<Task[]> {
  return db.getAllTasks();
}

export async function loadTasksByStatus(status: Status): Promise<Task[]> {
  return db.getTasksByStatus(status);
}

export async function searchTasks(query: string): Promise<Task[]> {
  return db.searchTasks(query);
}

/**
 * Swap task order within a status group.
 * `tasks` is the current ordered list; `fromIndex` and `toIndex` are positions within it.
 */
export async function reorderTasks(
  tasks: Task[],
  fromIndex: number,
  toIndex: number
): Promise<void> {
  if (fromIndex === toIndex) return;
  if (fromIndex < 0 || toIndex < 0) return;
  if (fromIndex >= tasks.length || toIndex >= tasks.length) return;

  const task = tasks[fromIndex];

  // Compute new sort_order by averaging neighbors at the target position.
  let newOrder: number;
  if (toIndex === 0) {
    newOrder = (tasks[0]?.sort_order ?? 1) - 1;
  } else if (toIndex >= tasks.length - 1) {
    newOrder = (tasks[tasks.length - 1]?.sort_order ?? 0) + 1;
  } else {
    // Place between the two neighbors at the target slot.
    const before = tasks[toIndex < fromIndex ? toIndex - 1 : toIndex];
    const after = tasks[toIndex < fromIndex ? toIndex : toIndex + 1];
    if (before && after) {
      newOrder = (before.sort_order + after.sort_order) / 2;
    } else {
      newOrder = toIndex;
    }
  }

  await db.reorderTask(task.id, newOrder);
}
