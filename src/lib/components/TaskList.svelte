<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import TaskRow from './TaskRow.svelte';
  import {
    nowTasks,
    laterTasks,
    somedayTasks,
    taskCounts,
  } from '$lib/stores';
  import type { Status } from '$lib/types';

  export let editingId: string | null = null;

  const dispatch = createEventDispatcher();

  let laterOpen = false;
  let somedayOpen = false;

  function toggleSection(section: 'later' | 'someday') {
    if (section === 'later') laterOpen = !laterOpen;
    else somedayOpen = !somedayOpen;
  }

  function handleEditStart(event: CustomEvent<{ id: string }>) {
    editingId = event.detail.id;
  }

  function handleEditDone(event: CustomEvent<{ id: string }>) {
    if (editingId !== event.detail.id) return;
    editingId = null;
    dispatch('editDone');
  }
</script>

<div class="task-list" role="listbox" aria-label="Tasks">
  <!-- NOW section — always visible, no header needed for the primary group -->
  {#if $nowTasks.length === 0}
    <div class="empty-state">Nothing queued.</div>
  {:else}
    {#each $nowTasks as task (task.id)}
      <TaskRow
        {task}
        editing={editingId === task.id}
        on:editStart={handleEditStart}
        on:editDone={handleEditDone}
      />
    {/each}
  {/if}

  <!-- LATER section -->
  {#if $taskCounts.later > 0}
    <button
      class="section-toggle"
      on:click={() => toggleSection('later')}
      aria-expanded={laterOpen}
    >
      <span class="section-arrow" class:open={laterOpen}>▸</span>
      <span class="section-label">LATER</span>
      <span class="section-count">{$taskCounts.later}</span>
    </button>

    {#if laterOpen}
      <div class="section-content">
        {#each $laterTasks as task (task.id)}
          <TaskRow
            {task}
            editing={editingId === task.id}
            on:editStart={handleEditStart}
            on:editDone={handleEditDone}
          />
        {/each}
      </div>
    {/if}
  {/if}

  <!-- SOMEDAY section -->
  {#if $taskCounts.someday > 0}
    <button
      class="section-toggle"
      on:click={() => toggleSection('someday')}
      aria-expanded={somedayOpen}
    >
      <span class="section-arrow" class:open={somedayOpen}>▸</span>
      <span class="section-label">SOMEDAY</span>
      <span class="section-count">{$taskCounts.someday}</span>
    </button>

    {#if somedayOpen}
      <div class="section-content">
        {#each $somedayTasks as task (task.id)}
          <TaskRow
            {task}
            editing={editingId === task.id}
            on:editStart={handleEditStart}
            on:editDone={handleEditDone}
          />
        {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .task-list {
    padding: var(--on-space-1) 0 var(--on-space-4);
  }

  .empty-state {
    padding: var(--on-space-8) var(--on-space-4);
    text-align: center;
    font-family: var(--on-font-interface);
    font-size: 13px;
    color: var(--on-text-quiet);
  }

  .section-toggle {
    display: flex;
    align-items: center;
    gap: var(--on-space-2);
    width: 100%;
    padding: var(--on-space-3) var(--on-space-3);
    margin-top: var(--on-space-2);
    border-top: 1px solid var(--on-hairline);
    cursor: pointer;
    transition: background var(--on-duration-fast) var(--on-ease);
    border-radius: var(--on-radius-sm);
  }

  .section-toggle:hover {
    background: rgba(232, 239, 245, 0.03);
  }

  .section-arrow {
    font-size: 10px;
    color: var(--on-text-quiet);
    transition: transform var(--on-duration-fast) var(--on-ease);
    display: inline-block;
  }

  .section-arrow.open {
    transform: rotate(90deg);
  }

  .section-label {
    font-family: var(--on-font-graphic);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--on-text-quiet);
  }

  .section-count {
    font-family: var(--on-font-mono);
    font-size: 10px;
    color: var(--on-text-quiet);
    opacity: 0.6;
    font-variant-numeric: tabular-nums;
  }

  .section-content {
    padding: var(--on-space-1) 0;
  }
</style>
