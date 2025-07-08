<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  
  import BenchmarkPicker from './BenchmarkPicker.svelte';
  import PerformanceFilters from './PerformanceFilters.svelte';
  import GroupedGridTable from './GroupedGridTable.svelte';
  import HardwareFilters from './HardwareFilters.svelte';
  
  let models = [];
  let loading = true;
  let expandedModels = new Set();
  
  // Filter and sort values
  let selectedBenchmark = 'mmlu'; // Default to MMLU Pro
  let minQuality = 0;
  let maxMemory = 100;
  let minSpeed = 0;
  let sortBy = 'quality'; // Default to quality sorting
  let sortDirection = 'desc';
  let selectedHardwareCategories = []; // All categories selected by default
  
  async function loadData() {
    loading = true;
    const params = new URLSearchParams({
      benchmark: selectedBenchmark,
      ...(minQuality > 0 && { min_quality: minQuality }),
      ...(maxMemory < 100 && { max_memory_gb: maxMemory }),
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
      
      <div class="filter-group">
        <label>
          Memory ≤ 
          <input 
            type="number" 
            bind:value={maxMemory} 
            on:change={handleFiltersChanged}
            min="1" 
            max="100" 
            step="1"
          /> GB
        </label>
      </div>
    </div>
  </div>

  {#if loading}
    <div class="loading">Loading performance data...</div>
  {:else if models.length === 0}
    <div class="no-results">No models match the current filters</div>
  {:else}
    <div class="grid-container">
      <div class="grid-header" class:no-benchmark={selectedBenchmark === 'none'}>
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
          on:click={() => handleSortChange('memory')}
        >
          Memory {getSortIndicator('memory')}
        </div>
        <div>Actions</div>
      </div>
      
      {#each models as model}
        <GroupedGridTable 
          {model}
          benchmark={selectedBenchmark}
          expanded={expandedModels.has(model.model_name)}
          on:toggle={() => toggleModel(model.model_name)}
          on:viewDetails={(e) => viewDetails(e.detail.id)}
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
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 2rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
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
  }
  
  .filter-group input {
    width: 80px;
    padding: 0.4rem;
    border: 1px solid #ddd;
    border-radius: 4px;
  }
  
  .loading, .no-results {
    text-align: center;
    padding: 3rem;
    color: #6c757d;
    font-size: 1.1rem;
  }
  
  .grid-container {
    background: white;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .grid-header {
    display: grid;
    grid-template-columns: 2.5fr 1.5fr 1fr 1.5fr 0.8fr 1fr;
    gap: 1rem;
    padding: 1rem;
    background-color: #6c757d;
    color: white;
    font-weight: bold;
  }
  
  .grid-header.no-benchmark {
    grid-template-columns: 2.5fr 1.5fr 1.5fr 0.8fr 1fr;
  }
  
  .sortable {
    cursor: pointer;
    user-select: none;
  }
  
  .sortable:hover {
    text-decoration: underline;
  }
</style>