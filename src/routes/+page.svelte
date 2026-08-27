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

  let captureRef: CaptureInput;
  let editingId: string | null = null;
  let commandPaletteActive = false;
  let settingsActive = false;

  onMount(() => {
    // Focus capture input on initial load
    setTimeout(() => captureRef?.focus(), 50);

    let unlisten: (() => void) | undefined;
    listen('summon', () => {
      clearSelection();
      searchActive.set(false);
      commandPaletteActive = false;
      settingsActive = false;
      editingId = null;
      captureRef?.focus();
    }).then((cleanup) => {
      unlisten = cleanup;
    }).catch(() => {
      // Fallback if not running in Tauri webview
    });

    return () => {
      if (unlisten) unlisten();
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
      try {
        await invoke('hide_window');
      } catch {
        // Fallback for non-Tauri dev
      }
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
      case '1':
        if (e.ctrlKey && sel) { e.preventDefault(); move(sel, 'now'); }
        break;
      case '2':
        if (e.ctrlKey && sel) { e.preventDefault(); move(sel, 'later'); }
        break;
      case '3':
        if (e.ctrlKey && sel) { e.preventDefault(); move(sel, 'someday'); }
        break;
      default:
        // Alt+Arrow for reorder
        if (e.altKey && sel) {
          const visible = get(visibleTasks);
          const idx = visible.findIndex((t) => t.id === sel);
          if (idx === -1) break;

          // Determine which status group the task is in
          const task = visible[idx];
          let statusTasks: typeof visible;
          if (task.status === 'now') statusTasks = get(nowTasks);
          else if (task.status === 'later') statusTasks = get(laterTasks);
          else statusTasks = get(somedayTasks);

          const statusIdx = statusTasks.findIndex((t) => t.id === sel);

          if (e.key === 'ArrowUp' && statusIdx > 0) {
            e.preventDefault();
            reorder(statusTasks, statusIdx, statusIdx - 1);
          } else if (e.key === 'ArrowDown' && statusIdx < statusTasks.length - 1) {
            e.preventDefault();
            reorder(statusTasks, statusIdx, statusIdx + 1);
          }
        }
        break;
    }
  }

  function handleEditDone() {
    editingId = null;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

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
