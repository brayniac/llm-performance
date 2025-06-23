<script>
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import ComparisonView from '$lib/components/ComparisonView.svelte';
  
  $: configA = $page.params.configA;
  $: configB = $page.params.configB;
</script>

<svelte:head>
  <title>Compare {configA} vs {configB} - LLM Benchmarks</title>
  <meta name="description" content="Detailed comparison of two LLM configurations across MMLU-Pro categories and performance metrics" />
</svelte:head>

<div class="comparison-page">
  <div class="breadcrumb">
    <button on:click={() => goto('/')} class="back-btn">
      ← Back to Performance Grid
    </button>
  </div>
  
  <ComparisonView {configA} {configB} />
</div>

<style>
  .comparison-page {
    max-width: 1400px;
    margin: 0 auto;
    padding: 0 2rem;
  }
  
  .breadcrumb {
    margin-bottom: 2rem;
  }
  
  .back-btn {
    background: none;
    border: 1px solid #6c757d;
    color: #6c757d;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .back-btn:hover {
    background: #6c757d;
    color: white;
  }
</style>