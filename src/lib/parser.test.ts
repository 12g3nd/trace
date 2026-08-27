import { describe, it, expect } from 'vitest';
import { parseInput } from './parser';

describe('Task Input Parser', () => {
  it('parses plain task text with no context or priority', () => {
    const result = parseInput('Cancel DigitalOcean');
    expect(result).toEqual({
      text: 'Cancel DigitalOcean',
      context: null,
      priority: 0,
      due_at: null,
    });
  });

  it('parses task with context', () => {
    const result = parseInput('Fallow algo ~ chatgpt');
    expect(result).toEqual({
      text: 'Fallow algo',
      context: 'chatgpt',
      priority: 0,
      due_at: null,
    });
  });

  it('parses task with context and priority', () => {
    const result = parseInput('Write "What\'s a God to a Non-believer" blog ~ chatgpt *****');
    expect(result).toEqual({
      text: 'Write "What\'s a God to a Non-believer" blog',
      context: 'chatgpt',
      priority: 5,
      due_at: null,
    });
  });

  it('parses task with only priority', () => {
    const result = parseInput('Respond to Hamza dad *****');
    expect(result).toEqual({
      text: 'Respond to Hamza dad',
      context: null,
      priority: 5,
      due_at: null,
    });
  });

  it('parses task with 2-star priority, context, and natural date', () => {
    const result = parseInput('Email Edwin tomorrow ~ FI99 **');
    expect(result.text).toBe('Email Edwin');
    expect(result.context).toBe('fi99');
    expect(result.priority).toBe(2);
    expect(result.due_at).not.toBeNull();
  });

  it('caps priority at 5 even if more asterisks are provided', () => {
    const result = parseInput('Super urgent task ~ critical *******');
    expect(result).toEqual({
      text: 'Super urgent task',
      context: 'critical',
      priority: 5,
      due_at: null,
    });
  });

  it('handles empty or whitespace-only input safely', () => {
    expect(parseInput('')).toEqual({
      text: '',
      context: null,
      priority: 0,
      due_at: null,
    });
    expect(parseInput('   ')).toEqual({
      text: '',
      context: null,
      priority: 0,
      due_at: null,
    });
  });

  it('handles multiple tildes gracefully by splitting on the last one', () => {
    const result = parseInput('Review ~ chapter 1 ~ book');
    expect(result).toEqual({
      text: 'Review ~ chapter 1',
      context: 'book',
      priority: 0,
      due_at: null,
    });
  });

  it('handles tilde followed only by stars', () => {
    const result = parseInput('Important item ~ ***');
    expect(result).toEqual({
      text: 'Important item',
      context: null,
      priority: 3,
      due_at: null,
    });
  });

  it('preserves casing of main text while normalizing context to lowercase', () => {
    const result = parseInput('Fix Lacquer Clipping ~ LacquerApp');
    expect(result).toEqual({
      text: 'Fix Lacquer Clipping',
      context: 'lacquerapp',
      priority: 0,
      due_at: null,
    });
  });
});
