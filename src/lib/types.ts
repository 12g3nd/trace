export type Status = 'now' | 'later' | 'someday' | 'done';

export interface Task {
  id: string;
  text: string;
  raw_input: string | null;
  link: string | null;
  status: Status;
  context: string | null;
  priority: number;
  due_at: string | null;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  sort_order: number;
}

export interface ParsedInput {
  text: string;
  context: string | null;
  priority: number;
  due_at?: string | null;
}

/** Status labels in display order. */
export const STATUS_ORDER: Status[] = ['now', 'later', 'someday', 'done'];

export const STATUS_LABELS: Record<Status, string> = {
  now: 'Now',
  later: 'Later',
  someday: 'Someday',
  done: 'Done',
};
