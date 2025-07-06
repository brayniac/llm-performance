<script>
  import { createEventDispatcher } from 'svelte';
  import { goto } from '$app/navigation';
  
  export let configurations = [];
  export let selectedConfigs = [];
  
  const dispatch = createEventDispatcher();
  
  function toggleSelection(configId) {
    dispatch('selectionChanged', { configId });
  }
  
  function viewDetails(configId) {
    goto(`/detail/${configId}`);
  }
  
  function getPerformanceTier(speed) {
    if (speed >= 80) return 'high';
    if (speed >= 40) return 'medium';
    return 'low';
  }
  
  function formatBackendName(backend) {
    return backend;
  }
  
  function getShortModelName(fullName) {
    // Extract the model name part after the last '/'
    const parts = fullName.split('/');
    const modelPart = parts[parts.length - 1];
    
    // Clean up common patterns
    return modelPart
      .replace(/-v([0-9.]+)$/, ' v$1') // Replace -v1 with ' v1'
      .replace(/-([0-9]+B)/, ' $1') // Replace -8B with ' 8B'
      .replace(/-/g, ' ') // Replace remaining dashes with spaces
      .trim();
  }
  
  function formatMemory(memoryGb) {
    return memoryGb.toFixed(1);
  }
  
  function formatSpeed(speed) {
    return speed.toFixed(1);
  }
</script>

<div class="grid">
  <div class="grid-row header">
    <div>Model</div>
    <div>Backend</div>
    <div>Speed</div>
    <div>Memory</div>
    <div>Hardware</div>
    <div>Actions</div>
  </div>
  
  {#each configurations as config (config.id)}
    <div class="grid-row" class:selected={selectedConfigs.includes(config.id)}>
      <div class="model-info">
        <div class="model-name">{getShortModelName(config.model_name)}</div>
        <div class="model-slug">{config.model_name}</div>
        <div class="quantization">{config.quantization}</div>
      </div>
      
      <div class="backend">{formatBackendName(config.backend)}</div>
      
      <div class="speed" data-tier={getPerformanceTier(config.tokens_per_second)}>
        {formatSpeed(config.tokens_per_second)} tok/s
      </div>
      
      <div class="memory">{formatMemory(config.memory_gb)} GB</div>
      
      <div class="hardware">
        <div class="gpu">{config.gpu_model}</div>
        <div class="cpu">{config.cpu_arch}</div>
      </div>
      
      <div class="actions">
        <button on:click={() => viewDetails(config.id)} class="detail-btn" title="View Details">
          Details
        </button>
        <label class="checkbox-label">
          <input 
            type="checkbox" 
            checked={selectedConfigs.includes(config.id)}
            on:change={() => toggleSelection(config.id)}
            disabled={!selectedConfigs.includes(config.id) && selectedConfigs.length >= 2}
          />
          <span class="sr-only">Select {config.model_name} for comparison</span>
        </label>
      </div>
    </div>
  {/each}
</div>

<style>
  .grid {
    border: 1px solid #e1e5e9;
    border-radius: 8px;
    overflow: hidden;
  }
  
  .grid-row {
    display: grid;
    grid-template-columns: 2.5fr 1fr 1fr 0.8fr 1.5fr 0.8fr;
    gap: 1rem;
    padding: 1rem;
    border-bottom: 1px solid #e1e5e9;
    transition: background-color 0.2s;
  }
  
  .grid-row:hover {
    background-color: #f8f9fa;
  }
  
  .grid-row.header {
    background-color: #6c757d;
    color: white;
    font-weight: bold;
  }
  
  .grid-row.selected {
    background-color: #e3f2fd;
    border-left: 4px solid #2196f3;
  }
  
  .model-info .model-name {
    font-weight: 600;
    color: #2c3e50;
    font-size: 1rem;
  }
  
  .model-info .model-slug {
    font-size: 0.75rem;
    color: #6c757d;
    margin-top: 2px;
  }
  
  .model-info .quantization {
    font-size: 0.875rem;
    color: #6c757d;
    font-family: monospace;
    margin-top: 2px;
  }
  
  .speed[data-tier="high"] { 
    color: #28a745; 
    font-weight: 600; 
  }
  
  .speed[data-tier="medium"] { 
    color: #ffc107; 
    font-weight: 600; 
  }
  
  .speed[data-tier="low"] { 
    color: #dc3545; 
  }
  
  .hardware .gpu {
    font-weight: 500;
  }
  
  .hardware .cpu {
    font-size: 0.875rem;
    color: #6c757d;
  }
  
  .actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-content: center;
  }
  
  .detail-btn {
    background: none;
    border: 1px solid #2196f3;
    color: #2196f3;
    padding: 0.4rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    transition: all 0.2s;
    white-space: nowrap;
  }
  
  .detail-btn:hover {
    background: #2196f3;
    color: white;
  }
  
  .checkbox-label {
    display: flex;
    align-items: center;
    cursor: pointer;
  }
  
  .checkbox-label input[type="checkbox"] {
    margin: 0;
  }
  
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>