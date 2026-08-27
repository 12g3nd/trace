import type { Task } from './types';

/**
 * Format tasks as standard plain text (TODO.txt compatible).
 */
export function formatAsTodoTxt(tasks: Task[]): string {
  const priorityLetters: Record<number, string> = {
    5: '(A) ',
    4: '(B) ',
    3: '(C) ',
    2: '(D) ',
    1: '(E) ',
  };

  return tasks
    .map((task) => {
      let line = '';

      if (task.status === 'done') {
        const completedDate = task.completed_at
          ? task.completed_at.slice(0, 10)
          : new Date().toISOString().slice(0, 10);
        const createdDate = task.created_at.slice(0, 10);
        line += `x ${completedDate} ${createdDate} `;
      } else if (task.priority > 0 && priorityLetters[task.priority]) {
        line += priorityLetters[task.priority];
      }

      line += task.text;

      if (task.context) {
        line += ` ~${task.context}`;
      }

      if (task.status !== 'now' && task.status !== 'done') {
        line += ` status:${task.status}`;
      }

      return line;
    })
    .join('\n');
}

/**
 * Format tasks as structured JSON with pretty printing.
 */
export function formatAsJson(tasks: Task[]): string {
  return JSON.stringify(
    {
      version: '1.0',
      exported_at: new Date().toISOString(),
      tasks,
    },
    null,
    2
  );
}

/**
 * Format tasks as spreadsheet-compatible CSV.
 */
export function formatAsCsv(tasks: Task[]): string {
  const headers = ['id', 'text', 'status', 'context', 'priority', 'created_at', 'completed_at'];
  
  function escapeCsv(val: string | number | null | undefined): string {
    if (val === null || val === undefined) return '';
    const str = String(val);
    if (str.includes(',') || str.includes('"') || str.includes('\n')) {
      return `"${str.replace(/"/g, '""')}"`;
    }
    return str;
  }

  const rows = tasks.map((t) => [
    escapeCsv(t.id),
    escapeCsv(t.text),
    escapeCsv(t.status),
    escapeCsv(t.context),
    escapeCsv(t.priority),
    escapeCsv(t.created_at),
    escapeCsv(t.completed_at),
  ]);

  return [headers.join(','), ...rows.map((r) => r.join(','))].join('\n');
}

/**
 * Trigger browser/webview file download for export.
 */
export function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
