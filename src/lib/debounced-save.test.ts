import { afterEach, describe, expect, it, vi } from 'vitest';
import { createDebouncedSaver } from './debounced-save';
import { flushPendingTaskEdits, registerPendingTaskEdit } from './pending-edits';

afterEach(() => {
  vi.useRealTimers();
});

describe('debounced task edit saves', () => {
  it('coalesces keystrokes and saves only the latest draft after 650ms', async () => {
    vi.useFakeTimers();
    const saved: string[] = [];
    const saver = createDebouncedSaver<string>(650, async (value) => {
      saved.push(value);
    });

    saver.schedule('fir');
    saver.schedule('first');
    await vi.advanceTimersByTimeAsync(649);
    expect(saved).toEqual([]);

    await vi.advanceTimersByTimeAsync(1);
    expect(saved).toEqual(['first']);
  });

  it('flushes a pending draft immediately for window lifecycle events', async () => {
    vi.useFakeTimers();
    const saved: string[] = [];
    const saver = createDebouncedSaver<string>(650, async (value) => {
      saved.push(value);
    });
    const unregister = registerPendingTaskEdit(saver.flush);

    saver.schedule('hide-safe draft');
    await flushPendingTaskEdits();

    expect(saved).toEqual(['hide-safe draft']);
    unregister();
  });

  it('persists a newer draft queued while a prior save is in flight', async () => {
    let releaseFirstSave: (() => void) | undefined;
    let markFirstSaveStarted: (() => void) | undefined;
    const firstSaveBlocked = new Promise<void>((resolve) => {
      releaseFirstSave = resolve;
    });
    const firstSaveStarted = new Promise<void>((resolve) => {
      markFirstSaveStarted = resolve;
    });
    const saved: string[] = [];
    const saver = createDebouncedSaver<string>(650, async (value) => {
      saved.push(value);
      if (value === 'first') {
        markFirstSaveStarted?.();
        await firstSaveBlocked;
      }
    });

    saver.schedule('first');
    const flush = saver.flush();
    await firstSaveStarted;
    saver.schedule('second');
    releaseFirstSave?.();
    await flush;

    expect(saved).toEqual(['first', 'second']);
  });
});
