<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { initDb } from '$lib/db';
  import { refresh } from '$lib/stores';
  import OrbitSidecar from '$lib/components/OrbitSidecar.svelte';

  let windowKind: 'loading' | 'main' | 'sidecar' = 'loading';
  let ready = false;
  let loading = true;
  let initError: string | null = null;

  async function initialize() {
    loading = true;
    initError = null;
    try {
      await initDb();
      await refresh();
      ready = true;
    } catch (err: unknown) {
      console.error('[Trace] Database initialization failed:', err);
      initError = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    try {
      windowKind = getCurrentWindow().label === 'sidecar' ? 'sidecar' : 'main';
    } catch {
      // Browser development always renders the main Trace application.
      windowKind = 'main';
    }

    if (windowKind === 'main') initialize();
  });
</script>

{#if windowKind === 'sidecar'}
  <OrbitSidecar />
{:else if windowKind === 'main' && ready}
  <slot />
{:else if windowKind === 'main' && initError}
  <div class="init-error-container">
    <div class="init-error-card">
      <div class="init-error-header">
        <span class="init-badge">DATABASE INITIALIZATION ERROR</span>
      </div>
      <div class="init-error-msg">
        Trace could not connect to or migrate its local storage (<code>trace.db</code>).
      </div>
      <pre class="init-error-trace">{initError}</pre>
      <div class="init-error-actions">
        <button class="retry-btn" on:click={initialize}>Retry Connection</button>
      </div>
    </div>
  </div>
{:else if windowKind === 'main' && loading}
  <div class="init-loading-container">
    <div class="init-spinner"></div>
  </div>
{/if}

<style>
  .init-error-container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    padding: var(--on-space-4);
    background: var(--on-bg);
    color: var(--on-text);
  }

  .init-error-card {
    max-width: 360px;
    width: 100%;
    background: var(--on-surface);
    border: 1px solid var(--on-priority-5);
    border-radius: var(--on-radius-md);
    padding: var(--on-space-4);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  }

  .init-badge {
    font-family: var(--on-font-graphic);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--on-priority-5);
  }

  .init-error-msg {
    font-family: var(--on-font-interface);
    font-size: 12px;
    color: var(--on-text-secondary);
    margin-top: var(--on-space-2);
    line-height: 1.4;
  }

  .init-error-trace {
    font-family: var(--on-font-mono);
    font-size: 11px;
    background: var(--on-surface-inset);
    border: 1px solid var(--on-hairline);
    padding: var(--on-space-2);
    border-radius: var(--on-radius-sm);
    color: var(--on-text);
    margin-top: var(--on-space-3);
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 140px;
    overflow-y: auto;
  }

  .init-error-actions {
    margin-top: var(--on-space-3);
    display: flex;
    justify-content: flex-end;
  }

  .retry-btn {
    font-family: var(--on-font-interface);
    font-size: 11px;
    font-weight: 500;
    padding: var(--on-space-1) var(--on-space-3);
    background: var(--on-accent-subtle);
    border: 1px solid var(--on-accent);
    color: var(--on-accent);
    border-radius: var(--on-radius-sm);
    cursor: pointer;
    transition: all var(--on-duration-fast) var(--on-ease);
  }

  .retry-btn:hover {
    background: var(--on-accent);
    color: var(--on-surface);
  }

  .init-loading-container {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--on-bg);
  }

  .init-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--on-hairline);
    border-top-color: var(--on-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
