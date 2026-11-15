import { describe, expect, it } from 'vitest';
import {
  cycleSidecarBay,
  isLauncherRunning,
  loadSidecarBay,
  saveSidecarBay,
  SIDECAR_BAY_STORAGE_KEY,
  type SidecarBay,
} from './sidecar-state';

describe('Sidecar bay state', () => {
  it('recognizes the open Codex and Claude launcher states', () => {
    const state = { codexRunning: true, claudeRunning: true, geminiRunning: false, localsendRunning: false };
    expect(isLauncherRunning('codex', state)).toBe(true);
    expect(isLauncherRunning('claude', state)).toBe(true);
    expect(isLauncherRunning('gemini', state)).toBe(false);
    expect(isLauncherRunning('gemini', { ...state, geminiRunning: true })).toBe(true);
    expect(isLauncherRunning('localsend', state)).toBe(false);
    expect(isLauncherRunning('localsend', { ...state, localsendRunning: true })).toBe(true);
  });

  it('cycles through Trace, Instruments, and Scene in both directions', () => {
    expect(cycleSidecarBay('trace', 1)).toBe('instruments');
    expect(cycleSidecarBay('instruments', 1)).toBe('scene');
    expect(cycleSidecarBay('scene', 1)).toBe('trace');
    expect(cycleSidecarBay('trace', -1)).toBe('scene');
    expect(cycleSidecarBay('scene', -1)).toBe('instruments');
    expect(cycleSidecarBay('instruments', -1)).toBe('trace');
  });

  it('keeps cycling deterministically across repeated navigation', () => {
    let forward: SidecarBay = 'trace';
    for (let index = 0; index < 42; index += 1) {
      forward = cycleSidecarBay(forward, 1);
    }
    expect(forward).toBe('trace');

    let backward: SidecarBay = 'trace';
    for (let index = 0; index < 42; index += 1) {
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

    saveSidecarBay(storage, 'scene');
    expect(values.get(SIDECAR_BAY_STORAGE_KEY)).toBe('scene');
    expect(loadSidecarBay(storage)).toBe('scene');

    saveSidecarBay(storage, 'instruments');
    expect(values.get(SIDECAR_BAY_STORAGE_KEY)).toBe('instruments');
    expect(loadSidecarBay(storage)).toBe('instruments');

    for (const retired of ['media', 'load', 'localsend', 'atlas']) {
      values.set(SIDECAR_BAY_STORAGE_KEY, retired);
      expect(loadSidecarBay(storage)).toBe('trace');
    }
    values.set(SIDECAR_BAY_STORAGE_KEY, 'retired-bay');
    expect(loadSidecarBay(storage)).toBe('trace');
  });
});
