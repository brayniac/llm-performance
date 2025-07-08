<script>
  import { createEventDispatcher } from 'svelte';
  
  export let model;
  export let benchmark;
  export let expanded = false;
  
  const dispatch = createEventDispatcher();
  
  function getShortModelName(fullName) {
    const parts = fullName.split('/');
    const modelPart = parts[parts.length - 1];
    
    return modelPart
      .replace(/-v([0-9.]+)$/, ' v$1')
      .replace(/-([0-9]+B)/, ' $1')
      .replace(/-/g, ' ')
      .trim();
  }
  
  function toggleExpansion() {
    dispatch('toggle');
  }
  
  function viewDetails(id) {
    dispatch('viewDetails', { id });
  }
  
  function getScoreColor(score) {
    if (score >= 70) return '#28a745';
    if (score >= 50) return '#ffc107';
    return '#dc3545';
  }
</script>

<div class="model-row" class:expanded style="grid-template-columns: {benchmark === 'none' ? '2.5fr 1.5fr 1fr 0.8fr 1fr' : '2.5fr 1.5fr 1fr 1fr 0.8fr 1fr'}">
  <div class="model-info">
    <button 
      class="expand-btn" 
      on:click={toggleExpansion}
      aria-label={expanded ? 'Collapse' : 'Expand'}
    >
      {expanded ? '▼' : '▶'}
    </button>
    <div>
      <div class="model-name">{getShortModelName(model.model_name)}</div>
      <div class="model-slug">{model.model_name}</div>
      <div class="quant-count">
        {model.qualifying_quantizations} of {model.total_quantizations} quants qualify
      </div>
    </div>
  </div>
  
  <div class="quantization">
    {model.best_quantization.quantization}
  </div>
  
  {#if benchmark !== 'none'}
    <div class="score" style="color: {model.best_quantization.quality_score === 0 ? '#6c757d' : getScoreColor(model.best_quantization.quality_score)}">
      {model.best_quantization.quality_score === 0 ? 'Unknown' : model.best_quantization.quality_score.toFixed(1) + '%'}
    </div>
  {/if}
  
  <div class="speed" class:unknown={model.best_quantization.tokens_per_second === 0}>
    {model.best_quantization.tokens_per_second === 0 ? 'Unknown' : model.best_quantization.tokens_per_second.toFixed(1) + ' tok/s'}
  </div>
  
  <div class="memory" class:unknown={model.best_quantization.memory_gb === 0}>
    {model.best_quantization.memory_gb === 0 ? 'Unknown' : model.best_quantization.memory_gb.toFixed(1) + ' GB'}
  </div>
  
  <div class="actions">
    <button 
      class="detail-btn" 
      on:click={() => viewDetails(model.best_quantization.id)}
    >
      Details
    </button>
  </div>
</div>

{#if expanded && model.all_quantizations}
  <div class="expanded-content">
    <div class="quant-header" class:no-benchmark={benchmark === 'none'}>
      <div>Quantization</div>
      {#if benchmark !== 'none'}
        <div>{benchmark.toUpperCase()} Score</div>
      {/if}
      <div>Speed</div>
      <div>Memory</div>
      <div>Actions</div>
    </div>
    
    {#each model.all_quantizations as quant}
      <div class="quant-row" class:no-benchmark={benchmark === 'none'}>
        <div>{quant.quantization}</div>
        {#if benchmark !== 'none'}
          <div style="color: {quant.quality_score === 0 ? '#6c757d' : getScoreColor(quant.quality_score)}">
            {quant.quality_score === 0 ? 'Unknown' : quant.quality_score.toFixed(1) + '%'}
          </div>
        {/if}
        <div class:unknown={quant.tokens_per_second === 0}>{quant.tokens_per_second === 0 ? 'Unknown' : quant.tokens_per_second.toFixed(1) + ' tok/s'}</div>
        <div class:unknown={quant.memory_gb === 0}>{quant.memory_gb === 0 ? 'Unknown' : quant.memory_gb.toFixed(1) + ' GB'}</div>
        <div>
          <button 
            class="detail-btn small" 
            on:click={() => viewDetails(quant.id)}
          >
            Details
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .model-row {
    display: grid;
    grid-template-columns: 2.5fr 1.5fr 1fr 1fr 0.8fr 1fr;
    gap: 1rem;
    padding: 1rem;
    border-bottom: 1px solid #e1e5e9;
    transition: background-color 0.2s;
  }
  
  .model-row:hover {
    background-color: #f8f9fa;
  }
  
  .model-row.expanded {
    background-color: #e3f2fd;
    border-left: 4px solid #2196f3;
  }
  
  .model-info {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }
  
  .expand-btn {
    background: none;
    border: none;
    font-size: 0.8rem;
    cursor: pointer;
    padding: 0;
    width: 20px;
    color: #6c757d;
  }
  
  .model-name {
    font-weight: 600;
    color: #2c3e50;
    font-size: 1rem;
  }
  
  .model-slug {
    font-size: 0.75rem;
    color: #6c757d;
    margin-top: 2px;
  }
  
  .quant-count {
    font-size: 0.75rem;
    color: #6c757d;
    margin-top: 4px;
  }
  
  .quantization {
    font-family: monospace;
    font-weight: 500;
  }
  
  .score {
    font-weight: 600;
  }
  
  .speed, .memory {
    font-variant-numeric: tabular-nums;
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
  }
  
  .detail-btn:hover {
    background: #2196f3;
    color: white;
  }
  
  .detail-btn.small {
    padding: 0.3rem 0.6rem;
    font-size: 0.8rem;
  }
  
  .expanded-content {
    background: #f8f9fa;
    border-bottom: 1px solid #e1e5e9;
    padding: 1rem 1rem 1rem 3rem;
  }
  
  .quant-header {
    display: grid;
    grid-template-columns: 1.5fr 1fr 1fr 0.8fr 1fr;
    gap: 1rem;
    padding: 0.5rem 0;
    font-weight: 600;
    font-size: 0.9rem;
    color: #6c757d;
    border-bottom: 1px solid #e1e5e9;
    margin-bottom: 0.5rem;
  }
  
  .quant-header.no-benchmark {
    grid-template-columns: 1.5fr 1fr 0.8fr 1fr;
  }
  
  .quant-row {
    display: grid;
    grid-template-columns: 1.5fr 1fr 1fr 0.8fr 1fr;
    gap: 1rem;
    padding: 0.5rem 0;
    font-size: 0.9rem;
  }
  
  .quant-row.no-benchmark {
    grid-template-columns: 1.5fr 1fr 0.8fr 1fr;
  }
  
  .quant-row:hover {
    background-color: white;
  }
  
  .unknown {
    color: #6c757d;
    font-style: italic;
  }
</style>