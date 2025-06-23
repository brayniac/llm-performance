<script>
  import { createEventDispatcher } from 'svelte';
  
  export let maxMemory = 32;
  export let minSpeed = 0;
  export let selectedBackends = ['llama_cpp', 'vllm'];
  export let selectedHardware = ['all'];
  
  const dispatch = createEventDispatcher();
  
  // When any filter changes, notify parent component
  $: dispatch('filtersChanged', {
    maxMemory,
    minSpeed,
    selectedBackends,
    selectedHardware
  });
</script>

<div class="filters">
  <h3>Performance Constraints</h3>
  
  <div class="filter-group">
    <label>
      Max Memory: {maxMemory}GB
      <input type="range" bind:value={maxMemory} min="8" max="64" step="4" />
    </label>
  </div>
  
  <div class="filter-group">
    <label>
      Min Speed: {minSpeed} tok/s
      <input type="range" bind:value={minSpeed} min="0" max="200" step="10" />
    </label>
  </div>
  
  <div class="filter-group">
    <label>Backend:</label>
    <div class="checkbox-group">
      <label>
        <input type="checkbox" bind:group={selectedBackends} value="llama_cpp" /> 
        llama.cpp
      </label>
      <label>
        <input type="checkbox" bind:group={selectedBackends} value="vllm" /> 
        vLLM
      </label>
    </div>
  </div>
  
  <div class="filter-group">
    <label>Hardware:</label>
    <div class="checkbox-group">
      <label>
        <input type="checkbox" bind:group={selectedHardware} value="all" /> 
        All
      </label>
      <label>
        <input type="checkbox" bind:group={selectedHardware} value="optimized_vm" /> 
        Optimized VM
      </label>
      <label>
        <input type="checkbox" bind:group={selectedHardware} value="bare_metal" /> 
        Bare Metal
      </label>
    </div>
  </div>
</div>

<style>
  .filters {
    background: #f8f9fa;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }
  
  .filters h3 {
    margin: 0 0 1rem 0;
    color: #2c3e50;
  }
  
  .filter-group {
    margin-bottom: 1rem;
  }
  
  .filter-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #495057;
  }
  
  .checkbox-group {
    display: flex;
    gap: 1rem;
    margin-top: 0.5rem;
  }
  
  .checkbox-group label {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-weight: normal;
  }
  
  input[type="range"] {
    width: 100%;
    margin-top: 0.5rem;
  }
  
  input[type="checkbox"] {
    margin: 0;
  }
</style>