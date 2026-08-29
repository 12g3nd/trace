<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { taskCounts, nowTasks } from '$lib/stores';
  import { invoke } from '@tauri-apps/api/core';
  import pinIcon from '../../assets/orbit/pin.svg';

  const dispatch = createEventDispatcher();

  let pinned = false;

  onMount(async () => {
    try {
      pinned = await invoke<boolean>('get_always_on_top');
    } catch {
      // Fallback
    }
  });

  async function togglePin() {
    try {
      pinned = await invoke<boolean>('toggle_always_on_top');
    } catch {
      pinned = !pinned;
    }
  }

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
      class="icon-btn"
      class:active={pinned}
      on:click={togglePin}
      aria-label="Toggle pin always on top"
      title={pinned ? 'Unpin from top' : 'Pin always on top'}
    >
      <img class="pin-icon" src={pinIcon} alt="" />
    </button>
    <button
      class="icon-btn"
      on:click={() => dispatch('openSettings')}
      aria-label="Open settings"
      title="Settings & Commands"
    >
      <svg class="settings-icon" viewBox="0 0 18 18" aria-hidden="true">
        <path d="M4 5.2h10M4 9h10M4 12.8h10M7 3.8v2.8M11.5 7.6v2.8M8.5 11.4v2.8" />
      </svg>
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
    gap: var(--on-space-2);
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
    margin-right: var(--on-space-1);
  }

  .counter-sep {
    margin: 0 1px;
    opacity: 0.5;
  }

  .icon-btn {
    font-size: 11px;
    color: var(--on-text-quiet);
    opacity: 0.6;
    transition: opacity var(--on-duration-fast) var(--on-ease), color var(--on-duration-fast) var(--on-ease);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3px 5px;
    border-radius: var(--on-radius-sm);
  }

  .icon-btn:hover {
    opacity: 1;
    color: var(--on-text);
  }

  .icon-btn.active {
    opacity: 1;
    color: var(--on-accent);
    background: var(--on-accent-subtle);
  }

  .pin-icon,
  .settings-icon {
    width: 13px;
    height: 13px;
  }

  .pin-icon {
    filter: invert(76%) sepia(14%) saturate(415%) hue-rotate(167deg) brightness(93%);
  }

  .icon-btn.active .pin-icon {
    filter: invert(51%) sepia(97%) saturate(3164%) hue-rotate(213deg) brightness(102%);
  }

  .settings-icon {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.25;
    stroke-linecap: round;
  }
</style>
