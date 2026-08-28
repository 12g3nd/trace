import { writable, derived, get } from 'svelte/store';
import type { Task, Status } from './types';
import * as taskService from './task-service';

// ─── Core state ──────────────────────────────────────────────────────────────

export const tasks = writable<Task[]>([]);
export const selectedId = writable<string | null>(null);
export const searchQuery = writable<string>('');
export const searchActive = writable<boolean>(false);

// ─── Undo stack ──────────────────────────────────────────────────────────────

interface UndoEntry {
  label: string;
  undo: () => Promise<void>;
}

export const undoStack = writable<UndoEntry | null>(null);

let undoTimer: ReturnType<typeof setTimeout> | null = null;

function pushUndo(entry: UndoEntry) {
  if (undoTimer) clearTimeout(undoTimer);
  undoStack.set(entry);
  // Auto-dismiss after 6 seconds.
  undoTimer = setTimeout(() => undoStack.set(null), 6000);
}

export async function performUndo() {
  const entry = get(undoStack);
  if (!entry) return;
  if (undoTimer) clearTimeout(undoTimer);
  undoStack.set(null);
  await entry.undo();
  await refresh();
}

// ─── Derived views ───────────────────────────────────────────────────────────

function filterByStatus(allTasks: Task[], status: Status): Task[] {
  return allTasks
    .filter((t) => t.status === status)
    .sort((a, b) => a.sort_order - b.sort_order);
}

export const nowTasks = derived(tasks, ($t) => filterByStatus($t, 'now'));
export const laterTasks = derived(tasks, ($t) => filterByStatus($t, 'later'));
export const somedayTasks = derived(tasks, ($t) => filterByStatus($t, 'someday'));
export const doneTasks = derived(tasks, ($t) => filterByStatus($t, 'done'));

/** All visible (non-done) tasks in display order, for keyboard navigation. */
export const visibleTasks = derived(
  [nowTasks, laterTasks, somedayTasks],
  ([$now, $later, $someday]) => [...$now, ...$later, ...$someday]
);

export const taskCounts = derived(tasks, ($t) => {
  const counts: Record<Status, number> = { now: 0, later: 0, someday: 0, done: 0 };
  for (const task of $t) counts[task.status]++;
  return counts;
});

// ─── Filtered (search) view ──────────────────────────────────────────────────

export const filteredTasks = derived(
  [tasks, searchQuery],
  ([$tasks, $query]) => {
    if (!$query.trim()) return [];
    const q = $query.trim().toLowerCase();

    // Context search: "~ context" or just "~context"
    if (q.startsWith('~')) {
      const ctx = q.slice(1).trim();
      if (!ctx) return $tasks;
      return $tasks.filter((t) => t.context?.toLowerCase().includes(ctx));
    }

    // General text search
    return $tasks.filter(
      (t) =>
        t.text.toLowerCase().includes(q) ||
        t.context?.toLowerCase().includes(q) ||
        t.raw_input?.toLowerCase().includes(q)
    );
  }
);

// ─── Actions ─────────────────────────────────────────────────────────────────

export async function refresh() {
  const all = await taskService.loadAllTasks();
  tasks.set(all);
}

export async function capture(rawInput: string, status: Status = 'now') {
  const task = await taskService.captureTask(rawInput, status);
  if (task) await refresh();
  return task;
}

export async function complete(id: string) {
  const all = get(tasks);
  const task = all.find((t) => t.id === id);
  if (!task || task.status === 'done') return;

  const previousStatus = task.status;
  await taskService.completeTask(id);
  await refresh();

  pushUndo({
    label: `"${task.text}" completed`,
    undo: async () => {
      await taskService.uncompleteTask(id, previousStatus);
    },
  });
}

export async function uncomplete(id: string) {
  await taskService.uncompleteTask(id);
  await refresh();
}

export async function move(id: string, newStatus: Status) {
  const all = get(tasks);
  const task = all.find((t) => t.id === id);
  if (!task) return;

  const previousStatus = task.status;
  await taskService.moveTask(id, newStatus);
  await refresh();

  pushUndo({
    label: `Moved to ${newStatus}`,
    undo: async () => {
      await taskService.moveTask(id, previousStatus);
    },
  });
}

export async function edit(id: string, newRawInput: string) {
  await taskService.editTask(id, newRawInput);
  await refresh();
}

export async function remove(id: string) {
  const all = get(tasks);
  const task = all.find((t) => t.id === id);
  if (!task) return;

  const snapshot = { ...task };
  await taskService.deleteTask(id);
  await refresh();

  pushUndo({
    label: `"${snapshot.text}" deleted`,
    undo: async () => {
      await taskService.restoreTask(snapshot);
    },
  });
}

export async function reorder(
  statusTasks: Task[],
  fromIndex: number,
  toIndex: number
) {
  await taskService.reorderTasks(statusTasks, fromIndex, toIndex);
  await refresh();
}

// ─── Keyboard selection helpers ──────────────────────────────────────────────

export function selectNext() {
  const visible = get(visibleTasks);
  const current = get(selectedId);
  if (visible.length === 0) return;

  if (!current) {
    selectedId.set(visible[0].id);
    return;
  }

  const idx = visible.findIndex((t) => t.id === current);
  if (idx < visible.length - 1) {
    selectedId.set(visible[idx + 1].id);
  }
}

export function selectPrev() {
  const visible = get(visibleTasks);
  const current = get(selectedId);
  if (visible.length === 0) return;

  if (!current) {
    selectedId.set(visible[visible.length - 1].id);
    return;
  }

  const idx = visible.findIndex((t) => t.id === current);
  if (idx > 0) {
    selectedId.set(visible[idx - 1].id);
  }
}

export function clearSelection() {
  selectedId.set(null);
}
