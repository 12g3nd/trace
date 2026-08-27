import { describe, it, expect } from 'vitest';
import { parseNaturalDate, formatRelativeDue } from './date-parser';

describe('Natural Language Date Parser', () => {
  const fixedNow = new Date('2026-08-27T12:00:00.000Z'); // Thursday

  it('parses "tomorrow" and extracts due date', () => {
    const result = parseNaturalDate('Email Edwin tomorrow ~ FI99 **', fixedNow);
    expect(result.cleanedText).toBe('Email Edwin ~ FI99 **');
    expect(result.dueAt).not.toBeNull();
    const parsedDate = new Date(result.dueAt!);
    expect(parsedDate.getDate()).toBe(fixedNow.getDate() + 1);
  });

  it('parses "today" correctly', () => {
    const result = parseNaturalDate('Finish slides today', fixedNow);
    expect(result.cleanedText).toBe('Finish slides');
    expect(result.dueAt).not.toBeNull();
    const parsedDate = new Date(result.dueAt!);
    expect(parsedDate.getDate()).toBe(fixedNow.getDate());
  });

  it('parses "in 3 days" correctly', () => {
    const result = parseNaturalDate('Submit PR in 3 days', fixedNow);
    expect(result.cleanedText).toBe('Submit PR');
    expect(result.dueAt).not.toBeNull();
    const parsedDate = new Date(result.dueAt!);
    expect(parsedDate.getDate()).toBe(fixedNow.getDate() + 3);
  });

  it('parses explicit due date due:2026-09-01', () => {
    const result = parseNaturalDate('Pay invoice due:2026-09-01', fixedNow);
    expect(result.cleanedText).toBe('Pay invoice');
    expect(result.dueAt).toContain('2026-09-01');
  });

  it('returns null dueAt when no date keywords are present', () => {
    const result = parseNaturalDate('Review codebase architecture', fixedNow);
    expect(result.cleanedText).toBe('Review codebase architecture');
    expect(result.dueAt).toBeNull();
  });
});

describe('Relative Due Badge Formatter', () => {
  const fixedNow = new Date('2026-08-27T12:00:00.000Z');

  it('formats today badge', () => {
    const today = new Date('2026-08-27T23:59:59.999Z').toISOString();
    const badge = formatRelativeDue(today, fixedNow);
    expect(badge?.label).toBe('today');
    expect(badge?.isOverdue).toBe(false);
  });

  it('formats tomorrow badge', () => {
    const tmrw = new Date('2026-08-28T23:59:59.999Z').toISOString();
    const badge = formatRelativeDue(tmrw, fixedNow);
    expect(badge?.label).toBe('tomorrow');
    expect(badge?.isOverdue).toBe(false);
  });

  it('formats overdue badge', () => {
    const past = new Date('2026-08-20T12:00:00.000Z').toISOString();
    const badge = formatRelativeDue(past, fixedNow);
    expect(badge?.label).toBe('overdue');
    expect(badge?.isOverdue).toBe(true);
  });
});
