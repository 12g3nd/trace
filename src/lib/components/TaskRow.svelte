<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Task } from '$lib/types';
  import { selectedId, complete, uncomplete, edit } from '$lib/stores';
  import { formatRelativeDue } from '$lib/date-parser';

  export let task: Task;
  export let editing = false;

  const dispatch = createEventDispatcher();

  let editText = '';
  let editInput: HTMLInputElement;

  $: selected = $selectedId === task.id;
  $: isDone = task.status === 'done';
  $: priorityLevel = task.priority;
  $: dueBadge = formatRelativeDue(task.due_at);

  function handleCheck(e: Event) {
    e.stopPropagation();
    if (isDone) {
      uncomplete(task.id);
    } else {
      complete(task.id);
    }
  }

  function handleRowClick() {
    selectedId.set(task.id);
  }

  function handleDblClick() {
    startEdit();
  }

  function startEdit() {
    editText = task.raw_input || task.text + (task.context ? ` ~ ${task.context}` : '') + (task.priority ? ' ' + '*'.repeat(task.priority) : '');
    editing = true;
    // Focus after mount
    requestAnimationFrame(() => editInput?.focus());
  }

  async function commitEdit() {
    if (editText.trim()) {
      await edit(task.id, editText);
    }
    editing = false;
    dispatch('editDone');
  }

  function cancelEdit() {
    editing = false;
    dispatch('editDone');
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      commitEdit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelEdit();
    }
  }

  // Trigger edit mode when parent sets editing=true
  $: if (editing && !editText) {
    startEdit();
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<div
  class="task-row"
  class:selected
  class:done={isDone}
  class:priority-1={priorityLevel === 1}
  class:priority-2={priorityLevel === 2}
  class:priority-3={priorityLevel >= 3 && priorityLevel <= 4}
  class:priority-5={priorityLevel === 5}
  role="option"
  aria-selected={selected}
  tabindex="-1"
  on:click={handleRowClick}
  on:dblclick={handleDblClick}
>
  {#if priorityLevel > 0}
    <div class="priority-bar" aria-label="Priority {priorityLevel}"></div>
  {/if}

  <button
    class="check"
    class:checked={isDone}
    on:click={handleCheck}
    aria-label={isDone ? 'Mark incomplete' : 'Mark complete'}
  >
    {#if isDone}
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <circle cx="7" cy="7" r="6" stroke="var(--on-accent-secondary)" stroke-width="1.5" fill="rgba(57, 212, 208, 0.12)" />
        <path d="M4.5 7L6.5 9L10 5" stroke="var(--on-accent-secondary)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    {:else}
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
        <circle cx="7" cy="7" r="6" stroke="var(--on-text-quiet)" stroke-width="1.2" />
      </svg>
    {/if}
  </button>

  <div class="task-content">
    {#if editing}
      <input
        class="edit-input"
        type="text"
        bind:value={editText}
        bind:this={editInput}
        on:keydown={handleEditKeydown}
        on:blur={commitEdit}
      />
    {:else}
      <span class="task-text">{task.text}</span>
      {#if task.context}
        <span class="task-context">{task.context}</span>
      {/if}
      {#if dueBadge}
        <span class="task-due" class:overdue={dueBadge.isOverdue}>{dueBadge.label}</span>
      {/if}
    {/if}
  </div>

  {#if !editing && priorityLevel > 0}
    <div class="priority-stars" aria-label="Priority {priorityLevel}">
      {#each Array(Math.min(priorityLevel, 5)) as _}
        <span class="star">★</span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .task-row {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: var(--on-space-2);
    padding: var(--on-space-2) var(--on-space-2) var(--on-space-2) var(--on-space-3);
    border-radius: var(--on-radius-sm);
    transition: background var(--on-duration-fast) var(--on-ease);
    cursor: default;
    min-height: 36px;
  }

  .task-row:hover {
    background: rgba(232, 239, 245, 0.03);
  }

  .task-row.selected {
    background: var(--on-accent-subtle);
    border-left: 2px solid var(--on-accent);
    padding-left: calc(var(--on-space-3) - 2px);
  }

  .task-row.done {
    opacity: 0.45;
  }

  .task-row.done .task-text {
    text-decoration: line-through;
    text-decoration-color: var(--on-text-quiet);
  }

  /* Priority bar — thin left-edge marker */
  .priority-bar {
    position: absolute;
    left: 0;
    top: 4px;
    bottom: 4px;
    width: 3px;
    border-radius: 2px;
    background: var(--on-text-quiet);
  }

  .priority-1 .priority-bar  { background: var(--on-priority-1); }
  .priority-2 .priority-bar  { background: var(--on-priority-2); }
  .priority-3 .priority-bar  { background: var(--on-priority-3); }
  .priority-5 .priority-bar  { background: var(--on-priority-5); }

  /* When selected, the priority bar replaces the selection border */
  .task-row.selected .priority-bar {
    display: none;
  }

  .check {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    margin-top: 1px;
    padding: 0;
    border-radius: 50%;
    transition: opacity var(--on-duration-fast) var(--on-ease);
  }

  .check:hover {
    opacity: 0.8;
  }

  .task-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--on-space-2);
  }

  .task-text {
    font-family: var(--on-font-interface);
    font-size: 14px;
    font-weight: 400;
    color: var(--on-text);
    line-height: 1.5;
  }

  .task-context {
    font-family: var(--on-font-mono);
    font-size: 11px;
    font-weight: 400;
    color: var(--on-text-quiet);
    letter-spacing: 0.02em;
    white-space: nowrap;
  }

  .task-due {
    font-family: var(--on-font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    background: var(--on-surface-raised);
    color: var(--on-text-secondary);
    letter-spacing: 0.02em;
    white-space: nowrap;
  }

  .task-due.overdue {
    background: rgba(255, 100, 124, 0.15);
    color: var(--on-priority-5);
    border: 1px solid rgba(255, 100, 124, 0.3);
  }

  .priority-stars {
    flex-shrink: 0;
    display: flex;
    gap: 1px;
    font-size: 10px;
    color: var(--on-text-quiet);
    margin-top: 3px;
    margin-left: auto;
    padding-left: var(--on-space-2);
  }

  .priority-1 .priority-stars { color: var(--on-priority-1); }
  .priority-2 .priority-stars { color: var(--on-priority-2); }
  .priority-3 .priority-stars { color: var(--on-priority-3); }
  .priority-5 .priority-stars { color: var(--on-priority-5); }

  .star {
    font-size: 9px;
    line-height: 1;
  }

  .edit-input {
    width: 100%;
    font-family: var(--on-font-interface);
    font-size: 14px;
    color: var(--on-text);
    background: var(--on-surface-inset);
    padding: var(--on-space-1) var(--on-space-2);
    border-radius: var(--on-radius-sm);
    border: 1px solid var(--on-accent);
  }
</style>
