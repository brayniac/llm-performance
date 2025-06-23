<script>
  import { onMount } from 'svelte';
  
  import ConfigCards from './ConfigCards.svelte';
  import RadarChart from './RadarChart.svelte';
  import PerformanceChart from './PerformanceChart.svelte';
  
  export let configA;
  export let configB;
  
  let comparisonData = null;
  let loading = true;
  let error = null;
  
  onMount(async () => {
    try {
      const response = await fetch(`/api/comparison?config_a=${configA}&config_b=${configB}`);
      if (!response.ok) throw new Error('Failed to load comparison data');
      comparisonData = await response.json();
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  });
  
  function getWinner(scoreA, scoreB) {
    if (Math.abs(scoreA - scoreB) < 1) return 'tie';
    return scoreA > scoreB ? 'a' : 'b';
  }
  
  function getDifference(scoreA, scoreB) {
    return Math.abs(scoreA - scoreB).toFixed(1);
  }
</script>

<div class="comparison-view">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      Loading comparison data...
    </div>
  {:else if error}
    <div class="error">
      Error: {error}
    </div>
  {:else if comparisonData}
    <ConfigCards {comparisonData} />
    
    <div class="charts-grid">
      <RadarChart {comparisonData} />
      <PerformanceChart {comparisonData} />
    </div>
    
    <div class="detailed-scores">
      <h3>Detailed Category Breakdown</h3>
      <div class="scores-table">
        <div class="table-header">
          <div>Category</div>
          <div>Config A</div>
          <div>Config B</div>
          <div>Difference</div>
          <div>Winner</div>
        </div>
        {#each comparisonData.categories as category}
          <div class="table-row">
            <div class="category-name">{category.name}</div>
            <div class="score score-a">{category.score_a.toFixed(1)}%</div>
            <div class="score score-b">{category.score_b.toFixed(1)}%</div>
            <div class="difference">±{getDifference(category.score_a, category.score_b)}%</div>
            <div class="winner" data-winner={getWinner(category.score_a, category.score_b)}>
              {#if getWinner(category.score_a, category.score_b) === 'a'}
                Config A
              {:else if getWinner(category.score_a, category.score_b) === 'b'}
                Config B
              {:else}
                Tie
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </div>
    
    <div class="summary">
      <h3>Summary</h3>
      <div class="summary-grid">
        <div class="summary-card">
          <h4>Overall MMLU-Pro</h4>
          <div class="summary-scores">
            <div class="summary-score">
              <span class="config-name">Config A:</span>
              <span class="score">{comparisonData.config_a.overall_score.toFixed(1)}%</span>
            </div>
            <div class="summary-score">
              <span class="config-name">Config B:</span>
              <span class="score">{comparisonData.config_b.overall_score.toFixed(1)}%</span>
            </div>
          </div>
        </div>
        
        <div class="summary-card">
          <h4>Performance Winner</h4>
          <div class="performance-winner">
            {#if comparisonData.config_a.performance.speed > comparisonData.config_b.performance.speed}
              <div class="winner-info">
                <strong>Config A</strong> is {((comparisonData.config_a.performance.speed / comparisonData.config_b.performance.speed - 1) * 100).toFixed(0)}% faster
              </div>
            {:else}
              <div class="winner-info">
                <strong>Config B</strong> is {((comparisonData.config_b.performance.speed / comparisonData.config_a.performance.speed - 1) * 100).toFixed(0)}% faster
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .comparison-view {
    max-width: 1400px;
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
  
  .charts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    margin-bottom: 3rem;
  }
  
  .detailed-scores {
    background: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    margin-bottom: 2rem;
  }
  
  .detailed-scores h3 {
    margin: 0 0 1.5rem 0;
    color: #2c3e50;
  }
  
  .scores-table {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
    gap: 1rem;
  }
  
  .table-header {
    display: contents;
    font-weight: bold;
    color: #6c757d;
    border-bottom: 2px solid #e1e5e9;
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
  
  .score-a { 
    color: #5470c6; 
    font-weight: 600; 
  }
  
  .score-b { 
    color: #ee6666; 
    font-weight: 600; 
  }
  
  .difference {
    color: #6c757d;
    font-family: monospace;
  }
  
  .winner[data-winner="a"] { 
    color: #5470c6; 
    font-weight: 600; 
  }
  
  .winner[data-winner="b"] { 
    color: #ee6666; 
    font-weight: 600; 
  }
  
  .winner[data-winner="tie"] { 
    color: #6c757d; 
  }
  
  .summary {
    background: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .summary h3 {
    margin: 0 0 1.5rem 0;
    color: #2c3e50;
  }
  
  .summary-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
  }
  
  .summary-card {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 1.5rem;
  }
  
  .summary-card h4 {
    margin: 0 0 1rem 0;
    color: #495057;
    font-size: 1.1rem;
  }
  
  .summary-scores {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .summary-score {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .summary-score .config-name {
    color: #6c757d;
    font-weight: 500;
  }
  
  .summary-score .score {
    font-size: 1.25rem;
    font-weight: bold;
    color: #2c3e50;
  }
  
  .performance-winner {
    font-size: 1.1rem;
    color: #495057;
  }
  
  .winner-info strong {
    color: #28a745;
  }
  
  /* Responsive design */
  @media (max-width: 768px) {
    .comparison-view {
      padding: 1rem;
    }
    
    .charts-grid {
      grid-template-columns: 1fr;
    }
    
    .summary-grid {
      grid-template-columns: 1fr;
    }
    
    .scores-table {
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