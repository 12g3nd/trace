<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  type Bay = 'trace' | 'media';
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

  let bay: Bay = 'trace';
  let unavailableLaunchers: Launcher[] = [];
  let localSendRunning = false;
  let media = EMPTY_MEDIA;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let tick = 0;
  let mediaRefreshInFlight = false;
  let localSendRefreshInFlight = false;

  $: mediaLabel = media.available
    ? media.title.trim() || media.artist.trim() || 'UNTITLED'
    : 'NO MEDIA';
  $: mediaTooltip = [media.title, media.artist].filter(Boolean).join(' — ') || 'No active media session';

  onMount(() => {
    document.documentElement.classList.add('sidecar-document');
    bay = localStorage.getItem('trace.sidecar.bay') === 'media' ? 'media' : 'trace';

    refreshLocalSendStatus();
    if (bay === 'media') refreshMedia();

    pollTimer = setInterval(() => {
      tick += 1;
      if (bay === 'media') refreshMedia();
      if (tick % 5 === 0) refreshLocalSendStatus();
      if (tick % 30 === 0) refreshAnchor();
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
      if (launcher === 'localsend') {
        setTimeout(refreshLocalSendStatus, 800);
      }
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

  function switchBay() {
    bay = bay === 'trace' ? 'media' : 'trace';
    localStorage.setItem('trace.sidecar.bay', bay);
    if (bay === 'media') refreshMedia();
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

<main class="sidecar-shell" aria-label="Orbit Sidecar">
  <nav class="launcher-group" aria-label="Application launchers">
    <button
      class="launcher-btn localsend"
      disabled={!launcherEnabled('localsend')}
      title={launcherTitle('localsend', 'LocalSend')}
      aria-label="LocalSend"
      on:click={() => openLauncher('localsend')}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M7.1 7.4a6.9 6.9 0 0 1 10.7 1.4M16.9 16.6A6.9 6.9 0 0 1 6.2 15.2" />
        <path d="m16.2 5.6 1.9 3.4-3.8.2M7.8 18.4 5.9 15l3.8-.2" />
        <circle cx="12" cy="12" r="1.7" />
      </svg>
      <span
        class="status-dot"
        class:running={localSendRunning}
        aria-hidden="true"
      ></span>
    </button>

    <button
      class="launcher-btn chatgpt"
      disabled={!launcherEnabled('chatgpt')}
      title={launcherTitle('chatgpt', 'ChatGPT')}
      aria-label="ChatGPT"
      on:click={() => openLauncher('chatgpt')}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <g class="knot">
          <ellipse cx="12" cy="7" rx="4.4" ry="2.5" />
          <ellipse cx="12" cy="7" rx="4.4" ry="2.5" transform="rotate(60 12 12)" />
          <ellipse cx="12" cy="7" rx="4.4" ry="2.5" transform="rotate(120 12 12)" />
          <ellipse cx="12" cy="7" rx="4.4" ry="2.5" transform="rotate(180 12 12)" />
          <ellipse cx="12" cy="7" rx="4.4" ry="2.5" transform="rotate(240 12 12)" />
          <ellipse cx="12" cy="7" rx="4.4" ry="2.5" transform="rotate(300 12 12)" />
        </g>
      </svg>
    </button>

    <button
      class="launcher-btn claude"
      disabled={!launcherEnabled('claude')}
      title={launcherTitle('claude', 'Claude')}
      aria-label="Claude"
      on:click={() => openLauncher('claude')}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 3v18M3 12h18M5.6 5.6l12.8 12.8M18.4 5.6 5.6 18.4" />
        <path d="m8.1 3.7 7.8 16.6M3.7 15.9l16.6-7.8M15.9 3.7 8.1 20.3M3.7 8.1l16.6 7.8" />
      </svg>
    </button>
  </nav>

  <div class="divider" aria-hidden="true"></div>

  <section class="dynamic-bay" aria-live="polite">
    {#if bay === 'trace'}
      <button class="trace-door" title="Open Trace capture" aria-label="Open Trace capture" on:click={openTrace}>
        <span class="bay-label">TRACE</span>
        <svg class="open-icon" viewBox="0 0 20 20" aria-hidden="true">
          <path d="M10 4v12M4 10h12" />
        </svg>
      </button>
    {:else}
      <div class="media-bay" title={mediaTooltip}>
        {#if media.artwork}
          <img class="artwork" src={media.artwork} alt="" />
        {:else}
          <div class="artwork artwork-fallback" aria-hidden="true">♪</div>
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
            <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M6.5 5v10M13.5 5v10" /></svg>
          {:else}
            <svg viewBox="0 0 20 20" aria-hidden="true"><path class="fill" d="m7 4.8 8 5.2-8 5.2z" /></svg>
          {/if}
        </button>
        <button
          class="media-btn"
          disabled={!media.available || !media.canNext}
          title="Next"
          aria-label="Next media track"
          on:click={() => runMediaCommand('next')}
        >
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path class="fill" d="m5.5 5 6.5 5-6.5 5z" />
            <path d="M13.5 5v10" />
          </svg>
        </button>
      </div>
    {/if}
  </section>

  <button
    class="switch-btn"
    title={bay === 'trace' ? 'Switch to Media' : 'Switch to Trace'}
    aria-label={bay === 'trace' ? 'Switch to Media controls' : 'Switch to Trace'}
    on:click={switchBay}
  >
    <svg viewBox="0 0 20 20" aria-hidden="true"><path d="m7.5 5 5 5-5 5" /></svg>
  </button>
</main>

<style>
  .sidecar-shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    gap: var(--on-space-1);
    padding: 5px 6px;
    overflow: hidden;
    color: var(--on-text);
    background: var(--on-bg);
    border: 1px solid var(--on-hairline-strong);
    border-radius: var(--on-radius-lg);
    box-shadow: 0 6px 18px var(--on-sidecar-shadow);
  }

  .launcher-group {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .launcher-btn,
  .media-btn,
  .switch-btn,
  .trace-door {
    transition:
      color var(--on-duration-fast) var(--on-ease),
      background var(--on-duration-fast) var(--on-ease),
      opacity var(--on-duration-fast) var(--on-ease);
  }

  .launcher-btn {
    position: relative;
    display: grid;
    place-items: center;
    width: 34px;
    height: 40px;
    color: var(--on-text-secondary);
    border-radius: var(--on-radius-sm);
  }

  .launcher-btn:hover:not(:disabled),
  .launcher-btn:focus-visible,
  .media-btn:hover:not(:disabled),
  .media-btn:focus-visible,
  .switch-btn:hover,
  .switch-btn:focus-visible,
  .trace-door:hover,
  .trace-door:focus-visible {
    color: var(--on-text);
    background: var(--on-accent-subtle);
  }

  .launcher-btn:disabled,
  .media-btn:disabled {
    cursor: default;
    opacity: 0.28;
  }

  .launcher-btn svg {
    width: 22px;
    height: 22px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.55;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .chatgpt .knot {
    stroke-width: 1.2;
  }

  .claude svg {
    color: var(--on-solar);
    stroke-width: 1.35;
  }

  .status-dot {
    position: absolute;
    right: 4px;
    bottom: 4px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--on-moondust);
    opacity: 0.55;
  }

  .status-dot.running {
    background: var(--on-signal);
    opacity: 1;
  }

  .divider {
    width: 1px;
    height: 24px;
    margin: 0 3px;
    background: var(--on-hairline-strong);
    flex-shrink: 0;
  }

  .dynamic-bay {
    flex: 1;
    min-width: 0;
    height: 40px;
  }

  .trace-door,
  .media-bay {
    width: 100%;
    height: 100%;
    min-width: 0;
    display: flex;
    align-items: center;
  }

  .trace-door {
    justify-content: space-between;
    padding: 0 3px 0 6px;
    border-radius: var(--on-radius-sm);
  }

  .bay-label {
    font-family: var(--on-font-graphic);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.12em;
    color: var(--on-text-secondary);
  }

  .open-icon {
    width: 19px;
    height: 19px;
    fill: none;
    stroke: var(--on-text-quiet);
    stroke-width: 1.4;
    stroke-linecap: round;
  }

  .media-bay {
    gap: 3px;
  }

  .artwork {
    width: 30px;
    height: 30px;
    flex: 0 0 30px;
    object-fit: cover;
    border: 1px solid var(--on-hairline);
    border-radius: var(--on-radius-sm);
  }

  .artwork-fallback {
    display: grid;
    place-items: center;
    background: var(--on-surface);
    color: var(--on-text-quiet);
    font-family: var(--on-font-graphic);
    font-size: 13px;
  }

  .media-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-family: var(--on-font-interface);
    font-size: 10px;
    font-weight: 500;
    color: var(--on-text);
  }

  .media-title.idle {
    font-family: var(--on-font-mono);
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--on-text-quiet);
  }

  .media-btn,
  .switch-btn {
    display: grid;
    place-items: center;
    width: 24px;
    height: 34px;
    flex: 0 0 24px;
    color: var(--on-text-secondary);
    border-radius: var(--on-radius-sm);
  }

  .media-btn svg,
  .switch-btn svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.55;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .media-btn .fill {
    fill: currentColor;
    stroke: none;
  }

  .switch-btn {
    border-left: 1px solid var(--on-hairline);
    border-radius: 0 var(--on-radius-sm) var(--on-radius-sm) 0;
  }

  .sidecar-shell :focus-visible {
    outline-offset: -2px;
  }
</style>
