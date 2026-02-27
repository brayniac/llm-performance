<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  import BenchmarkPicker from './BenchmarkPicker.svelte';
  import PerformanceFilters from './PerformanceFilters.svelte';
  import GroupedGridTable from './GroupedGridTable.svelte';
  import HardwareFilters from './HardwareFilters.svelte';
  import SelectionHeader from './SelectionHeader.svelte';

  let models = [];
  let loading = true;
  let expandedModels = new Set();
  let selectedConfigs = [];

  // Filter and sort values
  let selectedBenchmark = 'mmlu'; // Default to MMLU Pro
  let minQuality = 0;
  let minSpeed = 0;
  let sortBy = 'quality'; // Default to quality sorting
  let sortDirection = 'desc';
  let selectedHardwareCategories = []; // All categories selected by default

  async function loadData() {
    loading = true;
    const params = new URLSearchParams({
      benchmark: selectedBenchmark,
      ...(minQuality > 0 && { min_quality: minQuality }),
      ...(minSpeed > 0 && { min_speed: minSpeed }),
      sort_by: sortBy,
      sort_direction: sortDirection
    });

    // Add hardware categories as comma-separated string
    if (selectedHardwareCategories.length > 0) {
      params.set('hardware_categories', selectedHardwareCategories.join(','));
    }

    try {
      const response = await fetch(`/api/grouped-performance?${params}`);
      const data = await response.json();
      models = data.models;
    } catch (error) {
      console.error('Failed to load grouped performance data:', error);
      models = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });

  // Reload when filters change
  function handleFiltersChanged() {
    loadData();
  }

  // Handle benchmark selection change
  function handleBenchmarkChange(event) {
    selectedBenchmark = event.detail.benchmark;
    loadData();
  }

  // Handle hardware filter change
  function handleHardwareFilterChange(event) {
    selectedHardwareCategories = event.detail.selectedCategories;
    loadData();
  }

  // Handle sort change
  function handleSortChange(newSortBy) {
    if (sortBy === newSortBy) {
      sortDirection = sortDirection === 'desc' ? 'asc' : 'desc';
    } else {
      sortBy = newSortBy;
      sortDirection = newSortBy === 'memory' ? 'asc' : 'desc'; // Memory ascending by default
    }
    loadData();
  }

  // Toggle model expansion
  function toggleModel(modelName) {
    if (expandedModels.has(modelName)) {
      expandedModels.delete(modelName);
    } else {
      expandedModels.add(modelName);
    }
    expandedModels = expandedModels; // Trigger reactivity
  }

  // View details for a specific quantization
  function viewDetails(id) {
    goto(`/detail/${id}`);
  }

  // Handle selection changes
  function handleSelectionChanged(configId) {
    if (selectedConfigs.includes(configId)) {
      selectedConfigs = selectedConfigs.filter(id => id !== configId);
    } else if (selectedConfigs.length < 2) {
      selectedConfigs = [...selectedConfigs, configId];
    }
  }

  // Handle comparison
  function handleCompare(event) {
    const { selectedConfigs: configs } = event.detail;
    if (configs.length === 2) {
      goto(`/compare/${configs[0]}/${configs[1]}`);
    }
  }

  // Get sort indicator
  function getSortIndicator(field) {
    if (sortBy === field) {
      return sortDirection === 'desc' ? '▼' : '▲';
    }
    return '';
  }
</script>

<div class="grouped-performance-grid">
  <div class="controls">
    {#if false}
      <BenchmarkPicker
        bind:selectedBenchmark
        on:change={handleBenchmarkChange}
      />
    {/if}

    <HardwareFilters
      bind:selectedCategories={selectedHardwareCategories}
      on:change={handleHardwareFilterChange}
    />

    <div class="filters">
      {#if selectedBenchmark !== 'none'}
        <div class="filter-group">
          <label>
            MMLU Score ≥
            <input
              type="number"
              bind:value={minQuality}
              on:change={handleFiltersChanged}
              min="0"
              max="100"
              step="5"
            />%
          </label>
        </div>
      {/if}

      <div class="filter-group">
        <label>
          Speed ≥
          <input
            type="number"
            bind:value={minSpeed}
            on:change={handleFiltersChanged}
            min="0"
            max="500"
            step="10"
          /> tok/s
        </label>
      </div>
    </div>
  </div>

  {#if loading}
    <div class="loading">Loading performance data...</div>
  {:else if models.length === 0}
    <div class="no-results">No models match the current filters</div>
  {:else}
    {#if selectedConfigs.length > 0}
      <SelectionHeader
        {selectedConfigs}
        on:compare={handleCompare}
      />
    {/if}

    <div class="grid-container">
      <div class="grid-header" class:no-benchmark={selectedBenchmark === 'none'}>
        <div class="checkbox-column"></div>
        <div
          class="sortable"
          on:click={() => handleSortChange('model_name')}
        >
          Model {getSortIndicator('model_name')}
        </div>
        <div>Best Quantization</div>
        {#if selectedBenchmark !== 'none'}
          <div
            class="sortable"
            on:click={() => handleSortChange('quality')}
          >
            MMLU Score {getSortIndicator('quality')}
          </div>
        {/if}
        <div
          class="sortable"
          on:click={() => handleSortChange('speed')}
        >
          Speed {getSortIndicator('speed')}
        </div>
        <div
          class="sortable"
          on:click={() => handleSortChange('efficiency')}
        >
          Efficiency {getSortIndicator('efficiency')}
        </div>
        <div>Backend</div>
        <div>Platform</div>
        <div>Actions</div>
      </div>

      {#each models as model}
        <GroupedGridTable
          {model}
          benchmark={selectedBenchmark}
          expanded={expandedModels.has(model.model_name)}
          {selectedConfigs}
          on:toggle={() => toggleModel(model.model_name)}
          on:viewDetails={(e) => viewDetails(e.detail.id)}
          on:selectionChanged={(e) => handleSelectionChanged(e.detail.configId)}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .grouped-performance-grid {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
  }

  .controls {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 2rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
  }

  .filters {
    display: flex;
    gap: 2rem;
    margin-top: 1rem;
  }

  .filter-group label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .filter-group input {
    width: 80px;
    padding: 0.4rem;
    border: 1px solid var(--color-input-border);
    border-radius: 4px;
    background: var(--color-input-bg);
    color: var(--color-text-primary);
  }

  .loading, .no-results {
    text-align: center;
    padding: 3rem;
    color: var(--color-text-tertiary);
    font-size: 1.1rem;
  }

  .grid-container {
    background: var(--color-bg-primary);
    border-radius: 8px;
    overflow: hidden;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
  }

  .grid-header {
    display: grid;
    grid-template-columns: 40px 2.5fr 1.5fr 1fr 1fr 1.2fr 1fr 1fr 1fr;
    gap: 1rem;
    padding: 1rem;
    background-color: var(--color-grid-header-bg);
    color: var(--color-grid-header-text);
    font-weight: bold;
  }

  .grid-header.no-benchmark {
    grid-template-columns: 40px 2.5fr 1.5fr 1fr 1.2fr 1fr 1fr 1fr;
  }

  .checkbox-column {
    width: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sortable {
    cursor: pointer;
    user-select: none;
  }

  .sortable:hover {
    text-decoration: underline;
  }
</style>
