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
    
    <div class="summary">
      <h3>Comparison Summary</h3>
      <div class="summary-grid">
        <div class="summary-card">
          <h4>Quality Comparison</h4>
          <div class="quality-comparison">
            <h5>MMLU-Pro Overall Score (%)</h5>
            <div class="mini-chart">
              <div class="chart-bar">
                <div class="bar-segment config-a" style="width: {(comparisonData.config_a.overall_score / Math.max(comparisonData.config_a.overall_score, comparisonData.config_b.overall_score)) * 100}%">
                  <span class="bar-label">{comparisonData.config_a.model} {comparisonData.config_a.quantization}</span>
                  <span class="bar-value">{comparisonData.config_a.overall_score.toFixed(1)}%</span>
                </div>
                {#if comparisonData.config_a.overall_score > comparisonData.config_b.overall_score}
                  <span class="chart-trophy">🏆</span>
                {/if}
              </div>
              <div class="chart-bar">
                <div class="bar-segment config-b" style="width: {(comparisonData.config_b.overall_score / Math.max(comparisonData.config_a.overall_score, comparisonData.config_b.overall_score)) * 100}%">
                  <span class="bar-label">{comparisonData.config_b.model} {comparisonData.config_b.quantization}</span>
                  <span class="bar-value">{comparisonData.config_b.overall_score.toFixed(1)}%</span>
                </div>
                {#if comparisonData.config_b.overall_score > comparisonData.config_a.overall_score}
                  <span class="chart-trophy">🏆</span>
                {/if}
              </div>
            </div>
          </div>
        </div>
        
        <div class="summary-card">
          <h4>Performance Comparison</h4>
          <div class="performance-comparison">
            <div class="perf-category">
              <h5>Speed (tok/s)</h5>
              <div class="mini-chart">
                <div class="chart-bar">
                  <div class="bar-segment config-a" style="width: {(comparisonData.config_a.performance.speed / Math.max(comparisonData.config_a.performance.speed, comparisonData.config_b.performance.speed)) * 100}%">
                    <span class="bar-label">{comparisonData.config_a.model} {comparisonData.config_a.quantization}</span>
                    <span class="bar-value">{comparisonData.config_a.performance.speed}</span>
                  </div>
                  {#if comparisonData.config_a.performance.speed > comparisonData.config_b.performance.speed}
                    <span class="chart-trophy">🏆</span>
                  {/if}
                </div>
                <div class="chart-bar">
                  <div class="bar-segment config-b" style="width: {(comparisonData.config_b.performance.speed / Math.max(comparisonData.config_a.performance.speed, comparisonData.config_b.performance.speed)) * 100}%">
                    <span class="bar-label">{comparisonData.config_b.model} {comparisonData.config_b.quantization}</span>
                    <span class="bar-value">{comparisonData.config_b.performance.speed}</span>
                  </div>
                  {#if comparisonData.config_b.performance.speed > comparisonData.config_a.performance.speed}
                    <span class="chart-trophy">🏆</span>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    
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
  
  .summary {
    background: white;
    border-radius: 8px;
    padding: 2rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    margin-bottom: 3rem;
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
  
  .quality-comparison {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .quality-comparison h5 {
    margin: 0 0 0.75rem 0;
    color: #495057;
    font-size: 1rem;
    font-weight: 600;
  }
  
  .performance-comparison {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .perf-category h5 {
    margin: 0 0 0.75rem 0;
    color: #495057;
    font-size: 1rem;
    font-weight: 600;
  }
  
  .mini-chart {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .chart-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-height: 2.5rem;
  }
  
  .bar-segment {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    min-width: 200px;
    transition: all 0.3s ease;
  }
  
  .bar-segment.config-a {
    background: linear-gradient(90deg, #5470c6 0%, rgba(84, 112, 198, 0.8) 100%);
    color: white;
  }
  
  .bar-segment.config-b {
    background: linear-gradient(90deg, #ee6666 0%, rgba(238, 102, 102, 0.8) 100%);
    color: white;
  }
  
  .bar-segment.memory {
    opacity: 0.9; /* Slightly faded for "lower is better" */
  }
  
  .bar-label {
    font-weight: 500;
    font-size: 0.85rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }
  
  .bar-value {
    font-weight: 600;
    font-family: monospace;
    font-size: 0.9rem;
    white-space: nowrap;
  }
  
  .chart-trophy {
    font-size: 1.25rem;
    flex-shrink: 0;
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