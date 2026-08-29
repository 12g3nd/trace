<script lang="ts">
  import { onMount } from 'svelte';
  import TaskList from '$lib/components/TaskList.svelte';
  import CaptureInput from '$lib/components/CaptureInput.svelte';
  import SearchOverlay from '$lib/components/SearchOverlay.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import UndoToast from '$lib/components/UndoToast.svelte';
  import Header from '$lib/components/Header.svelte';
  import {
    selectedId,
    searchActive,
    selectNext,
    selectPrev,
    clearSelection,
    complete,
    remove,
    move,
    reorder,
    visibleTasks,
    nowTasks,
    laterTasks,
    somedayTasks,
    performUndo,
  } from '$lib/stores';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { flushPendingTaskEdits } from '$lib/pending-edits';

  let captureRef: CaptureInput;
  let editingId: string | null = null;
  let commandPaletteActive = false;
  let settingsActive = false;
  let dismissInFlight = false;

  async function dismissMainWindow() {
    if (dismissInFlight) return;
    dismissInFlight = true;
    try {
      await flushPendingTaskEdits();
      editingId = null;
      await invoke('hide_window');
    } catch {
      // Browser development has no native window to hide.
    } finally {
      dismissInFlight = false;
    }
  }

  onMount(() => {
    // Focus capture input on initial load
    setTimeout(() => captureRef?.focus(), 50);

    const unlisteners: (() => void)[] = [];
    listen('summon', async () => {
      await flushPendingTaskEdits();
      clearSelection();
      searchActive.set(false);
      commandPaletteActive = false;
      settingsActive = false;
      editingId = null;
      captureRef?.focus();
    }).then((cleanup) => {
      unlisteners.push(cleanup);
    }).catch(() => {
      // Fallback if not running in Tauri webview
    });

    listen('main-dismiss-requested', () => {
      void dismissMainWindow();
    }).then((cleanup) => {
      unlisteners.push(cleanup);
    }).catch(() => {
      // Fallback if not running in Tauri webview
    });

    listen('main-quit-requested', async () => {
      await flushPendingTaskEdits();
      await invoke('quit_trace');
    }).then((cleanup) => {
      unlisteners.push(cleanup);
    }).catch(() => {
      // Fallback if not running in Tauri webview
    });

    return () => {
      for (const unlisten of unlisteners) unlisten();
    };
  });

  async function handleKeydown(e: KeyboardEvent) {
    // Command palette: Ctrl+K
    if (e.ctrlKey && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault();
      commandPaletteActive = !commandPaletteActive;
      if (commandPaletteActive) {
        searchActive.set(false);
        settingsActive = false;
      }
      return;
    }

    // Don't intercept other keys if modals are open
    if (commandPaletteActive) {
      if (e.key === 'Escape') {
        commandPaletteActive = false;
      }
      return;
    }

    if (settingsActive) {
      if (e.key === 'Escape') {
        settingsActive = false;
      }
      return;
    }

    // Don't intercept when typing in an input/editing
    const tag = (e.target as HTMLElement)?.tagName;
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA';

    if (e.key === 'Escape') {
      if (get(searchActive)) {
        searchActive.set(false);
        return;
      }
      if (editingId) {
        editingId = null;
        return;
      }
      if (get(selectedId)) {
        clearSelection();
        return;
      }
      // If in idle state, Escape dismisses the window cleanly
      await dismissMainWindow();
      return;
    }

    // Ctrl+Z: undo
    if (e.ctrlKey && e.key === 'z') {
      e.preventDefault();
      performUndo();
      return;
    }

    // Search: / when not in input
    if (e.key === '/' && !isInput) {
      e.preventDefault();
      searchActive.set(true);
      return;
    }

    // When search is active, let it handle keys
    if (get(searchActive)) return;

    // Tab: focus capture input
    if (e.key === 'Tab' && !e.shiftKey && !isInput) {
      e.preventDefault();
      captureRef?.focus();
      clearSelection();
      return;
    }

    // Navigation when not editing/inputting
    if (isInput) return;

    const sel = get(selectedId);

    // Alt+Arrow for reordering selected task within its group
    if (e.altKey && sel) {
      const all = get(visibleTasks);
      const task = all.find((t) => t.id === sel);
      if (task) {
        let statusTasks: typeof all;
        if (task.status === 'now') statusTasks = get(nowTasks);
        else if (task.status === 'later') statusTasks = get(laterTasks);
        else if (task.status === 'someday') statusTasks = get(somedayTasks);
        else statusTasks = [];

        const statusIdx = statusTasks.findIndex((t) => t.id === sel);

        if (e.key === 'ArrowUp' && statusIdx > 0) {
          e.preventDefault();
          reorder(statusTasks, statusIdx, statusIdx - 1);
          return;
        } else if (e.key === 'ArrowDown' && statusIdx >= 0 && statusIdx < statusTasks.length - 1) {
          e.preventDefault();
          reorder(statusTasks, statusIdx, statusIdx + 1);
          return;
        }
      }
    }

    // Ctrl+1 / Ctrl+2 / Ctrl+3: move task status
    if (e.ctrlKey && sel) {
      if (e.key === '1' || e.code === 'Digit1') {
        e.preventDefault();
        move(sel, 'now');
        return;
      }
      if (e.key === '2' || e.code === 'Digit2') {
        e.preventDefault();
        move(sel, 'later');
        return;
      }
      if (e.key === '3' || e.code === 'Digit3') {
        e.preventDefault();
        move(sel, 'someday');
        return;
      }
    }

    switch (e.key) {
      case 'ArrowDown':
      case 'j':
        e.preventDefault();
        selectNext();
        break;
      case 'ArrowUp':
      case 'k':
        e.preventDefault();
        selectPrev();
        break;
      case ' ':
        // Complete selected task
        if (sel) {
          e.preventDefault();
          complete(sel);
          // Move selection to next task
          selectNext();
        }
        break;
      case 'Enter':
        // Edit selected task
        if (sel && !editingId) {
          e.preventDefault();
          editingId = sel;
        }
        break;
      case 'Delete':
      case 'Backspace':
        if (sel) {
          e.preventDefault();
          remove(sel);
        }
        break;
    }
  }

  function handleEditDone() {
    editingId = null;
  }
</script>

<svelte:window
  on:keydown={handleKeydown}
  on:blur={() => void flushPendingTaskEdits().catch(() => undefined)}
/>

<div class="app-shell">
  <Header on:openSettings={() => (settingsActive = true)} />

  <div class="task-area">
    <TaskList
      bind:editingId
      on:editDone={handleEditDone}
    />
  </div>

  <div class="capture-area">
    <CaptureInput bind:this={captureRef} />
  </div>

  <UndoToast />

  {#if $searchActive}
    <SearchOverlay on:close={() => searchActive.set(false)} />
  {/if}

  {#if commandPaletteActive}
    <CommandPalette
      on:close={() => (commandPaletteActive = false)}
      on:openSettings={() => {
        commandPaletteActive = false;
        settingsActive = true;
      }}
    />
  {/if}

  {#if settingsActive}
    <SettingsModal on:close={() => (settingsActive = false)} />
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--on-bg);
    position: relative;
  }

  .task-area {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0 var(--on-space-4);
  }

  .capture-area {
    flex-shrink: 0;
    border-top: 1px solid var(--on-hairline);
    padding: var(--on-space-3) var(--on-space-4);
    background: var(--on-surface-inset);
  }
</style>
