<script>
  import { createEventDispatcher } from 'svelte';

  export let selectedConfigs = [];

  const dispatch = createEventDispatcher();

  function handleCompare() {
    dispatch('compare', { selectedConfigs });
  }
</script>

<div class="selection-info">
  <div class="selection-count">
    {selectedConfigs.length}/2 selected for comparison
  </div>

  {#if selectedConfigs.length === 2}
    <button on:click={handleCompare} class="compare-btn">
      Compare Quality →
    </button>
  {/if}
</div>

<style>
  .selection-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 0;
    font-weight: 500;
    border-bottom: 1px solid var(--color-border-primary);
    margin-bottom: 1rem;
  }

  .selection-count {
    color: var(--color-text-tertiary);
  }

  .compare-btn {
    background: var(--color-accent);
    color: var(--color-text-inverted);
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .compare-btn:hover {
    background: var(--color-accent-hover);
  }
</style>
