<script lang="ts">
  import { createEventDispatcher, onDestroy } from 'svelte';
  import type { Task } from '$lib/types';
  import { selectedId, complete, uncomplete, edit } from '$lib/stores';
  import { formatRelativeDue } from '$lib/date-parser';
  import { normalizeTaskLink } from '$lib/url';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { createDebouncedSaver } from '$lib/debounced-save';
  import { registerPendingTaskEdit } from '$lib/pending-edits';

  export let task: Task;
  export let editing = false;

  const dispatch = createEventDispatcher();

  let editText = '';
  let editLink = '';
  let linkError: string | null = null;
  let editInput: HTMLInputElement;
  let editPanel: HTMLDivElement;
  let unregisterPendingEdit: (() => void) | null = null;
  let finishingEdit = false;
  let lastSavedText = '';
  let lastSavedLink: string | null = null;

  interface EditDraft {
    text: string;
    link: string;
  }

  const autosave = createDebouncedSaver<EditDraft>(650, persistDraft);

  $: selected = $selectedId === task.id;
  $: isDone = task.status === 'done';
  $: priorityLevel = task.priority;
  $: dueBadge = formatRelativeDue(task.due_at);

  let wasEditing = false;

  onDestroy(() => {
    unregisterPendingEdit?.();
    void autosave.flush().catch(() => undefined);
  });

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
    if (wasEditing) return;
    editText = task.raw_input || task.text + (task.context ? ` ~ ${task.context}` : '') + (task.priority ? ' ' + '*'.repeat(task.priority) : '');
    editLink = task.link ?? '';
    lastSavedText = editText.trim();
    lastSavedLink = task.link;
    linkError = null;
    editing = true;
    wasEditing = true;
    unregisterPendingEdit?.();
    unregisterPendingEdit = registerPendingTaskEdit(autosave.flush);
    dispatch('editStart', { id: task.id });
    requestAnimationFrame(() => editInput?.focus());
  }

  function currentDraft(): EditDraft {
    return { text: editText, link: editLink };
  }

  function scheduleAutosave() {
    autosave.schedule(currentDraft());
  }

  async function persistDraft(draft: EditDraft) {
    const trimmed = draft.text.trim();
    if (!trimmed) return;

    const normalizedLink = normalizeTaskLink(draft.link);
    if (normalizedLink.error) {
      linkError = normalizedLink.error;
    } else {
      linkError = null;
    }

    // Invalid link text remains visible for correction; valid task text still
    // autosaves with the last accepted link rather than being lost with it.
    const linkToSave = normalizedLink.error ? lastSavedLink : normalizedLink.value;
    if (trimmed === lastSavedText && linkToSave === lastSavedLink) return;

    await edit(task.id, trimmed, linkToSave);
    lastSavedText = trimmed;
    lastSavedLink = linkToSave;
  }

  async function finishEdit(forceClose = false) {
    if (finishingEdit) return;
    if (!editText.trim() && !forceClose) return;
    finishingEdit = true;
    let saveFailed = false;

    scheduleAutosave();
    try {
      await autosave.flush();
    } catch (error) {
      console.warn('[Trace] Could not autosave task edit:', error);
      saveFailed = true;
    } finally {
      finishingEdit = false;
    }

    if (saveFailed) return;
    if (linkError && !forceClose) return;

    unregisterPendingEdit?.();
    unregisterPendingEdit = null;
    editing = false;
    wasEditing = false;
    editText = '';
    editLink = '';
    linkError = null;
    dispatch('editDone', { id: task.id });
  }

  function handleEditKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      void finishEdit();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      void finishEdit();
    }
  }

  function handleEditorFocusout(e: FocusEvent) {
    const nextTarget = e.relatedTarget;
    if (nextTarget instanceof Node && editPanel?.contains(nextTarget)) return;
    void finishEdit();
  }

  async function openTaskLink() {
    if (!task.link) return;
    try {
      await openUrl(task.link);
    } catch (error) {
      console.warn('[Trace] Could not open task link:', error);
    }
  }

  // Trigger edit mode when parent sets editing=true
  $: if (editing && !wasEditing) {
    startEdit();
  } else if (!editing && wasEditing) {
    void finishEdit(true);
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
      <div class="edit-panel" bind:this={editPanel} on:focusout={handleEditorFocusout}>
        <input
          class="edit-input"
          type="text"
          bind:value={editText}
          bind:this={editInput}
          on:input={scheduleAutosave}
          on:keydown={handleEditKeydown}
          aria-label="Task"
        />
        <div class="link-input-row" class:invalid={linkError !== null}>
          <svg viewBox="0 0 18 18" aria-hidden="true">
            <path d="m7.2 10.8 3.6-3.6M6 12l-1.1 1.1a2.1 2.1 0 0 1-3-3L5 7a2.1 2.1 0 0 1 3 0M12 6l1.1-1.1a2.1 2.1 0 0 1 3 3L13 11a2.1 2.1 0 0 1-3 0" />
          </svg>
          <input
            class="link-input"
            type="url"
            bind:value={editLink}
            on:input={() => {
              linkError = null;
              scheduleAutosave();
            }}
            on:keydown={handleEditKeydown}
            placeholder="Link (optional)"
            aria-label="Link (optional)"
            aria-invalid={linkError !== null}
          />
        </div>
        {#if linkError}
          <span class="link-error">{linkError}</span>
        {/if}
      </div>
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

  {#if !editing && task.link}
    <button
      class="task-link"
      title={task.link}
      aria-label={`Open link for ${task.text}`}
      on:click|stopPropagation={openTaskLink}
      on:dblclick|stopPropagation
    >
      <svg viewBox="0 0 18 18" aria-hidden="true">
        <path d="m7.2 10.8 3.6-3.6M6 12l-1.1 1.1a2.1 2.1 0 0 1-3-3L5 7a2.1 2.1 0 0 1 3 0M12 6l1.1-1.1a2.1 2.1 0 0 1 3 3L13 11a2.1 2.1 0 0 1-3 0" />
      </svg>
    </button>
  {/if}

  {#if !editing && priorityLevel > 0}
    <div class="priority-stars" aria-label="Priority {priorityLevel}">
      {#each Array(Math.min(priorityLevel, 5)) as _}
        <svg class="star" viewBox="0 0 12 12" aria-hidden="true">
          <path d="m6 1 1.4 3 3.3.4-2.4 2.3.6 3.3L6 8.4 3.1 10l.6-3.3-2.4-2.3L4.6 4z" />
        </svg>
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

  .task-link {
    width: 22px;
    height: 22px;
    flex: 0 0 22px;
    display: grid;
    place-items: center;
    padding: 0;
    margin-top: 0;
    color: var(--on-text-quiet);
    opacity: 0.58;
    border-radius: var(--on-radius-sm);
    transition: opacity var(--on-duration-fast) var(--on-ease), color var(--on-duration-fast) var(--on-ease), background var(--on-duration-fast) var(--on-ease);
  }

  .task-link:hover,
  .task-link:focus-visible {
    opacity: 1;
    color: var(--on-accent);
    background: var(--on-accent-subtle);
  }

  .task-link svg,
  .link-input-row svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.25;
    stroke-linecap: round;
    stroke-linejoin: round;
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
    width: 9px;
    height: 9px;
    fill: currentColor;
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

  .edit-panel {
    width: 100%;
    display: grid;
    gap: var(--on-space-1);
  }

  .link-input-row {
    display: flex;
    align-items: center;
    gap: var(--on-space-1);
    color: var(--on-text-quiet);
    background: var(--on-surface-inset);
    padding: 0 var(--on-space-2);
    border: 1px solid var(--on-hairline);
    border-radius: var(--on-radius-sm);
  }

  .link-input-row:focus-within {
    color: var(--on-accent);
    border-color: var(--on-accent);
  }

  .link-input-row.invalid {
    border-color: var(--on-priority-5);
  }

  .link-input {
    min-width: 0;
    flex: 1;
    padding: var(--on-space-1) 0;
    font-family: var(--on-font-mono);
    font-size: 11px;
    color: var(--on-text-secondary);
  }

  .link-error {
    font-family: var(--on-font-mono);
    font-size: 9px;
    color: var(--on-priority-5);
  }
</style>
