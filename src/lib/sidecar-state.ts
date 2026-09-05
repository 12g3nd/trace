export const SIDECAR_BAYS = ['trace', 'media', 'load'] as const;
export type SidecarBay = (typeof SIDECAR_BAYS)[number];
export const SIDECAR_LAUNCHERS = ['localsend', 'codex', 'claude'] as const;
export type SidecarLauncher = (typeof SIDECAR_LAUNCHERS)[number];

export interface SidecarLauncherState {
  codexRunning: boolean;
  claudeRunning: boolean;
}

export const SIDECAR_BAY_STORAGE_KEY = 'trace.sidecar.bay';

interface BayStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function parseSidecarBay(value: string | null): SidecarBay {
  return SIDECAR_BAYS.includes(value as SidecarBay) ? (value as SidecarBay) : 'trace';
}

export function loadSidecarBay(storage: BayStorage): SidecarBay {
  return parseSidecarBay(storage.getItem(SIDECAR_BAY_STORAGE_KEY));
}

export function saveSidecarBay(storage: BayStorage, bay: SidecarBay): void {
  storage.setItem(SIDECAR_BAY_STORAGE_KEY, bay);
}

export function isLauncherRunning(launcher: SidecarLauncher, state: SidecarLauncherState): boolean {
  return (launcher === 'codex' && state.codexRunning)
    || (launcher === 'claude' && state.claudeRunning);
}

export function cycleSidecarBay(current: SidecarBay, direction: -1 | 1): SidecarBay {
  const currentIndex = SIDECAR_BAYS.indexOf(current);
  return SIDECAR_BAYS[(currentIndex + direction + SIDECAR_BAYS.length) % SIDECAR_BAYS.length];
}
