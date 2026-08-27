<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { taskCounts, nowTasks } from '$lib/stores';

  const dispatch = createEventDispatcher();

  const now = new Date();
  const days = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
  const months = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
  const dateStr = `${days[now.getDay()]} ${months[now.getMonth()]} ${now.getDate()}`;
</script>

<header class="header">
  <span class="date">{dateStr}</span>
  <div class="header-right">
    <span class="counter">
      <span class="counter-done">{$taskCounts.now - $nowTasks.filter(t => t.status === 'done').length}</span>
      <span class="counter-sep">/</span>
      <span class="counter-total">{$taskCounts.now + $taskCounts.later + $taskCounts.someday}</span>
    </span>
    <button
      class="settings-btn"
      on:click={() => dispatch('openSettings')}
      aria-label="Open settings"
      title="Settings & Commands"
    >
      ⚙
    </button>
  </div>
</header>

<style>
  .header {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--on-space-4) var(--on-space-4) var(--on-space-2);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: var(--on-space-3);
  }

  .date {
    font-family: var(--on-font-graphic);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--on-text-secondary);
  }

  .counter {
    font-family: var(--on-font-mono);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    color: var(--on-text-quiet);
  }

  .counter-sep {
    margin: 0 1px;
    opacity: 0.5;
  }

  .settings-btn {
    font-size: 12px;
    color: var(--on-text-quiet);
    opacity: 0.7;
    transition: opacity var(--on-duration-fast) var(--on-ease), color var(--on-duration-fast) var(--on-ease);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border-radius: var(--on-radius-sm);
  }

  .settings-btn:hover {
    opacity: 1;
    color: var(--on-text);
  }
</style>
