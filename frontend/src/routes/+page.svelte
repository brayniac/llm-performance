<script>
  import PerformanceGrid from '$lib/components/PerformanceGrid.svelte';
</script>

<svelte:head>
  <title>LLM Performance Comparison</title>
  <meta name="description" content="Compare LLM model performance across different quantization schemes and hardware configurations" />
</svelte:head>

<div class="home-page">
  <div class="hero">
    <h2>LLM Performance & Quality Comparison</h2>
    <p>Compare models across different quantization schemes, backends, and hardware configurations</p>
  </div>
  
  <PerformanceGrid />
</div>

<style>
  .hero {
    text-align: center;
    padding: 3rem 2rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    margin-bottom: 2rem;
  }
  
  .hero h2 {
    margin: 0 0 1rem 0;
    font-size: 2.5rem;
    font-weight: 300;
  }
  
  .hero p {
    margin: 0;
    font-size: 1.25rem;
    opacity: 0.9;
  }
</style>
