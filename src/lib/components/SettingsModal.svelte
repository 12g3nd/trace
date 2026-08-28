<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { taskCounts, tasks } from '$lib/stores';
  import { isEnabled, enable, disable } from '@tauri-apps/plugin-autostart';
  import { formatAsTodoTxt, formatAsJson, formatAsCsv, downloadFile } from '$lib/export';
  import { get } from 'svelte/store';

  const dispatch = createEventDispatcher();

  let autostartActive = false;
  let autostartSupported = true;

  onMount(async () => {
    try {
      autostartActive = await isEnabled();
    } catch {
      autostartSupported = false;
    }
  });

  async function toggleAutostart() {
    try {
      if (autostartActive) {
        await disable();
        autostartActive = false;
      } else {
        await enable();
        autostartActive = true;
      }
    } catch (e) {
      console.error('Failed to toggle autostart', e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      dispatch('close');
    }
  }

  function exportTxt() {
    const all = get(tasks);
    downloadFile(formatAsTodoTxt(all), `todo-${new Date().toISOString().slice(0, 10)}.txt`, 'text/plain');
  }

  function exportJson() {
    const all = get(tasks);
    downloadFile(formatAsJson(all), `todo-${new Date().toISOString().slice(0, 10)}.json`, 'application/json');
  }

  function exportCsv() {
    const all = get(tasks);
    downloadFile(formatAsCsv(all), `todo-${new Date().toISOString().slice(0, 10)}.csv`, 'text/csv');
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="modal-overlay" role="dialog" aria-modal="true" aria-label="Settings" tabindex="-1" on:click|self={() => dispatch('close')}>
  <div class="modal-panel">
    <div class="modal-header">
      <span class="modal-title">TRACE</span>
      <button class="close-btn" on:click={() => dispatch('close')} aria-label="Close settings">✕</button>
    </div>

    <div class="modal-body">
      <!-- Section: System Integration -->
      <div class="section">
        <div class="section-title">SYSTEM STARTUP</div>
        {#if autostartSupported}
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Launch at Windows startup</span>
              <span class="setting-sub">Run quietly in system tray on boot</span>
            </div>
            <button
              class="toggle-btn"
              class:active={autostartActive}
              on:click={toggleAutostart}
              aria-pressed={autostartActive}
            >
              {autostartActive ? 'ON' : 'OFF'}
            </button>
          </div>
        {:else}
          <div class="setting-sub">Autostart plugin active in desktop mode.</div>
        {/if}
      </div>

      <!-- Section: Shortcuts Reference -->
      <div class="section">
        <div class="section-title">KEYBOARD SHORTCUTS</div>
        <div class="shortcuts-grid">
          <div class="shortcut-row"><kbd>Win+Shift+T</kbd><span>Global summon / hide</span></div>
          <div class="shortcut-row"><kbd>Ctrl+K</kbd><span>Command palette</span></div>
          <div class="shortcut-row"><kbd>/</kbd><span>Search tasks</span></div>
          <div class="shortcut-row"><kbd>Tab</kbd><span>Focus capture input</span></div>
          <div class="shortcut-row"><kbd>Space</kbd><span>Toggle complete</span></div>
          <div class="shortcut-row"><kbd>Enter</kbd><span>Inline edit</span></div>
          <div class="shortcut-row"><kbd>Alt+↑/↓</kbd><span>Reorder in list</span></div>
          <div class="shortcut-row"><kbd>Ctrl+1/2/3</kbd><span>Move Now / Later / Someday</span></div>
          <div class="shortcut-row"><kbd>Ctrl+Z</kbd><span>Undo last action</span></div>
          <div class="shortcut-row"><kbd>Esc</kbd><span>Dismiss / Deselect</span></div>
        </div>
      </div>

      <!-- Section: Storage & Export -->
      <div class="section">
        <div class="section-title">DATA & PORTABILITY</div>
        <div class="stats-row">
          <div class="stat-item">
            <span class="stat-num">{$taskCounts.now}</span>
            <span class="stat-label">Now</span>
          </div>
          <div class="stat-item">
            <span class="stat-num">{$taskCounts.later}</span>
            <span class="stat-label">Later</span>
          </div>
          <div class="stat-item">
            <span class="stat-num">{$taskCounts.someday}</span>
            <span class="stat-label">Someday</span>
          </div>
          <div class="stat-item">
            <span class="stat-num">{$taskCounts.done}</span>
            <span class="stat-label">Done</span>
          </div>
        </div>

        <div class="export-actions">
          <button class="export-btn" on:click={exportTxt}>Export .TXT</button>
          <button class="export-btn" on:click={exportJson}>Export .JSON</button>
          <button class="export-btn" on:click={exportCsv}>Export .CSV</button>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(11, 23, 49, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 120;
  }

  .modal-panel {
    width: min(380px, 92vw);
    max-height: 85vh;
    background: var(--on-surface);
    border: 1px solid var(--on-hairline-strong);
    border-radius: var(--on-radius-md);
    overflow-y: auto;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--on-space-3) var(--on-space-4);
    border-bottom: 1px solid var(--on-hairline);
    background: var(--on-surface-inset);
  }

  .modal-title {
    font-family: var(--on-font-graphic);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--on-text);
  }

  .close-btn {
    font-size: 13px;
    color: var(--on-text-quiet);
    padding: 2px 6px;
    border-radius: var(--on-radius-sm);
    transition: color var(--on-duration-fast) var(--on-ease);
  }

  .close-btn:hover {
    color: var(--on-text);
  }

  .modal-body {
    padding: var(--on-space-3) var(--on-space-4) var(--on-space-4);
    display: flex;
    flex-direction: column;
    gap: var(--on-space-4);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: var(--on-space-2);
  }

  .section-title {
    font-family: var(--on-font-graphic);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--on-text-quiet);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--on-space-2) 0;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .setting-label {
    font-size: 13px;
    color: var(--on-text);
  }

  .setting-sub {
    font-size: 11px;
    color: var(--on-text-quiet);
  }

  .toggle-btn {
    font-family: var(--on-font-mono);
    font-size: 11px;
    padding: 3px 8px;
    border-radius: var(--on-radius-sm);
    border: 1px solid var(--on-hairline-strong);
    color: var(--on-text-quiet);
    transition: all var(--on-duration-fast) var(--on-ease);
  }

  .toggle-btn.active {
    background: var(--on-accent-subtle);
    border-color: var(--on-accent);
    color: var(--on-accent);
  }

  .shortcuts-grid {
    display: flex;
    flex-direction: column;
    gap: var(--on-space-1);
    font-size: 12px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 0;
  }

  .shortcut-row span {
    color: var(--on-text-secondary);
  }

  kbd {
    font-family: var(--on-font-mono);
    font-size: 10px;
    background: var(--on-surface-inset);
    border: 1px solid var(--on-hairline);
    padding: 1px 5px;
    border-radius: 3px;
    color: var(--on-text);
  }

  .stats-row {
    display: flex;
    gap: var(--on-space-2);
    padding: var(--on-space-1) 0;
  }

  .stat-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    background: var(--on-surface-inset);
    padding: var(--on-space-2);
    border-radius: var(--on-radius-sm);
    border: 1px solid var(--on-hairline);
  }

  .stat-num {
    font-family: var(--on-font-mono);
    font-size: 14px;
    font-weight: 500;
    color: var(--on-text);
  }

  .stat-label {
    font-size: 10px;
    color: var(--on-text-quiet);
  }

  .export-actions {
    display: flex;
    gap: var(--on-space-2);
    margin-top: var(--on-space-1);
  }

  .export-btn {
    flex: 1;
    font-family: var(--on-font-interface);
    font-size: 11px;
    font-weight: 500;
    padding: var(--on-space-2);
    background: var(--on-surface-raised);
    border: 1px solid var(--on-hairline);
    border-radius: var(--on-radius-sm);
    color: var(--on-text);
    text-align: center;
    transition: background var(--on-duration-fast) var(--on-ease);
  }

  .export-btn:hover {
    background: var(--on-accent-subtle);
    border-color: var(--on-accent);
  }
</style>
