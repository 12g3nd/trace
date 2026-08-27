<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { initDb } from '$lib/db';
  import { refresh } from '$lib/stores';

  let ready = false;

  onMount(async () => {
    await initDb();
    await refresh();
    ready = true;
  });
</script>

{#if ready}
  <slot />
{/if}
