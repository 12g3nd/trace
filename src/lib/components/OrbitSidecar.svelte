<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import chatgptIcon from '../../assets/orbit/chatgpt.svg';
  import claudeIcon from '../../assets/orbit/claude.svg';
  import localSendIcon from '../../assets/orbit/localsend.svg';
  import {
    cycleSidecarBay,
    loadSidecarBay,
    saveSidecarBay,
    type SidecarBay,
  } from '$lib/sidecar-state';

  type Launcher = 'localsend' | 'chatgpt' | 'claude';

  interface MediaState {
    available: boolean;
    title: string;
    artist: string;
    source: string;
    artworkKey: string | null;
    artwork: string | null;
    playing: boolean;
    canToggle: boolean;
    canNext: boolean;
  }

  interface LoadState {
    memoryUsedGib: number;
    memoryTotalGib: number;
    cpuPercent: number;
  }

  const EMPTY_MEDIA: MediaState = {
    available: false,
    title: '',
    artist: '',
    source: '',
    artworkKey: null,
    artwork: null,
    playing: false,
    canToggle: false,
    canNext: false,
  };

  let bay: SidecarBay = 'trace';
  let unavailableLaunchers: Launcher[] = [];
  let localSendRunning = false;
  let media = EMPTY_MEDIA;
  let load: LoadState | null = null;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let tick = 0;
  let mediaRefreshInFlight = false;
  let loadRefreshInFlight = false;
  let localSendRefreshInFlight = false;

  $: mediaLabel = media.available
    ? media.title.trim() || media.artist.trim() || 'UNTITLED'
    : 'NO MEDIA';
  $: mediaTooltip = [media.title, media.artist].filter(Boolean).join(' — ') || 'No active media session';
  $: loadTooltip = load
    ? `Memory ${load.memoryUsedGib.toFixed(1)} of ${load.memoryTotalGib.toFixed(1)} GiB; CPU ${Math.round(load.cpuPercent)} percent`
    : 'System load is initializing';

  onMount(() => {
    document.documentElement.classList.add('sidecar-document');
    try {
      bay = loadSidecarBay(localStorage);
    } catch {
      bay = 'trace';
    }

    void refreshLocalSendStatus();
    if (bay === 'media') void refreshMedia();
    if (bay === 'load') void refreshLoad();

    pollTimer = setInterval(() => {
      tick += 1;
      if (bay === 'media') void refreshMedia();
      if (bay === 'load' && tick % 2 === 0) void refreshLoad();
      if (tick % 5 === 0) void refreshLocalSendStatus();
      if (tick % 15 === 0) void refreshAnchor();
    }, 1000);

    return () => {
      document.documentElement.classList.remove('sidecar-document');
      if (pollTimer) clearInterval(pollTimer);
    };
  });

  async function refreshLocalSendStatus() {
    if (localSendRefreshInFlight) return;
    localSendRefreshInFlight = true;
    try {
      localSendRunning = await invoke<boolean>('is_localsend_running');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not read LocalSend status:', error);
      localSendRunning = false;
    } finally {
      localSendRefreshInFlight = false;
    }
  }

  async function refreshMedia() {
    if (mediaRefreshInFlight) return;
    mediaRefreshInFlight = true;
    try {
      const next = await invoke<MediaState>('get_media_state', { artworkKey: media.artworkKey });
      if (next.artworkKey === media.artworkKey && !next.artwork && media.artwork) {
        next.artwork = media.artwork;
      }
      media = next;
    } catch (error) {
      console.warn('[Trace Sidecar] Could not read Windows media state:', error);
      media = EMPTY_MEDIA;
    } finally {
      mediaRefreshInFlight = false;
    }
  }

  async function refreshLoad() {
    if (loadRefreshInFlight || bay !== 'load') return;
    loadRefreshInFlight = true;
    try {
      load = await invoke<LoadState>('get_load_state');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not read system load:', error);
      load = null;
    } finally {
      loadRefreshInFlight = false;
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
    return launcherEnabled(launcher) ? name : `${name} is unavailable`;
  }

  async function openLauncher(launcher: Launcher) {
    if (!launcherEnabled(launcher)) return;
    try {
      await invoke('launch_app', { app: launcher });
      if (launcher === 'localsend') setTimeout(refreshLocalSendStatus, 800);
    } catch (error) {
      console.warn(`[Trace Sidecar] Could not launch ${launcher}:`, error);
      unavailableLaunchers = [...new Set([...unavailableLaunchers, launcher])];
    }
  }

  async function openTrace() {
    try {
      await invoke('show_trace');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not show Trace:', error);
    }
  }

  async function openTaskManager() {
    try {
      await invoke('open_task_manager');
    } catch (error) {
      console.warn('[Trace Sidecar] Could not open Task Manager:', error);
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
    if (bay === 'media') void refreshMedia();
    if (bay === 'load') void refreshLoad();
  }

  function handleWheel(event: WheelEvent) {
    if (Math.abs(event.deltaY) < 1) return;
    selectBay(event.deltaY > 0 ? 1 : -1);
  }

  async function runMediaCommand(action: 'toggle' | 'next') {
    try {
      await invoke('media_command', { action });
      setTimeout(refreshMedia, 120);
    } catch (error) {
      console.warn(`[Trace Sidecar] Media command ${action} failed:`, error);
    }
  }
</script>

<main
  class="sidecar-shell"
  aria-label="Orbit Sidecar"
  on:contextmenu|preventDefault={openContextMenu}
>
  <nav class="launcher-group" aria-label="Application launchers">
    <button
      class="launcher-btn"
      disabled={!launcherEnabled('localsend')}
      title={launcherTitle('localsend', 'LocalSend')}
      aria-label="LocalSend"
      on:click={() => openLauncher('localsend')}
    >
      <img class="launcher-icon localsend-icon" src={localSendIcon} alt="" />
      <span class:running={localSendRunning} class="status-dot" aria-hidden="true"></span>
    </button>

    <button
      class="launcher-btn"
      disabled={!launcherEnabled('chatgpt')}
      title={launcherTitle('chatgpt', 'ChatGPT')}
      aria-label="ChatGPT"
      on:click={() => openLauncher('chatgpt')}
    >
      <img class="launcher-icon chatgpt-icon" src={chatgptIcon} alt="" />
    </button>

    <button
      class="launcher-btn"
      disabled={!launcherEnabled('claude')}
      title={launcherTitle('claude', 'Claude')}
      aria-label="Claude"
      on:click={() => openLauncher('claude')}
    >
      <img class="launcher-icon claude-icon" src={claudeIcon} alt="" />
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
          {:else if bay === 'media'}
            <div class="media-bay" title={mediaTooltip}>
              {#if media.artwork}
                <img class="artwork" src={media.artwork} alt="" />
              {:else}
                <div class="artwork artwork-fallback" aria-hidden="true">
                  <svg viewBox="0 0 20 20"><path d="M8 14.2V5.4l7-1.5v8.5M8 14.2c0 1-1.1 1.8-2.4 1.8s-2.4-.8-2.4-1.8 1.1-1.8 2.4-1.8 2.4.8 2.4 1.8Zm7-1.8c0 1-1.1 1.8-2.4 1.8s-2.4-.8-2.4-1.8 1.1-1.8 2.4-1.8 2.4.8 2.4 1.8Z" /></svg>
                </div>
              {/if}
              <span class:idle={!media.available} class="media-title">{mediaLabel}</span>
              <button
                class="media-btn"
                disabled={!media.available || !media.canToggle}
                title={media.playing ? 'Pause' : 'Play'}
                aria-label={media.playing ? 'Pause current media' : 'Play current media'}
                on:click={() => runMediaCommand('toggle')}
              >
                {#if media.playing}
                  <svg viewBox="0 0 18 18" aria-hidden="true"><path d="M6 5v8M12 5v8" /></svg>
                {:else}
                  <svg viewBox="0 0 18 18" aria-hidden="true"><path class="fill" d="m6.5 4.8 7 4.2-7 4.2z" /></svg>
                {/if}
              </button>
              <button
                class="media-btn"
                disabled={!media.available || !media.canNext}
                title="Next"
                aria-label="Next media track"
                on:click={() => runMediaCommand('next')}
              >
                <svg viewBox="0 0 18 18" aria-hidden="true">
                  <path class="fill" d="m5 5 6 4-6 4z" />
                  <path d="M12.5 5v8" />
                </svg>
              </button>
            </div>
          {:else}
            <button class="load-bay" title={loadTooltip} aria-label={`${loadTooltip}; open Task Manager`} on:click={openTaskManager}>
              {#if load}
                <span><strong>MEM</strong> {load.memoryUsedGib.toFixed(1)}/{load.memoryTotalGib.toFixed(1)}G</span>
                <span><strong>CPU</strong> {Math.round(load.cpuPercent)}%</span>
              {:else}
                <span class="bay-label">LOAD</span>
                <span class="load-pending">INITIALIZING</span>
              {/if}
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
    background: rgba(11, 23, 49, 0.92);
    border: 1px solid rgba(232, 239, 245, 0.12);
    border-radius: 12px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.16);
    user-select: none;
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

  .status-dot {
    position: absolute;
    right: 3px;
    bottom: 4px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--on-text-quiet);
    opacity: 0.5;
  }

  .status-dot.running {
    background: var(--on-accent-secondary);
    opacity: 1;
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

  .trace-door,
  .load-bay {
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
  .trace-door:focus-visible,
  .load-bay:hover,
  .load-bay:focus-visible {
    color: var(--on-text);
    background: rgba(16, 42, 76, 0.62);
  }

  .bay-label {
    font-family: var(--on-font-graphic);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.11em;
  }

  .open-icon {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: var(--on-accent);
    stroke-width: 1.5;
    stroke-linecap: round;
  }

  .media-bay {
    width: 100%;
    height: 38px;
    display: flex;
    align-items: center;
    gap: 3px;
    min-width: 0;
  }

  .artwork {
    width: 24px;
    height: 24px;
    flex: 0 0 24px;
    border-radius: 5px;
    object-fit: cover;
    background: var(--on-surface-raised);
  }

  .artwork-fallback {
    display: grid;
    place-items: center;
    color: var(--on-text-quiet);
  }

  .artwork-fallback svg {
    width: 15px;
    height: 15px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.25;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .media-title {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--on-font-interface);
    font-size: 10px;
    color: var(--on-text-secondary);
  }

  .media-title.idle {
    font-family: var(--on-font-mono);
    font-size: 9px;
    letter-spacing: 0.04em;
    color: var(--on-text-quiet);
  }

  .media-btn {
    width: 20px;
    height: 30px;
    flex: 0 0 20px;
    display: grid;
    place-items: center;
    padding: 0;
    border-radius: 6px;
    color: var(--on-text-secondary);
    transition: color 150ms var(--on-ease), background 150ms var(--on-ease), opacity 150ms var(--on-ease);
  }

  .media-btn:hover:not(:disabled),
  .media-btn:focus-visible {
    color: var(--on-text);
    background: rgba(232, 239, 245, 0.07);
  }

  .media-btn:disabled {
    opacity: 0.25;
  }

  .media-btn svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .media-btn .fill {
    fill: currentColor;
    stroke: none;
  }

  .load-bay {
    justify-content: space-between;
    gap: 5px;
    padding: 0 2px;
    font-family: var(--on-font-mono);
    font-size: 8px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .load-bay strong {
    color: var(--on-accent-secondary);
    font-size: 8px;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  .load-pending {
    color: var(--on-text-quiet);
    font-size: 8px;
    letter-spacing: 0.05em;
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
    .trace-door,
    .load-bay,
    .media-btn {
      transition: none;
    }
  }
</style>
