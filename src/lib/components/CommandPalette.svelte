<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { tasks, refresh, doneTasks } from '$lib/stores';
  import { formatAsTodoTxt, formatAsJson, formatAsCsv, downloadFile } from '$lib/export';
  import { deleteTask } from '$lib/task-service';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher();

  let query = '';
  let selectedIndex = 0;
  let inputEl: HTMLInputElement;

  interface Command {
    id: string;
    title: string;
    description: string;
    icon: string;
    action: () => Promise<void> | void;
  }

  const commands: Command[] = [
    {
      id: 'export-txt',
      title: 'Export as TODO.txt',
      description: 'Download tasks in standard plain text format',
      icon: 'TXT',
      action: () => {
        const all = get(tasks);
        const text = formatAsTodoTxt(all);
        const date = new Date().toISOString().slice(0, 10);
        downloadFile(text, `todo-${date}.txt`, 'text/plain');
      },
    },
    {
      id: 'export-json',
      title: 'Export as JSON',
      description: 'Download full structured task data',
      icon: 'JSON',
      action: () => {
        const all = get(tasks);
        const json = formatAsJson(all);
        const date = new Date().toISOString().slice(0, 10);
        downloadFile(json, `todo-${date}.json`, 'application/json');
      },
    },
    {
      id: 'export-csv',
      title: 'Export as CSV',
      description: 'Download spreadsheet-compatible task list',
      icon: 'CSV',
      action: () => {
        const all = get(tasks);
        const csv = formatAsCsv(all);
        const date = new Date().toISOString().slice(0, 10);
        downloadFile(csv, `todo-${date}.csv`, 'text/csv');
      },
    },
    {
      id: 'clear-done',
      title: 'Clear Completed Tasks',
      description: 'Permanently remove all tasks in Done',
      icon: 'CLR',
      action: async () => {
        const done = get(doneTasks);
        for (const task of done) {
          await deleteTask(task.id);
        }
        await refresh();
      },
    },
    {
      id: 'settings',
      title: 'Settings & Diagnostics',
      description: 'View shortcuts, system startup, and app info',
      icon: 'SET',
      action: () => {
        dispatch('openSettings');
      },
    },
    {
      id: 'reanchor-sidecar',
      title: 'Re-anchor Sidecar',
      description: 'Return Sidecar to the primary display bottom-left rail position',
      icon: 'POS',
      action: async () => {
        await invoke('reanchor_sidecar');
      },
    },
    {
      id: 'quit-trace',
      title: 'Quit Trace',
      description: 'Exit the main window and Orbit Sidecar completely',
      icon: 'OFF',
      action: async () => {
        await invoke('quit_trace');
      },
    },
  ];

  $: filteredCommands = commands.filter(
    (c) =>
      c.title.toLowerCase().includes(query.toLowerCase()) ||
      c.description.toLowerCase().includes(query.toLowerCase())
  );

  $: if (selectedIndex >= filteredCommands.length) {
    selectedIndex = Math.max(0, filteredCommands.length - 1);
  }

  onMount(() => {
    inputEl?.focus();
  });

  async function execute(cmd: Command) {
    await cmd.action();
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      dispatch('close');
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (selectedIndex < filteredCommands.length - 1) selectedIndex++;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (selectedIndex > 0) selectedIndex--;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const cmd = filteredCommands[selectedIndex];
      if (cmd) execute(cmd);
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="palette-overlay" role="dialog" aria-modal="true" aria-label="Command Palette" tabindex="-1" on:click|self={() => dispatch('close')}>
  <div class="palette-panel">
    <div class="palette-search">
      <span class="palette-prefix">&gt;</span>
      <input
        bind:this={inputEl}
        bind:value={query}
        type="text"
        class="palette-input"
        placeholder="Type a command..."
        on:keydown={handleKeydown}
        aria-label="Command"
      />
    </div>

    {#if filteredCommands.length > 0}
      <div class="palette-list" role="listbox">
        {#each filteredCommands as cmd, i (cmd.id)}
          <button
            class="palette-item"
            class:active={i === selectedIndex}
            role="option"
            aria-selected={i === selectedIndex}
            on:click={() => execute(cmd)}
            on:mouseenter={() => (selectedIndex = i)}
          >
            <span class="item-icon">{cmd.icon}</span>
            <div class="item-info">
              <span class="item-title">{cmd.title}</span>
              <span class="item-desc">{cmd.description}</span>
            </div>
          </button>
        {/each}
      </div>
    {:else}
      <div class="no-commands">No matching commands.</div>
    {/if}
  </div>
</div>

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    background: rgba(11, 23, 49, 0.75);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 60px;
    z-index: 110;
  }

  .palette-panel {
    width: min(380px, 92vw);
    background: var(--on-surface);
    border: 1px solid var(--on-hairline-strong);
    border-radius: var(--on-radius-md);
    overflow: hidden;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  }

  .palette-search {
    display: flex;
    align-items: center;
    gap: var(--on-space-2);
    padding: var(--on-space-3) var(--on-space-4);
    border-bottom: 1px solid var(--on-hairline);
    background: var(--on-surface-inset);
  }

  .palette-prefix {
    font-family: var(--on-font-mono);
    font-size: 13px;
    color: var(--on-accent);
    flex-shrink: 0;
  }

  .palette-input {
    flex: 1;
    font-family: var(--on-font-interface);
    font-size: 14px;
    color: var(--on-text);
    background: transparent;
    border: none;
    outline: none;
  }

  .palette-input::placeholder {
    color: var(--on-text-quiet);
  }

  .palette-list {
    max-height: 280px;
    overflow-y: auto;
    padding: var(--on-space-1) 0;
  }

  .palette-item {
    display: flex;
    align-items: center;
    gap: var(--on-space-3);
    width: 100%;
    padding: var(--on-space-2) var(--on-space-4);
    text-align: left;
    transition: background var(--on-duration-fast) var(--on-ease);
    cursor: pointer;
  }

  .palette-item:hover,
  .palette-item.active {
    background: var(--on-accent-subtle);
    border-left: 2px solid var(--on-accent);
    padding-left: calc(var(--on-space-4) - 2px);
  }

  .item-icon {
    font-family: var(--on-font-mono);
    font-size: 9px;
    font-weight: 500;
    color: var(--on-text-secondary);
    background: var(--on-surface-raised);
    padding: 2px 4px;
    border-radius: 3px;
    min-width: 32px;
    text-align: center;
    flex-shrink: 0;
  }

  .item-info {
    display: flex;
    flex-direction: column;
    gap: 1px;
    overflow: hidden;
  }

  .item-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--on-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .item-desc {
    font-size: 11px;
    color: var(--on-text-quiet);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .no-commands {
    padding: var(--on-space-4);
    text-align: center;
    font-size: 13px;
    color: var(--on-text-quiet);
  }
</style>
