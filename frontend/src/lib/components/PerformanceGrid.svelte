<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  
  import PerformanceFilters from './PerformanceFilters.svelte';
  import GridTable from './GridTable.svelte';
  import SelectionHeader from './SelectionHeader.svelte';
  
  let configurations = [];
  let filteredConfigs = [];
  let selectedConfigs = [];
  let loading = true;
  
  // Filter values (these get passed to PerformanceFilters)
  let maxMemory = 32;
  let minSpeed = 0;
  let selectedBackends = ['llama_cpp', 'vllm'];
  let selectedHardware = ['all'];
  
  onMount(async () => {
    const response = await fetch('/api/performance-grid');
    configurations = await response.json();
    loading = false;
  });

/*
  onMount(async () => {
    // Mock data for testing - replace with real API call later
    configurations = [
      {
        id: "mistral-small-3.2-24b-q8_0-zen2-rtx4090",
        model_name: "Mistral Small 3.2 24B",
        quantization: "Q8_0",
        backend: "llama_cpp",
        tokens_per_second: 45.2,
        memory_gb: 18.5,
        gpu_model: "RTX 4090",
        cpu_arch: "Zen2",
        hardware_type: "optimized_vm"
      },
      {
        id: "mistral-small-3.2-24b-q4_0-zen2-rtx4090",
        model_name: "Mistral Small 3.2 24B",
        quantization: "Q4_0",
        backend: "llama_cpp",
        tokens_per_second: 78.1,
        memory_gb: 12.3,
        gpu_model: "RTX 4090",
        cpu_arch: "Zen2",
        hardware_type: "optimized_vm"
      },
      {
        id: "llama-3.1-8b-q8_0-zen1-rtx4090",
        model_name: "Llama 3.1 8B",
        quantization: "Q8_0",
        backend: "llama_cpp",
        tokens_per_second: 92.5,
        memory_gb: 8.7,
        gpu_model: "RTX 4090",
        cpu_arch: "Zen1",
        hardware_type: "bare_metal"
      }
    ];
    loading = false;
  });
*/
  
  // When filters change, update the filtered list
  function handleFiltersChanged(event) {
    const filters = event.detail;
    
    filteredConfigs = configurations.filter(config => {
      return config.memory_gb <= filters.maxMemory &&
             config.tokens_per_second >= filters.minSpeed &&
             filters.selectedBackends.includes(config.backend) &&
             (filters.selectedHardware.includes('all') || 
              filters.selectedHardware.includes(config.hardware_type));
    });
  }
  
  // When selection changes in the grid
  function handleSelectionChanged(event) {
    const { configId } = event.detail;
    
    if (selectedConfigs.includes(configId)) {
      selectedConfigs = selectedConfigs.filter(id => id !== configId);
    } else if (selectedConfigs.length < 2) {
      selectedConfigs = [...selectedConfigs, configId];
    }
  }
  
  // When user clicks compare button
  function handleCompare(event) {
    const { selectedConfigs: configs } = event.detail;
    if (configs.length === 2) {
      goto(`/compare/${configs[0]}/${configs[1]}`);
    }
  }
  
  // Run initial filter when data loads
  $: if (configurations.length > 0) {
    handleFiltersChanged({ 
      detail: { maxMemory, minSpeed, selectedBackends, selectedHardware }
    });
  }
</script>

<div class="performance-grid">
  <PerformanceFilters 
    bind:maxMemory
    bind:minSpeed
    bind:selectedBackends
    bind:selectedHardware
    on:filtersChanged={handleFiltersChanged}
  />

  {#if loading}
    <div class="loading">Loading performance data...</div>
  {:else}
    <div class="grid-container">
      <SelectionHeader 
        {selectedConfigs}
        on:compare={handleCompare}
      />
      
      <GridTable 
        configurations={filteredConfigs}
        {selectedConfigs}
        on:selectionChanged={handleSelectionChanged}
      />
    </div>
  {/if}
</div>

<style>
  .performance-grid {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .loading {
    text-align: center;
    padding: 3rem;
    color: #6c757d;
    font-size: 1.1rem;
  }
  
  .grid-container {
    background: white;
    border-radius: 8px;
    padding: 1rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
</style>