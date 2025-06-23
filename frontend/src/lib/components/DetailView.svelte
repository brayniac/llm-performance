<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import SingleRadarChart from './SingleRadarChart.svelte';
  
  export let configId;
  
  let detailData = null;
  let loading = true;
  let error = null;
  
  onMount(async () => {
    try {
      console.log('Fetching detail data for config ID:', configId);
      const response = await fetch(`/api/detail/${configId}`);
      console.log('Response status:', response.status);
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error('Response error:', errorText);
        throw new Error(`Failed to load detail data: ${response.status} - ${errorText}`);
      }
      
      detailData = await response.json();
      console.log('Detail data loaded:', detailData);
    } catch (e) {
      console.error('Error loading detail data:', e);
      error = e.message;
    } finally {
      loading = false;
    }
  });
  
  function formatPercentage(score) {
    return score.toFixed(1);
  }
  
  function getScoreTier(score) {
    if (score >= 80) return 'excellent';
    if (score >= 70) return 'good';
    if (score >= 60) return 'fair';
    return 'poor';
  }
</script>

<div class="detail-view">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      Loading detailed information...
    </div>
  {:else if error}
    <div class="error">
      Error: {error}
    </div>
  {:else if detailData}
    <div class="header">
      <button on:click={() => goto('/')} class="back-btn">
        ← Back to Performance Grid
      </button>
      
      <div class="config-header">
        <h1 class="config-title">{detailData.config.name}</h1>
        <div class="config-subtitle">
          {detailData.system_info.gpu_model} / {detailData.system_info.cpu_arch} / {detailData.config.backend}
        </div>
        <div class="test-date">
          Tested on {detailData.config.test_run_date}
        </div>
      </div>
    </div>
    
    <div class="content-grid">
      <!-- Overall Score Card -->
      <div class="score-card">
        <h3>Overall MMLU-Pro Score</h3>
        <div class="overall-score" data-tier={getScoreTier(detailData.config.overall_score)}>
          {formatPercentage(detailData.config.overall_score)}%
        </div>
        <div class="score-label">
          {#if detailData.config.overall_score >= 80}
            Excellent Performance
          {:else if detailData.config.overall_score >= 70}
            Good Performance
          {:else if detailData.config.overall_score >= 60}
            Fair Performance
          {:else}
            Needs Improvement
          {/if}
        </div>
      </div>
      
      <!-- Performance Metrics Card -->
      <div class="performance-card">
        <h3>Performance Metrics</h3>
        <div class="metrics-grid">
          <div class="metric">
            <div class="metric-label">Speed</div>
            <div class="metric-value">{detailData.config.performance.speed} tok/s</div>
          </div>
          <div class="metric">
            <div class="metric-label">Memory Usage</div>
            <div class="metric-value">{detailData.config.performance.memory} GB</div>
          </div>
          <div class="metric">
            <div class="metric-label">Loading Time</div>
            <div class="metric-value">{detailData.config.performance.loading_time}s</div>
          </div>
        </div>
      </div>
      
      <!-- System Info Card -->
      <div class="system-card">
        <h3>System Configuration</h3>
        <div class="system-details">
          <div class="detail-row">
            <span class="label">GPU:</span>
            <span class="value">{detailData.system_info.gpu_model} ({detailData.system_info.gpu_memory_gb}GB)</span>
          </div>
          <div class="detail-row">
            <span class="label">CPU:</span>
            <span class="value">{detailData.system_info.cpu_model} ({detailData.system_info.cpu_arch})</span>
          </div>
          <div class="detail-row">
            <span class="label">RAM:</span>
            <span class="value">{detailData.system_info.ram_gb}GB {detailData.system_info.ram_type}</span>
          </div>
          <div class="detail-row">
            <span class="label">Backend:</span>
            <span class="value">{detailData.config.backend} {detailData.config.backend_version}</span>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Radar Chart -->
    <div class="chart-section">
      <SingleRadarChart 
        categories={detailData.categories} 
        configName={detailData.config.name}
      />
    </div>
    
    <!-- Category Scores Table -->
    <div class="categories-table">
      <h3>Category Breakdown</h3>
      <div class="table-container">
        <div class="table-header">
          <div>Category</div>
          <div>Score</div>
          <div>Correct/Total</div>
          <div>Performance</div>
        </div>
        {#each detailData.categories as category}
          <div class="table-row">
            <div class="category-name">{category.name}</div>
            <div class="category-score" data-tier={getScoreTier(category.score)}>
              {formatPercentage(category.score)}%
            </div>
            <div class="category-stats">
              {#if category.correct_answers && category.total_questions}
                {category.correct_answers}/{category.total_questions}
              {:else}
                —
              {/if}
            </div>
            <div class="category-performance" data-tier={getScoreTier(category.score)}>
              {#if category.score >= 80}
                Excellent
              {:else if category.score >= 70}
                Good
              {:else if category.score >= 60}
                Fair
              {:else}
                Poor
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .detail-view {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 400px;
    color: #6c757d;
  }
  
  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #f3f3f3;
    border-top: 4px solid #2196f3;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .error {
    text-align: center;
    padding: 2rem;
    color: #dc3545;
    background: #f8d7da;
    border-radius: 8px;
    border: 1px solid #f5c6cb;
  }
  
  .header {
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
    margin-bottom: 1rem;
  }
  
  .back-btn:hover {
    background: #6c757d;
    color: white;
  }
  
  .config-header {
    text-align: center;
    padding: 2rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border-radius: 12px;
  }
  
  .config-title {
    margin: 0 0 0.5rem 0;
    font-size: 2.5rem;
    font-weight: 300;
  }
  
  .config-subtitle {
    font-size: 1.25rem;
    opacity: 0.9;
    margin-bottom: 0.5rem;
  }
  
  .test-date {
    font-size: 0.9rem;
    opacity: 0.8;
  }
  
  .content-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 2rem;
    margin-bottom: 3rem;
  }
  
  .score-card, .performance-card, .system-card {
    background: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .score-card h3, .performance-card h3, .system-card h3 {
    margin: 0 0 1.5rem 0;
    color: #2c3e50;
    font-size: 1.1rem;
  }
  
  .overall-score {
    font-size: 3rem;
    font-weight: bold;
    text-align: center;
    margin-bottom: 0.5rem;
  }
  
  .overall-score[data-tier="excellent"] { color: #28a745; }
  .overall-score[data-tier="good"] { color: #17a2b8; }
  .overall-score[data-tier="fair"] { color: #ffc107; }
  .overall-score[data-tier="poor"] { color: #dc3545; }
  
  .score-label {
    text-align: center;
    color: #6c757d;
    font-weight: 500;
  }
  
  .metrics-grid {
    display: grid;
    gap: 1rem;
  }
  
  .metric {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: #f8f9fa;
    border-radius: 6px;
  }
  
  .metric-label {
    color: #6c757d;
    font-weight: 500;
  }
  
  .metric-value {
    font-weight: 600;
    color: #2c3e50;
  }
  
  .system-details {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid #f1f3f4;
  }
  
  .detail-row:last-child {
    border-bottom: none;
  }
  
  .detail-row .label {
    font-weight: 500;
    color: #6c757d;
    min-width: 100px;
  }
  
  .detail-row .value {
    color: #2c3e50;
    text-align: right;
    font-family: monospace;
    font-size: 0.9rem;
  }
  
  .chart-section {
    margin-bottom: 3rem;
  }
  
  .categories-table {
    background: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .categories-table h3 {
    margin: 0 0 1.5rem 0;
    color: #2c3e50;
  }
  
  .table-container {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr;
    gap: 1rem;
  }
  
  .table-header {
    display: contents;
    font-weight: bold;
    color: #6c757d;
  }
  
  .table-header > div {
    padding-bottom: 0.75rem;
    border-bottom: 2px solid #e1e5e9;
  }
  
  .table-row {
    display: contents;
  }
  
  .table-row > div {
    padding: 0.75rem 0;
    border-bottom: 1px solid #f1f3f4;
  }
  
  .category-name {
    font-weight: 500;
    color: #2c3e50;
  }
  
  .category-score {
    font-weight: 600;
  }
  
  .category-score[data-tier="excellent"] { color: #28a745; }
  .category-score[data-tier="good"] { color: #17a2b8; }
  .category-score[data-tier="fair"] { color: #ffc107; }
  .category-score[data-tier="poor"] { color: #dc3545; }
  
  .category-stats {
    color: #6c757d;
    font-family: monospace;
  }
  
  .category-performance {
    font-weight: 500;
    font-size: 0.9rem;
  }
  
  .category-performance[data-tier="excellent"] { color: #28a745; }
  .category-performance[data-tier="good"] { color: #17a2b8; }
  .category-performance[data-tier="fair"] { color: #ffc107; }
  .category-performance[data-tier="poor"] { color: #dc3545; }
  
  /* Responsive design */
  @media (max-width: 768px) {
    .detail-view {
      padding: 1rem;
    }
    
    .content-grid {
      grid-template-columns: 1fr;
    }
    
    .config-title {
      font-size: 2rem;
    }
    
    .table-container {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }
    
    .table-header,
    .table-row {
      display: block;
    }
    
    .table-header > div,
    .table-row > div {
      padding: 0.5rem;
      border-bottom: 1px solid #f1f3f4;
    }
  }
</style>