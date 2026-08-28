import Database from '@tauri-apps/plugin-sql';
import type { Task, Status } from './types';

let db: Database | null = null;

/** Connect to the SQLite database and run migrations. */
export async function initDb(): Promise<void> {
  db = await Database.load('sqlite:trace.db');
  await migrate();
}

function getDb(): Database {
  if (!db) throw new Error('Database not initialized — call initDb() first');
  return db;
}

// ─── Migrations ──────────────────────────────────────────────────────────────

const MIGRATIONS: string[] = [
  // v1: initial schema
  `CREATE TABLE IF NOT EXISTS tasks (
    id          TEXT PRIMARY KEY,
    text        TEXT NOT NULL,
    raw_input   TEXT,
    status      TEXT NOT NULL DEFAULT 'now'
                  CHECK (status IN ('now', 'later', 'someday', 'done')),
    context     TEXT,
    priority    INTEGER NOT NULL DEFAULT 0
                  CHECK (priority BETWEEN 0 AND 5),
    due_at      TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    completed_at TEXT,
    sort_order  REAL NOT NULL DEFAULT 0
  );
  CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
  CREATE INDEX IF NOT EXISTS idx_tasks_context ON tasks(context);`,
];

async function migrate(): Promise<void> {
  const d = getDb();

  // Track applied migrations with a simple table.
  await d.execute(`
    CREATE TABLE IF NOT EXISTS migrations (
      version     INTEGER PRIMARY KEY,
      applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
    )
  `);

  const applied = await d.select<{ version: number }[]>(
    'SELECT version FROM migrations ORDER BY version'
  );
  const appliedSet = new Set(applied.map((r) => r.version));

  for (let i = 0; i < MIGRATIONS.length; i++) {
    if (appliedSet.has(i)) continue;

    // SQLite doesn't support multi-statement execute in all drivers,
    // so split on semicolons and run each individually.
    const statements = MIGRATIONS[i]
      .split(';')
      .map((s) => s.trim())
      .filter(Boolean);

    for (const stmt of statements) {
      await d.execute(stmt);
    }

    await d.execute('INSERT INTO migrations (version) VALUES ($1)', [i]);
  }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function generateId(): string {
  const bytes = new Uint8Array(8);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
}

function isoNow(): string {
  return new Date().toISOString();
}

// ─── Queries ─────────────────────────────────────────────────────────────────

export async function getAllTasks(): Promise<Task[]> {
  const d = getDb();
  return d.select<Task[]>(
    'SELECT * FROM tasks ORDER BY status, sort_order, created_at'
  );
}

export async function getTasksByStatus(status: Status): Promise<Task[]> {
  const d = getDb();
  return d.select<Task[]>(
    'SELECT * FROM tasks WHERE status = $1 ORDER BY sort_order, created_at',
    [status]
  );
}

/** Get the next sort_order value for a given status group. */
async function nextSortOrder(status: Status): Promise<number> {
  const d = getDb();
  const rows = await d.select<{ max_order: number | null }[]>(
    'SELECT MAX(sort_order) as max_order FROM tasks WHERE status = $1',
    [status]
  );
  const max = rows[0]?.max_order ?? 0;
  return max + 1;
}

export async function createTask(
  text: string,
  rawInput: string | null,
  context: string | null,
  priority: number,
  status: Status = 'now',
  dueAt: string | null = null
): Promise<Task> {
  const d = getDb();
  const id = generateId();
  const now = isoNow();
  const order = await nextSortOrder(status);

  await d.execute(
    `INSERT INTO tasks (id, text, raw_input, status, context, priority, due_at, created_at, updated_at, sort_order)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`,
    [id, text, rawInput, status, context, priority, dueAt, now, now, order]
  );

  return {
    id,
    text,
    raw_input: rawInput,
    status,
    context,
    priority,
    due_at: dueAt,
    created_at: now,
    updated_at: now,
    completed_at: null,
    sort_order: order,
  };
}

export async function updateTask(
  id: string,
  fields: Partial<Pick<Task, 'text' | 'raw_input' | 'context' | 'priority' | 'status' | 'sort_order' | 'due_at'>>
): Promise<void> {
  const d = getDb();
  const sets: string[] = [];
  const values: unknown[] = [];
  let paramIndex = 1;

  for (const [key, value] of Object.entries(fields)) {
    sets.push(`${key} = $${paramIndex}`);
    values.push(value);
    paramIndex++;
  }

  if (sets.length === 0) return;

  sets.push(`updated_at = $${paramIndex}`);
  values.push(isoNow());
  paramIndex++;

  values.push(id);
  await d.execute(
    `UPDATE tasks SET ${sets.join(', ')} WHERE id = $${paramIndex}`,
    values
  );
}

export async function moveTask(id: string, newStatus: Status): Promise<void> {
  const d = getDb();
  const now = isoNow();
  const order = await nextSortOrder(newStatus);
  const completedAt = newStatus === 'done' ? now : null;

  await d.execute(
    `UPDATE tasks SET status = $1, completed_at = $2, updated_at = $3, sort_order = $4 WHERE id = $5`,
    [newStatus, completedAt, now, order, id]
  );
}

export async function completeTask(id: string): Promise<void> {
  await moveTask(id, 'done');
}

export async function uncompleteTask(id: string, restoreStatus: Status = 'now'): Promise<void> {
  await moveTask(id, restoreStatus);
}

export async function deleteTask(id: string): Promise<void> {
  const d = getDb();
  await d.execute('DELETE FROM tasks WHERE id = $1', [id]);
}

export async function reorderTask(id: string, newOrder: number): Promise<void> {
  const d = getDb();
  await d.execute(
    'UPDATE tasks SET sort_order = $1, updated_at = $2 WHERE id = $3',
    [newOrder, isoNow(), id]
  );
}

export async function searchTasks(query: string): Promise<Task[]> {
  const d = getDb();
  const pattern = `%${query}%`;
  return d.select<Task[]>(
    `SELECT * FROM tasks
     WHERE text LIKE $1 OR context LIKE $1 OR raw_input LIKE $1
     ORDER BY
       CASE status WHEN 'now' THEN 0 WHEN 'later' THEN 1 WHEN 'someday' THEN 2 ELSE 3 END,
       sort_order`,
    [pattern]
  );
}
