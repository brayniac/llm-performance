<script>
  import { createEventDispatcher } from 'svelte';

  export let model;
  export let benchmark;
  export let expanded = false;
  export let selectedConfigs = [];

  const dispatch = createEventDispatcher();

  // Check if the best hardware config is selected
  $: isSelected = selectedConfigs.includes(model.best_hardware.best_config.id);

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

  function viewDetails(id, loraAdapter) {
    dispatch('viewDetails', { id, loraAdapter: loraAdapter || '' });
  }

  function handleCheckboxChange() {
    dispatch('selectionChanged', { configId: model.best_hardware.best_config.id });
  }

  function getScoreColor(score) {
    if (score >= 70) return 'var(--color-success)';
    if (score >= 50) return 'var(--color-warning)';
    return 'var(--color-danger)';
  }

  function extractGpuModel(hardwareString) {
    // Extract GPU model from hardware string like "RTX 4090 / Zen2"
    const parts = hardwareString.split(' / ');
    if (parts.length > 0) {
      const gpu = parts[0];

      // If it's CPU Only, extract meaningful info from CPU part
      if (gpu === 'CPU Only' && parts.length > 1) {
        const cpuInfo = parts[1];
        // Extract model name from CPU string - e.g., "AMD EPYC 4564P 16-Core Processor"
        if (cpuInfo.includes('EPYC')) {
          const match = cpuInfo.match(/EPYC\s+(\w+)/);
          return match ? `EPYC ${match[1]}` : 'EPYC';
        } else if (cpuInfo.includes('Xeon')) {
          const match = cpuInfo.match(/Xeon\s+(\w+)/);
          return match ? `Xeon ${match[1]}` : 'Xeon';
        } else if (cpuInfo.includes('Ryzen')) {
          const match = cpuInfo.match(/Ryzen\s+(\d+\s+\w+)/);
          return match ? `Ryzen ${match[1]}` : 'Ryzen';
        } else if (cpuInfo.includes('Core')) {
          const match = cpuInfo.match(/Core\s+(i\d+[-\w]+)/);
          return match ? match[1] : 'Intel Core';
        }
        // Fallback - just show CPU
        return 'CPU';
      }

      // Shorten common GPU prefixes
      return gpu
        .replace('NVIDIA GeForce ', '')
        .replace('NVIDIA ', '')
        .replace('AMD Radeon ', '')
        .trim();
    }
    return hardwareString;
  }
</script>

<div class="model-row" class:expanded class:selected={isSelected} style="grid-template-columns: {benchmark === 'none' ? '40px 2.5fr 1.5fr 1fr 1.2fr 1fr 1fr 1fr' : '40px 2.5fr 1.5fr 1fr 1fr 1.2fr 1fr 1fr 1fr'}">
  <div class="checkbox-column">
    <input
      type="checkbox"
      checked={isSelected}
      disabled={!isSelected && selectedConfigs.length >= 2}
      on:change={handleCheckboxChange}
    />
  </div>
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
        {model.qualifying_platforms} of {model.total_hardware_platforms} hardware platforms
      </div>
      {#if model.best_hardware.best_config.concurrent_requests || model.best_hardware.best_config.max_context_length || model.best_hardware.best_config.load_pattern || model.best_hardware.best_config.gpu_power_limit_watts || model.best_hardware.best_config.dataset_name}
        <div class="config-badges">
          {#if model.best_hardware.best_config.load_pattern && model.best_hardware.best_config.concurrent_requests}
            <span class="badge" title="Load Pattern">{model.best_hardware.best_config.load_pattern} ({model.best_hardware.best_config.concurrent_requests})</span>
          {:else if model.best_hardware.best_config.concurrent_requests}
            <span class="badge" title="Concurrent Requests">{model.best_hardware.best_config.concurrent_requests} requests</span>
          {:else if model.best_hardware.best_config.load_pattern}
            <span class="badge" title="Load Pattern">{model.best_hardware.best_config.load_pattern}</span>
          {/if}
          {#if model.best_hardware.best_config.max_context_length}
            <span class="badge" title="Max Context Length">{model.best_hardware.best_config.max_context_length} ctx</span>
          {/if}
          {#if model.best_hardware.best_config.dataset_name}
            <span class="badge" title="Dataset">{model.best_hardware.best_config.dataset_name}</span>
          {/if}
          {#if model.best_hardware.best_config.gpu_power_limit_watts}
            <span class="badge" title="GPU Power Limit">{model.best_hardware.best_config.gpu_power_limit_watts}W</span>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <div class="quantization">
    {model.best_hardware.best_config.quantization}
    {#if model.best_hardware.best_config.lora_adapter}
      <span class="lora-badge" title="LoRA: {model.best_hardware.best_config.lora_adapter}">{model.best_hardware.best_config.lora_adapter}</span>
    {/if}
  </div>

  {#if benchmark !== 'none'}
    <div class="score" style="color: {model.best_hardware.best_config.quality_score === 0 ? 'var(--color-text-tertiary)' : getScoreColor(model.best_hardware.best_config.quality_score)}">
      {model.best_hardware.best_config.quality_score === 0 ? 'Unknown' : model.best_hardware.best_config.quality_score.toFixed(1) + '%'}
    </div>
  {/if}

  <div class="speed" class:unknown={model.best_hardware.best_config.tokens_per_second === 0}>
    {model.best_hardware.best_config.tokens_per_second === 0 ? 'Unknown' : model.best_hardware.best_config.tokens_per_second.toFixed(1) + ' tok/s'}
  </div>

  <div class="efficiency" class:unknown={!model.best_hardware.best_config.tokens_per_kwh}>
    {model.best_hardware.best_config.tokens_per_kwh ? (model.best_hardware.best_config.tokens_per_kwh / 1000000).toFixed(2) + 'M/kWh' : 'N/A'}
  </div>

  <div class="backend">
    {model.best_hardware.best_config.backend}
  </div>

  <div class="platform" title="{model.best_hardware.hardware}">
    {extractGpuModel(model.best_hardware.hardware)}
  </div>

  <div class="actions">
    <button
      class="detail-btn"
      on:click={() => viewDetails(model.best_hardware.best_config.id, model.best_hardware.best_config.lora_adapter)}
    >
      Details
    </button>
  </div>
</div>

{#if expanded && model.all_hardware_platforms}
  <div class="expanded-content">
    <div class="quant-header" class:no-benchmark={benchmark === 'none'}>
      <div>Hardware Platform</div>
      <div>Quantization</div>
      {#if benchmark !== 'none'}
        <div>MMLU Score</div>
      {/if}
      <div>Speed</div>
      <div>Backend</div>
      <div>Actions</div>
    </div>

    {#each model.all_hardware_platforms as platform}
      <div class="quant-row" class:no-benchmark={benchmark === 'none'}>
        <div>
          <div class="platform-name" title="{platform.hardware}">
            {extractGpuModel(platform.hardware)}
          </div>
          {#if platform.total_configs > 1}
            <div class="config-count">{platform.total_configs} configs tested</div>
          {/if}
        </div>
        <div>
          <div>
            {platform.best_config.quantization}
            {#if platform.best_config.lora_adapter}
              <span class="lora-badge" title="LoRA: {platform.best_config.lora_adapter}">{platform.best_config.lora_adapter}</span>
            {/if}
          </div>
          {#if platform.best_config.concurrent_requests || platform.best_config.max_context_length || platform.best_config.load_pattern || platform.best_config.gpu_power_limit_watts || platform.best_config.dataset_name}
            <div class="config-badges" style="margin-top: 4px;">
              {#if platform.best_config.load_pattern && platform.best_config.concurrent_requests}
                <span class="badge" title="Load Pattern">{platform.best_config.load_pattern} ({platform.best_config.concurrent_requests})</span>
              {:else if platform.best_config.concurrent_requests}
                <span class="badge" title="Concurrent Requests">{platform.best_config.concurrent_requests} requests</span>
              {:else if platform.best_config.load_pattern}
                <span class="badge" title="Load Pattern">{platform.best_config.load_pattern}</span>
              {/if}
              {#if platform.best_config.max_context_length}
                <span class="badge" title="Max Context Length">{platform.best_config.max_context_length} ctx</span>
              {/if}
              {#if platform.best_config.dataset_name}
                <span class="badge" title="Dataset">{platform.best_config.dataset_name}</span>
              {/if}
              {#if platform.best_config.gpu_power_limit_watts}
                <span class="badge" title="GPU Power Limit">{platform.best_config.gpu_power_limit_watts}W</span>
              {/if}
            </div>
          {/if}
        </div>
        {#if benchmark !== 'none'}
          <div style="color: {platform.best_config.quality_score === 0 ? 'var(--color-text-tertiary)' : getScoreColor(platform.best_config.quality_score)}">
            {platform.best_config.quality_score === 0 ? 'Unknown' : platform.best_config.quality_score.toFixed(1) + '%'}
          </div>
        {/if}
        <div class:unknown={platform.best_config.tokens_per_second === 0}>
          {platform.best_config.tokens_per_second === 0 ? 'Unknown' : platform.best_config.tokens_per_second.toFixed(1) + ' tok/s'}
        </div>
        <div class="backend">{platform.best_config.backend}</div>
        <div>
          <button
            class="detail-btn small"
            on:click={() => viewDetails(platform.best_config.id, platform.best_config.lora_adapter)}
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
    grid-template-columns: 40px 2.5fr 1.5fr 1fr 1fr 1fr 1fr 1fr;
    gap: 1rem;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border-primary);
    transition: background-color 0.2s;
  }

  .model-row:hover {
    background-color: var(--color-bg-hover);
  }

  .model-row.expanded {
    background-color: var(--color-bg-expanded);
    border-left: 4px solid var(--color-accent);
  }

  .model-row.selected {
    background-color: var(--color-bg-selected);
  }

  .checkbox-column {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .checkbox-column input[type="checkbox"] {
    cursor: pointer;
    width: 18px;
    height: 18px;
  }

  .checkbox-column input[type="checkbox"]:disabled {
    cursor: not-allowed;
    opacity: 0.5;
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
    color: var(--color-text-tertiary);
  }

  .model-name {
    font-weight: 600;
    color: var(--color-text-primary);
    font-size: 1rem;
  }

  .model-slug {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    margin-top: 2px;
  }

  .quant-count {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
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
    border: 1px solid var(--color-accent);
    color: var(--color-accent);
    padding: 0.4rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .detail-btn:hover {
    background: var(--color-accent);
    color: var(--color-text-inverted);
  }

  .detail-btn.small {
    padding: 0.3rem 0.6rem;
    font-size: 0.8rem;
  }

  .expanded-content {
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border-primary);
    padding: 1rem 1rem 1rem 3rem;
  }

  .quant-header {
    display: grid;
    grid-template-columns: 1.5fr 1fr 1fr 1fr 1fr 1fr;
    gap: 1rem;
    padding: 0.5rem 0;
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--color-text-tertiary);
    border-bottom: 1px solid var(--color-border-primary);
    margin-bottom: 0.5rem;
  }

  .quant-header.no-benchmark {
    grid-template-columns: 1.5fr 1fr 1fr 1fr 1fr;
  }

  .quant-row {
    display: grid;
    grid-template-columns: 1.5fr 1fr 1fr 1fr 1fr 1fr;
    gap: 1rem;
    padding: 0.5rem 0;
    font-size: 0.9rem;
  }

  .quant-row.no-benchmark {
    grid-template-columns: 1.5fr 1fr 1fr 1fr 1fr;
  }

  .platform-name {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .config-count {
    font-size: 0.7rem;
    color: var(--color-text-tertiary);
    margin-top: 2px;
  }

  .quant-row:hover {
    background-color: var(--color-bg-primary);
  }

  .unknown {
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .platform {
    color: var(--color-text-secondary);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .platform.small {
    font-size: 0.9rem;
  }

  .config-badges {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
  }

  .badge {
    display: inline-block;
    padding: 2px 6px;
    font-size: 0.7rem;
    font-weight: 500;
    background-color: var(--color-accent-muted);
    color: var(--color-accent-muted-text);
    border-radius: 3px;
    white-space: nowrap;
  }

  .lora-badge {
    display: inline-block;
    padding: 1px 5px;
    font-size: 0.7rem;
    font-weight: 500;
    background-color: var(--color-warning, #f59e0b);
    color: #fff;
    border-radius: 3px;
    margin-left: 4px;
    vertical-align: middle;
  }
</style>
