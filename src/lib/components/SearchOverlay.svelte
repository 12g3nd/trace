<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { filteredTasks, searchQuery, selectedId } from '$lib/stores';
  import type { Task } from '$lib/types';
  import { STATUS_LABELS } from '$lib/types';

  const dispatch = createEventDispatcher();

  let inputEl: HTMLInputElement;

  onMount(() => {
    inputEl?.focus();
    searchQuery.set('');
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      searchQuery.set('');
      dispatch('close');
    } else if (e.key === 'Enter') {
      // Select first result
      const results = $filteredTasks;
      if (results.length > 0) {
        selectedId.set(results[0].id);
        searchQuery.set('');
        dispatch('close');
      }
    }
  }

  function selectTask(task: Task) {
    selectedId.set(task.id);
    searchQuery.set('');
    dispatch('close');
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div class="search-overlay" role="dialog" aria-label="Search tasks" on:click|self={() => dispatch('close')}>
  <div class="search-panel">
    <div class="search-bar">
      <span class="search-icon" aria-hidden="true">/</span>
      <input
        bind:this={inputEl}
        bind:value={$searchQuery}
        type="text"
        class="search-input"
        placeholder="search tasks..."
        on:keydown={handleKeydown}
        aria-label="Search"
      />
    </div>

    {#if $filteredTasks.length > 0}
      <div class="search-results" role="listbox">
        {#each $filteredTasks.slice(0, 20) as task (task.id)}
          <button
            class="search-result"
            role="option"
            on:click={() => selectTask(task)}
          >
            <span class="result-text">{task.text}</span>
            <span class="result-meta">
              {#if task.context}
                <span class="result-context">{task.context}</span>
              {/if}
              <span class="result-status">{STATUS_LABELS[task.status]}</span>
            </span>
          </button>
        {/each}
      </div>
    {:else if $searchQuery.trim()}
      <div class="no-results">No matches.</div>
    {/if}
  </div>
</div>

<style>
  .search-overlay {
    position: fixed;
    inset: 0;
    background: rgba(11, 23, 49, 0.7);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
    z-index: 100;
  }

  .search-panel {
    width: min(380px, 90vw);
    background: var(--on-surface);
    border: 1px solid var(--on-hairline-strong);
    border-radius: var(--on-radius-md);
    overflow: hidden;
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: var(--on-space-2);
    padding: var(--on-space-3) var(--on-space-4);
    border-bottom: 1px solid var(--on-hairline);
  }

  .search-icon {
    font-family: var(--on-font-mono);
    font-size: 13px;
    color: var(--on-text-quiet);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    font-family: var(--on-font-interface);
    font-size: 14px;
    color: var(--on-text);
    background: transparent;
    border: none;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--on-text-quiet);
  }

  .search-results {
    max-height: 300px;
    overflow-y: auto;
    padding: var(--on-space-1) 0;
  }

  .search-result {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--on-space-3);
    width: 100%;
    padding: var(--on-space-2) var(--on-space-4);
    text-align: left;
    transition: background var(--on-duration-fast) var(--on-ease);
    cursor: pointer;
  }

  .search-result:hover {
    background: rgba(232, 239, 245, 0.05);
  }

  .result-text {
    font-size: 13px;
    color: var(--on-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-meta {
    display: flex;
    gap: var(--on-space-2);
    flex-shrink: 0;
  }

  .result-context {
    font-family: var(--on-font-mono);
    font-size: 10px;
    color: var(--on-text-quiet);
  }

  .result-status {
    font-family: var(--on-font-mono);
    font-size: 10px;
    color: var(--on-text-quiet);
    opacity: 0.6;
    text-transform: lowercase;
  }

  .no-results {
    padding: var(--on-space-4);
    text-align: center;
    font-size: 13px;
    color: var(--on-text-quiet);
  }
</style>
