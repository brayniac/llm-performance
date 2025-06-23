<script>
  import { createEventDispatcher } from 'svelte';
  
  export let configurations = [];
  export let selectedConfigs = [];
  
  const dispatch = createEventDispatcher();
  
  function toggleSelection(configId) {
    dispatch('selectionChanged', { configId });
  }
  
  function getPerformanceTier(speed) {
    if (speed >= 80) return 'high';
    if (speed >= 40) return 'medium';
    return 'low';
  }
</script>

<div class="grid">
  <div class="grid-row header">
    <div>Model</div>
    <div>Backend</div>
    <div>Speed</div>
    <div>Memory</div>
    <div>Hardware</div>
    <div>Select</div>
  </div>
  
  {#each configurations as config (config.id)}
    <div class="grid-row" class:selected={selectedConfigs.includes(config.id)}>
      <div class="model-info">
        <div class="model-name">{config.model_name}</div>
        <div class="quantization">{config.quantization}</div>
      </div>
      
      <div class="backend">{config.backend}</div>
      
      <div class="speed" data-tier={getPerformanceTier(config.tokens_per_second)}>
        {config.tokens_per_second} tok/s
      </div>
      
      <div class="memory">{config.memory_gb}GB</div>
      
      <div class="hardware">
        <div class="gpu">{config.gpu_model}</div>
        <div class="cpu">{config.cpu_arch}</div>
      </div>
      
      <div class="select">
        <input 
          type="checkbox" 
          checked={selectedConfigs.includes(config.id)}
          on:change={() => toggleSelection(config.id)}
          disabled={!selectedConfigs.includes(config.id) && selectedConfigs.length >= 2}
        />
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
    grid-template-columns: 2fr 1fr 1fr 1fr 1.5fr 0.5fr;
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
  }
  
  .model-info .quantization {
    font-size: 0.875rem;
    color: #6c757d;
    font-family: monospace;
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
</style>