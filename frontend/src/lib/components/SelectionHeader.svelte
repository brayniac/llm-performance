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
    border-bottom: 1px solid #e1e5e9;
    margin-bottom: 1rem;
  }
  
  .selection-count {
    color: #6c757d;
  }
  
  .compare-btn {
    background: #2196f3;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s;
  }
  
  .compare-btn:hover {
    background: #1976d2;
  }
</style>