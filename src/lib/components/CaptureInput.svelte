<script lang="ts">
  import { capture } from '$lib/stores';

  let inputValue = '';
  let inputEl: HTMLInputElement;

  export function focus() {
    inputEl?.focus();
  }

  async function handleSubmit() {
    const text = inputValue.trim();
    if (!text) return;

    await capture(text);
    inputValue = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === 'Escape') {
      inputValue = '';
      inputEl?.blur();
    }
  }
</script>

<div class="capture">
  <span class="capture-icon" aria-hidden="true">+</span>
  <input
    bind:this={inputEl}
    bind:value={inputValue}
    type="text"
    class="capture-input"
    placeholder="type anything..."
    on:keydown={handleKeydown}
    aria-label="Add a task"
  />
</div>

<style>
  .capture {
    display: flex;
    align-items: center;
    gap: var(--on-space-2);
  }

  .capture-icon {
    font-family: var(--on-font-interface);
    font-size: 16px;
    font-weight: 400;
    color: var(--on-text-quiet);
    flex-shrink: 0;
    width: 20px;
    text-align: center;
    transition: color var(--on-duration-fast) var(--on-ease);
  }

  .capture:focus-within .capture-icon {
    color: var(--on-accent);
  }

  .capture-input {
    flex: 1;
    font-family: var(--on-font-interface);
    font-size: 14px;
    color: var(--on-text);
    background: transparent;
    padding: var(--on-space-1) 0;
    border: none;
    outline: none;
  }

  .capture-input::placeholder {
    color: var(--on-text-quiet);
    font-style: normal;
  }
</style>
