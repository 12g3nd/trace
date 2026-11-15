<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import chatgptIcon from '../../assets/orbit/chatgpt.svg';
  import claudeIcon from '../../assets/orbit/claude.svg';
  import geminiIcon from '../../assets/orbit/gemini.png';
  import {
    SIDECAR_BAY_STORAGE_KEY,
    cycleSidecarBay,
    isLauncherRunning,
    loadSidecarBay,
    saveSidecarBay,
    type SidecarLauncher,
    type SidecarLauncherState,
    type SidecarBay,
  } from '$lib/sidecar-state';

  type Launcher = SidecarLauncher;


  let bay: SidecarBay = 'trace';
  let glass = false;
  /** The wallpaper scene's accent (#rrggbb), or '' when scene accents are off in Orbit. */
  let sceneAccent = '';
  let appliedMaterial: boolean | null = null;
  let unavailableLaunchers: Launcher[] = [];
  let launching: Launcher[] = [];
  let launcherRefreshInFlight = false;
  let launcherState: SidecarLauncherState = { codexRunning: false, claudeRunning: false, geminiRunning: false, localsendRunning: false };
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let tick = 0;
  let disposed = false;

  onMount(() => {
    document.documentElement.classList.add('shell-document');
    try {
      bay = loadSidecarBay(localStorage);
    } catch {
      bay = 'trace';
    }

    function syncBay(event: StorageEvent) {
      if (event.key === SIDECAR_BAY_STORAGE_KEY || event.key === null) {
        try { bay = loadSidecarBay(localStorage); } catch { bay = 'trace'; }
      }
    }
    window.addEventListener('storage', syncBay);
    void refreshLauncherState();
    void refreshMaterial();

    pollTimer = setInterval(() => {
      tick += 1;
      void refreshLauncherState();
      if (tick % 3 === 0) void refreshMaterial();
      if (tick % 15 === 0) void refreshAnchor();
    }, 1000);

    return () => {
      disposed = true;
      window.removeEventListener('storage', syncBay);
      document.documentElement.classList.remove('shell-document');
      if (pollTimer) clearInterval(pollTimer);
    };
  });

  async function refreshMaterial() {
    try {
      const enabled = await invoke<boolean>('orbit_glass', { previous: appliedMaterial });
      if (!disposed) appliedMaterial = glass = enabled;
    } catch {
      if (!disposed) glass = false;
    }
    try {
      const accent = await invoke<string | null>('orbit_scene_accent');
      if (!disposed) sceneAccent = accent ?? '';
    } catch {
      if (!disposed) sceneAccent = '';
    }
  }

  async function refreshAnchor() {
    try {
      await invoke('reanchor_sidecar');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not refresh monitor anchor:', error);
    }
  }

  function launcherEnabled(launcher: Launcher): boolean {
    return !unavailableLaunchers.includes(launcher);
  }

  function launcherTitle(launcher: Launcher, name: string): string {
    if (!launcherEnabled(launcher)) return `${name} is unavailable`;
    return isLauncherRunning(launcher, launcherState) ? `${name} is open` : name;
  }

  async function refreshLauncherState() {
    if (launcherRefreshInFlight || disposed) return;
    launcherRefreshInFlight = true;
    try {
      launcherState = await invoke<SidecarLauncherState>('get_launcher_state');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not read launcher state:', error);
      launcherState = { codexRunning: false, claudeRunning: false, geminiRunning: false, localsendRunning: false };
    } finally {
      launcherRefreshInFlight = false;
    }
  }

  async function openLauncher(launcher: Launcher) {
    if (!launcherEnabled(launcher) || launching.includes(launcher)) return;
    launching = [...launching, launcher];
    try {
      await invoke('launch_app', { app: launcher });
      setTimeout(refreshLauncherState, 500);
    } catch (error) {
      console.warn(`[Trace Sidecar] Could not launch ${launcher}:`, error);
    } finally {
      launching = launching.filter((pending) => pending !== launcher);
    }
  }

  async function openTrace() {
    try {
      await invoke('show_trace');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not show Trace:', error);
    }
  }

  let panelError = '';

  async function openOrbitPanel(panel: 'instruments' | 'scene') {
    panelError = '';
    try {
      await invoke('open_orbit_panel', { panel });
    } catch (error) {
      panelError = String(error);
    }
  }
  async function openContextMenu() {
    try {
      await invoke('show_sidecar_menu');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not show context menu:', error);
    }
  }

  function selectBay(direction: -1 | 1) {
    bay = cycleSidecarBay(bay, direction);
    try {
      saveSidecarBay(localStorage, bay);
    } catch {
      // A disabled webview storage preference should not affect the Sidecar.
    }
  }

  function handleWheel(event: WheelEvent) {
    if (Math.abs(event.deltaY) < 1) return;
    selectBay(event.deltaY > 0 ? 1 : -1);
  }

</script>

<main
  class="sidecar-shell"
  class:glass
  class:accented={sceneAccent !== ''}
  style:--scene-accent={sceneAccent || null}
  aria-label="Orbit Sidecar"
  on:contextmenu|preventDefault={openContextMenu}
>
  <nav class="launcher-group" aria-label="Application launchers">
    <button
      class="launcher-btn"
      class:running={isLauncherRunning('codex', launcherState)}
      disabled={!launcherEnabled('codex')}
      title={launcherTitle('codex', 'Codex')}
      aria-label={launcherTitle('codex', 'Codex')}
      on:click={() => openLauncher('codex')}
    >
      <img class="launcher-icon chatgpt-icon" src={chatgptIcon} alt="" />
    </button>

    <button
      class="launcher-btn"
      class:running={isLauncherRunning('claude', launcherState)}
      disabled={!launcherEnabled('claude')}
      title={launcherTitle('claude', 'Claude')}
      aria-label={launcherTitle('claude', 'Claude')}
      on:click={() => openLauncher('claude')}
    >
      <img class="launcher-icon claude-icon" src={claudeIcon} alt="" />
    </button>

    <button
      class="launcher-btn"
      class:running={isLauncherRunning('gemini', launcherState)}
      disabled={!launcherEnabled('gemini')}
      title={launcherTitle('gemini', 'Gemini')}
      aria-label={launcherTitle('gemini', 'Gemini')}
      on:click={() => openLauncher('gemini')}
    >
      <img class="launcher-icon" src={geminiIcon} alt="" />
    </button>
  </nav>

  <div class="divider" aria-hidden="true"></div>

  <section
    class="dynamic-bay"
    aria-label={`${bay} Sidecar bay`}
    aria-live="polite"
    on:wheel|preventDefault={handleWheel}
  >
    <button
      class="nav-btn"
      title="Previous bay"
      aria-label="Previous Sidecar bay"
      on:click={() => selectBay(-1)}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="m10 3.5-4.5 4.5 4.5 4.5" /></svg>
    </button>

    <div class="bay-frame">
      {#key bay}
        <div class="bay-page">
          {#if bay === 'trace'}
            <button class="trace-door" title="Open Trace capture" aria-label="Open Trace capture" on:click={openTrace}>
              <span class="bay-label">TRACE</span>
              <svg class="open-icon" viewBox="0 0 18 18" aria-hidden="true">
                <path d="M9 4v10M4 9h10" />
              </svg>
            </button>
          {:else}
            <button class="trace-door" title={panelError || (bay === 'scene' ? 'Wallpaper and Gallery controls' : 'Open the Lake Ontario Listening Station')} on:click={() => openOrbitPanel(bay === 'scene' ? 'scene' : 'instruments')}>
              <span class="bay-label">{bay === 'scene' ? 'SCENE' : 'INSTRUMENTS'}</span>
            </button>
          {/if}
        </div>
      {/key}
    </div>

    <button
      class="nav-btn"
      title="Next bay"
      aria-label="Next Sidecar bay"
      on:click={() => selectBay(1)}
    >
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="m6 3.5 4.5 4.5L6 12.5" /></svg>
    </button>
  </section>
</main>

<style>
  .sidecar-shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px 4px;
    overflow: hidden;
    color: var(--on-text);
    background: rgba(11, 23, 49, 0.8);
    border: 1px solid rgba(127, 166, 196, 0.17);
    border-radius: 10px;
    user-select: none;
    --mark: var(--on-accent);
  }

  /* Scene accents: the capture mark and the open-app halo follow the wallpaper. */
  .sidecar-shell.accented {
    --mark: var(--scene-accent);
  }

  .launcher-group {
    display: flex;
    align-items: center;
    gap: 1px;
    flex: 0 0 auto;
  }

  button {
    border: 0;
    -webkit-app-region: no-drag;
    pointer-events: auto;
  }

  .launcher-btn {
    position: relative;
    width: 28px;
    height: 38px;
    display: grid;
    place-items: center;
    padding: 0;
    border-radius: 8px;
    transition: background 150ms var(--on-ease), opacity 150ms var(--on-ease);
  }

  .launcher-btn:hover:not(:disabled),
  .launcher-btn:focus-visible {
    background: rgba(232, 239, 245, 0.08);
  }

  .launcher-btn.running {
    background: rgba(42, 148, 226, 0.22);
    box-shadow: 0 0 8px rgba(74, 191, 255, 0.34);
  }

  .sidecar-shell.accented .launcher-btn.running {
    background: color-mix(in srgb, var(--scene-accent) 22%, transparent);
    box-shadow: 0 0 8px color-mix(in srgb, var(--scene-accent) 34%, transparent);
  }

  .launcher-btn.running .launcher-icon {
    transform: scale(1.08);
  }

  .launcher-btn:disabled {
    opacity: 0.3;
  }

  .launcher-icon {
    display: block;
    width: 20px;
    height: 20px;
    object-fit: contain;
    pointer-events: none;
  }

  .chatgpt-icon {
    width: 22px;
    height: 22px;
    filter: invert(94%) sepia(8%) saturate(258%) hue-rotate(166deg) brightness(101%);
  }

  .claude-icon {
    width: 19px;
    height: 19px;
  }

  .divider {
    width: 1px;
    height: 24px;
    margin: 0 1px;
    flex: 0 0 auto;
    background: rgba(232, 239, 245, 0.13);
  }

  .dynamic-bay {
    position: relative;
    isolation: isolate;
    min-width: 0;
    height: 38px;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .nav-btn {
    position: relative;
    z-index: 3;
    width: 26px;
    height: 38px;
    flex: 0 0 26px;
    display: grid;
    place-items: center;
    padding: 0;
    color: rgba(168, 184, 202, 0.52);
    border-radius: 6px;
    transition: color 150ms var(--on-ease), background 150ms var(--on-ease);
  }

  .nav-btn:hover,
  .nav-btn:focus-visible {
    color: var(--on-text-secondary);
    background: rgba(232, 239, 245, 0.05);
  }

  .nav-btn svg {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .bay-frame,
  .bay-page {
    min-width: 0;
    height: 38px;
    flex: 1;
  }

  .bay-frame {
    position: relative;
    z-index: 1;
    overflow: hidden;
    isolation: isolate;
  }

  .bay-page {
    position: relative;
    z-index: 1;
    width: 100%;
    pointer-events: auto;
    animation: bay-in 170ms var(--on-ease);
  }

  .trace-door {
    width: 100%;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 0 4px;
    border-radius: 8px;
    color: var(--on-text-secondary);
    transition: color 150ms var(--on-ease), background 150ms var(--on-ease);
  }

  .trace-door:hover,
  .trace-door:focus-visible {
    color: var(--on-text);
    background: rgba(16, 42, 76, 0.62);
  }

  .sidecar-shell.glass {
    background: rgba(16, 29, 54, 0.52);
    border-radius: 8px;
  }

  .bay-label {
    font-family: var(--on-font-interface);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.035em;
  }

  .open-icon {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: var(--mark);
    stroke-width: 1.5;
    stroke-linecap: round;
  }

  @keyframes bay-in {
    from {
      opacity: 0;
      transform: translateX(2px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .bay-page {
      animation: none;
    }

    .launcher-btn,
    .nav-btn,
    .trace-door {
      transition: none;
    }
  }
</style>
