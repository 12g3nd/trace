import { describe, expect, it } from 'vitest';
import {
  cycleSidecarBay,
  isLauncherRunning,
  loadSidecarBay,
  saveSidecarBay,
  SIDECAR_BAY_STORAGE_KEY,
} from './sidecar-state';

describe('Sidecar bay state', () => {
  it('recognizes the open Codex and Claude launcher states', () => {
    expect(isLauncherRunning('codex', { codexRunning: true, claudeRunning: false })).toBe(true);
    expect(isLauncherRunning('claude', { codexRunning: false, claudeRunning: true })).toBe(true);
    expect(isLauncherRunning('localsend', { codexRunning: true, claudeRunning: true })).toBe(false);
  });

  it('cycles forward and backward through all three bays', () => {
    expect(cycleSidecarBay('trace', 1)).toBe('media');
    expect(cycleSidecarBay('media', 1)).toBe('load');
    expect(cycleSidecarBay('load', 1)).toBe('trace');
    expect(cycleSidecarBay('trace', -1)).toBe('load');
  });

  it('keeps cycling deterministically across repeated navigation', () => {
    let forward: 'trace' | 'media' | 'load' = 'trace';
    for (let index = 0; index < 30; index += 1) {
      forward = cycleSidecarBay(forward, 1);
    }
    expect(forward).toBe('trace');

    let backward: 'trace' | 'media' | 'load' = 'trace';
    for (let index = 0; index < 30; index += 1) {
      backward = cycleSidecarBay(backward, -1);
    }
    expect(backward).toBe('trace');
  });

  it('persists valid selections and falls back safely for stale values', () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    };

    saveSidecarBay(storage, 'load');
    expect(values.get(SIDECAR_BAY_STORAGE_KEY)).toBe('load');
    expect(loadSidecarBay(storage)).toBe('load');

    values.set(SIDECAR_BAY_STORAGE_KEY, 'retired-bay');
    expect(loadSidecarBay(storage)).toBe('trace');
  });
});
