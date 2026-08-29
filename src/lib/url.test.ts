import { describe, expect, it } from 'vitest';
import { normalizeTaskLink } from './url';

describe('normalizeTaskLink', () => {
  it('accepts http and https URLs without rewriting them', () => {
    expect(normalizeTaskLink('https://example.com/path?q=1')).toEqual({
      value: 'https://example.com/path?q=1',
      error: null,
    });
    expect(normalizeTaskLink('http://localhost:3000/task')).toEqual({
      value: 'http://localhost:3000/task',
      error: null,
    });
  });

  it('normalizes a bare domain to https', () => {
    expect(normalizeTaskLink('example.com/notes')).toEqual({
      value: 'https://example.com/notes',
      error: null,
    });
  });

  it('treats blank input as link removal and rejects invalid input', () => {
    expect(normalizeTaskLink('   ')).toEqual({ value: null, error: null });
    expect(normalizeTaskLink('not a url')).toEqual({
      value: null,
      error: 'Use an http:// or https:// URL.',
    });
    expect(normalizeTaskLink('ftp://example.com')).toEqual({
      value: null,
      error: 'Use an http:// or https:// URL.',
    });
  });
});
