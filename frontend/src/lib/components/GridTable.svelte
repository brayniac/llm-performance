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
    const parts = fullName.split('/');
    const modelPart = parts[parts.length - 1];

    return modelPart
      .replace(/-v([0-9.]+)$/, ' v$1')
      .replace(/-([0-9]+B)/, ' $1')
      .replace(/-/g, ' ')
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
    border: 1px solid var(--color-border-primary);
    border-radius: 8px;
    overflow: hidden;
  }

  .grid-row {
    display: grid;
    grid-template-columns: 2.5fr 1fr 1fr 0.8fr 1.5fr 0.8fr;
    gap: 1rem;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border-primary);
    transition: background-color 0.2s;
  }

  .grid-row:hover {
    background-color: var(--color-bg-hover);
  }

  .grid-row.header {
    background-color: var(--color-grid-header-bg);
    color: var(--color-grid-header-text);
    font-weight: bold;
  }

  .grid-row.selected {
    background-color: var(--color-bg-selected);
    border-left: 4px solid var(--color-accent);
  }

  .model-info .model-name {
    font-weight: 600;
    color: var(--color-text-primary);
    font-size: 1rem;
  }

  .model-info .model-slug {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    margin-top: 2px;
  }

  .model-info .quantization {
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
    font-family: monospace;
    margin-top: 2px;
  }

  .speed[data-tier="high"] {
    color: var(--color-success);
    font-weight: 600;
  }

  .speed[data-tier="medium"] {
    color: var(--color-warning);
    font-weight: 600;
  }

  .speed[data-tier="low"] {
    color: var(--color-danger);
  }

  .hardware .gpu {
    font-weight: 500;
  }

  .hardware .cpu {
    font-size: 0.875rem;
    color: var(--color-text-tertiary);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-content: center;
  }

  .detail-btn {
    background: none;
    border: 1px solid var(--color-accent);
    color: var(--color-accent);
    padding: 0.4rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .detail-btn:hover {
    background: var(--color-accent);
    color: var(--color-text-inverted);
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
