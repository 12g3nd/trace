import { describe, it, expect } from 'vitest';
import { formatAsTodoTxt, formatAsJson, formatAsCsv } from './export';
import type { Task } from './types';

const mockTasks: Task[] = [
  {
    id: '1',
    text: 'Cancel DigitalOcean',
    raw_input: 'Cancel DigitalOcean ~ krine',
    link: 'https://cloud.digitalocean.com/',
    status: 'now',
    context: 'krine',
    priority: 0,
    due_at: null,
    created_at: '2026-08-27T12:00:00.000Z',
    updated_at: '2026-08-27T12:00:00.000Z',
    completed_at: null,
    sort_order: 1,
  },
  {
    id: '2',
    text: 'Write essay',
    raw_input: 'Write essay ~ chatgpt *****',
    link: null,
    status: 'now',
    context: 'chatgpt',
    priority: 5,
    due_at: null,
    created_at: '2026-08-27T12:00:00.000Z',
    updated_at: '2026-08-27T12:00:00.000Z',
    completed_at: null,
    sort_order: 2,
  },
  {
    id: '3',
    text: 'Completed chore',
    raw_input: 'Completed chore',
    link: null,
    status: 'done',
    context: null,
    priority: 0,
    due_at: null,
    created_at: '2026-08-27T10:00:00.000Z',
    updated_at: '2026-08-27T11:00:00.000Z',
    completed_at: '2026-08-27T11:00:00.000Z',
    sort_order: 3,
  },
];

describe('Export Formatters', () => {
  it('formats tasks as TODO.txt plain text', () => {
    const output = formatAsTodoTxt(mockTasks);
    expect(output).toContain('Cancel DigitalOcean ~krine');
    expect(output).toContain('(A) Write essay ~chatgpt');
    expect(output).toContain('x 2026-08-27 2026-08-27 Completed chore');
  });

  it('formats tasks as JSON', () => {
    const jsonStr = formatAsJson(mockTasks);
    const parsed = JSON.parse(jsonStr);
    expect(parsed.version).toBe('1.0');
    expect(parsed.tasks).toHaveLength(3);
    expect(parsed.tasks[1].priority).toBe(5);
    expect(parsed.tasks[0].link).toBe('https://cloud.digitalocean.com/');
  });

  it('formats tasks as CSV with proper headers and escaping', () => {
    const csvStr = formatAsCsv(mockTasks);
    const lines = csvStr.split('\n');
    expect(lines[0]).toBe('id,text,link,status,context,priority,created_at,completed_at');
    expect(lines[1]).toContain('Cancel DigitalOcean');
    expect(lines[1]).toContain('https://cloud.digitalocean.com/');
    expect(lines[2]).toContain('Write essay');
  });
});
