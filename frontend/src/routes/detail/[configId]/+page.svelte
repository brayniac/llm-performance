<script>
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import DetailView from '$lib/components/DetailView.svelte';
  
  $: configId = $page.params.configId;
  
  function goBack() {
    goto('/');
  }
</script>

<svelte:head>
  <title>Details: {configId} - LLM Benchmarks</title>
  <meta name="description" content="Detailed view of {configId} performance and quality metrics" />
</svelte:head>

<div class="detail-page">
  <div class="breadcrumb">
    <button on:click={goBack} class="back-btn">
      ← Back to Performance Grid
    </button>
  </div>
  
  <DetailView {configId} />
</div>

<style>
  .detail-page {
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