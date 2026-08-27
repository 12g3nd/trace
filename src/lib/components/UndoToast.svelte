<script lang="ts">
  import { undoStack, performUndo } from '$lib/stores';
</script>

{#if $undoStack}
  <div class="undo-toast" role="status" aria-live="polite">
    <span class="undo-label">{$undoStack.label}</span>
    <button class="undo-btn" on:click={performUndo}>Undo</button>
  </div>
{/if}

<style>
  .undo-toast {
    position: fixed;
    bottom: 64px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: var(--on-space-3);
    padding: var(--on-space-2) var(--on-space-4);
    background: var(--on-surface);
    border: 1px solid var(--on-hairline-strong);
    border-radius: var(--on-radius-md);
    z-index: 90;
    font-size: 12px;
    color: var(--on-text-secondary);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  }

  .undo-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .undo-btn {
    font-family: var(--on-font-interface);
    font-size: 12px;
    font-weight: 500;
    color: var(--on-accent);
    cursor: pointer;
    white-space: nowrap;
    padding: var(--on-space-1) var(--on-space-2);
    border-radius: var(--on-radius-sm);
    transition: background var(--on-duration-fast) var(--on-ease);
  }

  .undo-btn:hover {
    background: var(--on-accent-subtle);
  }
</style>
