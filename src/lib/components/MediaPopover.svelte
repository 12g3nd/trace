<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { emitTo, listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    EMPTY_MEDIA,
    MEDIA_POPOVER_HOVER_EVENT,
    MEDIA_REFRESH_EVENT,
    MEDIA_STATE_EVENT,
    type MediaState,
  } from '$lib/media-state';

  let media = EMPTY_MEDIA;
  let unlistenState: UnlistenFn | undefined;
  let disposed = false;

  $: title = media.available ? media.title.trim() || 'UNTITLED' : 'NO MEDIA';
  $: artist = media.available ? media.artist.trim() || 'Unknown artist' : 'No active session';

  onMount(() => {
    document.documentElement.classList.add('shell-document');

    void listen<MediaState>(MEDIA_STATE_EVENT, (event) => updateMedia(event.payload))
      .then((unlisten) => {
        if (disposed) unlisten();
        else unlistenState = unlisten;
      })
      .catch((error) => console.warn('[Trace Media] Could not subscribe to media state:', error));

    return () => {
      disposed = true;
      unlistenState?.();
      document.documentElement.classList.remove('shell-document');
      void reportHover(false);
    };
  });

  function updateMedia(next: MediaState) {
    if (next.artworkKey === media.artworkKey && !next.artwork && media.artwork) {
      next.artwork = media.artwork;
    }
    media = next;
  }

  async function reportHover(hovered: boolean) {
    try {
      await emitTo('sidecar', MEDIA_POPOVER_HOVER_EVENT, hovered);
    } catch (error) {
      console.warn('[Trace Media] Could not report popover hover state:', error);
    }
  }

  async function openSource() {
    if (!media.available) return;
    try {
      await invoke('open_media_source');
    } catch (error) {
      console.warn('[Trace Media] Could not activate media source:', error);
    }
  }

  async function runMediaCommand(action: 'previous' | 'toggle' | 'next') {
    try {
      await invoke('media_command', { action });
      await emitTo('sidecar', MEDIA_REFRESH_EVENT);
    } catch (error) {
      console.warn(`[Trace Media] Media command ${action} failed:`, error);
    }
  }
</script>

<main
  class="popover-shell"
  aria-label="Expanded media controls"
  on:mouseenter={() => reportHover(true)}
  on:mouseleave={() => reportHover(false)}
>
  <button
    class="source-surface"
    tabindex="-1"
    disabled={!media.available}
    title={media.available ? `Open ${media.source || 'media player'}` : 'No active media session'}
    aria-label={media.available ? 'Open current media source' : 'No active media session'}
    on:click={openSource}
  ></button>

  <div class="player-layout">
    {#if media.artwork}
      <img class="album-art" src={media.artwork} alt="" />
    {:else}
      <div class="album-art album-art-fallback" aria-hidden="true">
        <svg viewBox="0 0 24 24">
          <path d="M9.5 17V6.6l8-1.8v10M9.5 17c0 1.2-1.3 2.1-2.8 2.1S4 18.2 4 17s1.2-2.1 2.7-2.1 2.8.9 2.8 2.1Zm8-2.2c0 1.2-1.2 2.1-2.7 2.1S12 16 12 14.8s1.3-2.1 2.8-2.1 2.7.9 2.7 2.1Z" />
        </svg>
      </div>
    {/if}

    <section class="media-details" aria-live="polite">
      <div class="metadata">
        <div class:idle={!media.available} class="track-title" title={title}>{title}</div>
        <div class="track-artist" title={artist}>{artist}</div>
      </div>

      <div class="transport" aria-label="Media transport controls">
        <button
          class="transport-btn"
          disabled={!media.available || !media.canPrevious}
          title="Previous"
          aria-label="Previous media track"
          on:click={() => runMediaCommand('previous')}
        >
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path d="M6.2 5.5v9" />
            <path class="fill" d="m14.5 5.5-6.4 4.5 6.4 4.5z" />
          </svg>
        </button>
        <button
          class="transport-btn primary"
          disabled={!media.available || !media.canToggle}
          title={media.playing ? 'Pause' : 'Play'}
          aria-label={media.playing ? 'Pause current media' : 'Play current media'}
          on:click={() => runMediaCommand('toggle')}
        >
          {#if media.playing}
            <svg viewBox="0 0 20 20" aria-hidden="true"><path d="M7 5.5v9M13 5.5v9" /></svg>
          {:else}
            <svg viewBox="0 0 20 20" aria-hidden="true"><path class="fill" d="m7 5 7.5 5L7 15z" /></svg>
          {/if}
        </button>
        <button
          class="transport-btn"
          disabled={!media.available || !media.canNext}
          title="Next"
          aria-label="Next media track"
          on:click={() => runMediaCommand('next')}
        >
          <svg viewBox="0 0 20 20" aria-hidden="true">
            <path class="fill" d="m5.5 5.5 6.4 4.5-6.4 4.5z" />
            <path d="M13.8 5.5v9" />
          </svg>
        </button>
      </div>
    </section>
  </div>
</main>

<style>
  .popover-shell {
    position: relative;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    padding: 10px;
    color: var(--on-text);
    background: rgba(11, 23, 49, 0.8);
    border: 1px solid rgba(127, 166, 196, 0.17);
    border-radius: 10px;
    user-select: none;
  }

  .source-surface {
    position: absolute;
    inset: 0;
    z-index: 1;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 0;
    border-radius: 10px;
    cursor: pointer;
  }

  .source-surface:disabled {
    cursor: default;
  }

  .player-layout {
    position: relative;
    z-index: 2;
    height: 100%;
    display: grid;
    grid-template-columns: 68px minmax(0, 1fr);
    align-items: center;
    gap: 12px;
    pointer-events: none;
  }

  .album-art {
    width: 68px;
    height: 68px;
    display: block;
    border-radius: 8px;
    object-fit: cover;
    background: rgba(16, 42, 76, 0.68);
  }

  .album-art-fallback {
    display: grid;
    place-items: center;
    color: var(--on-text-quiet);
  }

  .album-art-fallback svg {
    width: 28px;
    height: 28px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .media-details {
    min-width: 0;
    height: 70px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .metadata {
    min-width: 0;
    padding-top: 1px;
  }

  .track-title,
  .track-artist {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track-title {
    font-family: var(--on-font-graphic);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.01em;
    color: var(--on-text);
  }

  .track-title.idle {
    font-family: var(--on-font-mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    color: var(--on-text-quiet);
  }

  .track-artist {
    margin-top: 2px;
    font-family: var(--on-font-interface);
    font-size: 10px;
    color: rgba(168, 184, 202, 0.74);
  }

  .transport {
    display: flex;
    align-items: center;
    gap: 5px;
    pointer-events: auto;
  }

  .transport-btn {
    width: 28px;
    height: 27px;
    display: grid;
    place-items: center;
    padding: 0;
    border-radius: 7px;
    color: var(--on-text-secondary);
    transition: color 150ms var(--on-ease), background 150ms var(--on-ease), opacity 150ms var(--on-ease);
  }

  .transport-btn.primary {
    width: 32px;
    color: var(--on-text);
    background: rgba(127, 166, 196, 0.1);
  }

  .transport-btn:hover:not(:disabled),
  .transport-btn:focus-visible {
    color: var(--on-text);
    background: rgba(127, 166, 196, 0.14);
  }

  .transport-btn:disabled {
    opacity: 0.25;
  }

  .transport-btn svg {
    width: 17px;
    height: 17px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.35;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .transport-btn .fill {
    fill: currentColor;
    stroke: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .transport-btn {
      transition: none;
    }
  }
</style>
