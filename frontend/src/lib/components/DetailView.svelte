<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import SingleRadarChart from './SingleRadarChart.svelte';
  import MultiQuantizationRadarChart from './MultiQuantizationRadarChart.svelte';
  import ContextConcurrencyHeatmap from './ContextConcurrencyHeatmap.svelte';

  export let configId;

  let detailData = null;
  let analysisData = null;
  let loading = true;
  let error = null;

  // Unified scales for heatmaps (calculated across all quantizations)
  let speedGlobalMin = null;
  let speedGlobalMax = null;
  let ttftGlobalMin = null;
  let ttftGlobalMax = null;
  let tpotGlobalMin = null;
  let tpotGlobalMax = null;
  let itlGlobalMin = null;
  let itlGlobalMax = null;
  let efficiencyGlobalMin = null;
  let efficiencyGlobalMax = null;

  // Calculate global min/max for unified heatmap scales
  $: if (analysisData && analysisData.heatmap_data) {
    const speedData = analysisData.heatmap_data.speed_data;
    const ttftData = analysisData.heatmap_data.ttft_data;
    const tpotData = analysisData.heatmap_data.tpot_data;
    const itlData = analysisData.heatmap_data.itl_data;
    const efficiencyData = analysisData.heatmap_data.efficiency_data;

    let allSpeedValues = [];
    let allTtftValues = [];
    let allTpotValues = [];
    let allItlValues = [];
    let allEfficiencyValues = [];

    // Collect all values across all quantizations
    Object.values(speedData).forEach(quantData => {
      Object.values(quantData).forEach(powerLimitData => {
        Object.values(powerLimitData).forEach(value => {
          if (value !== undefined && value !== null) {
            allSpeedValues.push(value);
          }
        });
      });
    });

    Object.values(ttftData).forEach(quantData => {
      Object.values(quantData).forEach(powerLimitData => {
        Object.values(powerLimitData).forEach(value => {
          if (value !== undefined && value !== null) {
            allTtftValues.push(value);
          }
        });
      });
    });

    if (tpotData) {
      Object.values(tpotData).forEach(quantData => {
        Object.values(quantData).forEach(powerLimitData => {
          Object.values(powerLimitData).forEach(value => {
            if (value !== undefined && value !== null) {
              allTpotValues.push(value);
            }
          });
        });
      });
    }

    if (itlData) {
      Object.values(itlData).forEach(quantData => {
        Object.values(quantData).forEach(powerLimitData => {
          Object.values(powerLimitData).forEach(value => {
            if (value !== undefined && value !== null) {
              allItlValues.push(value);
            }
          });
        });
      });
    }

    if (efficiencyData) {
      Object.values(efficiencyData).forEach(quantData => {
        Object.values(quantData).forEach(powerLimitData => {
          Object.values(powerLimitData).forEach(value => {
            if (value !== undefined && value !== null) {
              allEfficiencyValues.push(value);
            }
          });
        });
      });
    }

    // Calculate global min/max for speed
    if (allSpeedValues.length > 0) {
      speedGlobalMin = Math.min(...allSpeedValues);
      speedGlobalMax = Math.max(...allSpeedValues);
    }

    // Calculate global min/max for ttft
    if (allTtftValues.length > 0) {
      ttftGlobalMin = Math.min(...allTtftValues);
      ttftGlobalMax = Math.max(...allTtftValues);
    }

    // Calculate global min/max for tpot
    if (allTpotValues.length > 0) {
      tpotGlobalMin = Math.min(...allTpotValues);
      tpotGlobalMax = Math.max(...allTpotValues);
    }

    // Calculate global min/max for itl
    if (allItlValues.length > 0) {
      itlGlobalMin = Math.min(...allItlValues);
      itlGlobalMax = Math.max(...allItlValues);
    }

    // Calculate global min/max for efficiency
    if (allEfficiencyValues.length > 0) {
      efficiencyGlobalMin = Math.min(...allEfficiencyValues);
      efficiencyGlobalMax = Math.max(...allEfficiencyValues);
    }
  }

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

      // Now fetch model+hardware analysis data
      const modelName = encodeURIComponent(detailData.config.model);
      const gpuModel = encodeURIComponent(detailData.system_info.gpu_model);

      console.log('Fetching analysis data for:', modelName, gpuModel);
      const analysisResponse = await fetch(`/api/model-hardware-analysis/${modelName}/${gpuModel}`);

      if (analysisResponse.ok) {
        analysisData = await analysisResponse.json();
        console.log('Analysis data loaded:', analysisData);
      } else {
        console.warn('Could not load analysis data, continuing with single config view');
      }
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
    if (score >= 97) return 'a-plus';
    if (score >= 93) return 'a';
    if (score >= 90) return 'a-minus';
    if (score >= 87) return 'b-plus';
    if (score >= 83) return 'b';
    if (score >= 80) return 'b-minus';
    if (score >= 77) return 'c-plus';
    if (score >= 73) return 'c';
    if (score >= 70) return 'c-minus';
    if (score >= 67) return 'd-plus';
    if (score >= 63) return 'd';
    if (score >= 60) return 'd-minus';
    return 'f';
  }

  function getGradeLabel(score) {
    if (score >= 97) return 'A+';
    if (score >= 93) return 'A';
    if (score >= 90) return 'A-';
    if (score >= 87) return 'B+';
    if (score >= 83) return 'B';
    if (score >= 80) return 'B-';
    if (score >= 77) return 'C+';
    if (score >= 73) return 'C';
    if (score >= 70) return 'C-';
    if (score >= 67) return 'D+';
    if (score >= 63) return 'D';
    if (score >= 60) return 'D-';
    return 'F';
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
  {:else if detailData && analysisData}
    <!-- Model + Hardware Analysis View -->
    <div class="header">
      <div class="config-header">
        <h1 class="config-title">{analysisData.model_name}</h1>
        <div class="config-subtitle">
          {analysisData.gpu_model}
        </div>
        <div class="test-date">
          {analysisData.total_configurations} configuration{analysisData.total_configurations !== 1 ? 's' : ''} tested
        </div>
      </div>
    </div>

    <!-- System Configuration Card -->
    <div class="system-config-section">
      <h2>System Configuration</h2>
      <div class="system-details-grid">
        <div class="detail-item">
          <span class="label">GPU:</span>
          <span class="value">{detailData.system_info.gpu_model} ({detailData.system_info.gpu_memory_gb}GB)</span>
        </div>
        <div class="detail-item">
          <span class="label">CPU:</span>
          <span class="value">{detailData.system_info.cpu_model} ({detailData.system_info.cpu_arch})</span>
        </div>
        <div class="detail-item">
          <span class="label">RAM:</span>
          <span class="value">{detailData.system_info.ram_gb}GB {detailData.system_info.ram_type}</span>
        </div>
        <div class="detail-item">
          <span class="label">Backend:</span>
          <span class="value">{detailData.config.backend} {detailData.config.backend_version}</span>
        </div>
      </div>
    </div>

    <!-- Quantization Comparison -->
    <div class="quantization-comparison">
          <h3>Quantization Comparison</h3>
          {#each analysisData.backends as backendGroup}
            <div class="backend-section">
              <h4 class="backend-header">{backendGroup.backend}</h4>
              <div class="quant-grid">
                {#each backendGroup.quantizations as quant}
                  <div class="quant-card">
                    <div class="quant-header">{quant.quantization}</div>
                    <div class="quant-stats">
                      <div class="stat">
                        <span class="stat-label">Best Speed</span>
                        <span class="stat-value">{quant.best_speed.toFixed(1)} tok/s</span>
                      </div>
                      {#if quant.best_ttft}
                        <div class="stat">
                          <span class="stat-label">Best TTFT</span>
                          <span class="stat-value">{quant.best_ttft.toFixed(2)} ms</span>
                        </div>
                      {/if}
                      {#if quant.best_tokens_per_kwh}
                        <div class="stat">
                          <span class="stat-label">Best Efficiency</span>
                          <span class="stat-value">{(quant.best_tokens_per_kwh / 1000000).toFixed(2)}M tok/kWh</span>
                        </div>
                      {/if}
                      <div class="stat">
                        <span class="stat-label">Quality Score</span>
                        <span class="stat-value">{quant.quality_score.toFixed(1)}%</span>
                      </div>
                      <div class="stat">
                        <span class="stat-label">Configs Tested</span>
                        <span class="stat-value">{quant.configuration_count}</span>
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/each}
        </div>

        <!-- Multi-Quantization Radar Chart -->
        <MultiQuantizationRadarChart analysisData={analysisData} />

        <!-- Power Limit x Concurrency Heatmaps -->
        <div class="heatmaps-section">
          <h3>Performance Heatmaps</h3>
          <p class="heatmap-description">Explore how performance varies with GPU power limit and concurrency levels</p>

          {#each analysisData.backends as backendGroup}
            <div class="backend-section">
              <h4 class="backend-header">{backendGroup.backend}</h4>
              {#each backendGroup.quantizations as quant}
                {@const compositeKey = `${backendGroup.backend}||${quant.quantization}`}
                <div class="heatmap-group">
                  <h4>{quant.quantization} Performance</h4>
                  <div class="heatmap-triple">
                    <div class="heatmap-wrapper">
                      <ContextConcurrencyHeatmap
                        heatmapData={analysisData.heatmap_data}
                        quantization={compositeKey}
                        displayLabel={quant.quantization}
                        metric="speed"
                        globalMin={speedGlobalMin}
                        globalMax={speedGlobalMax}
                      />
                    </div>
                    <div class="heatmap-wrapper">
                      <ContextConcurrencyHeatmap
                        heatmapData={analysisData.heatmap_data}
                        quantization={compositeKey}
                        displayLabel={quant.quantization}
                        metric="ttft"
                        globalMin={ttftGlobalMin}
                        globalMax={ttftGlobalMax}
                      />
                    </div>
                    {#if analysisData.heatmap_data.tpot_data}
                      <div class="heatmap-wrapper">
                        <ContextConcurrencyHeatmap
                          heatmapData={analysisData.heatmap_data}
                          quantization={compositeKey}
                          displayLabel={quant.quantization}
                          metric="tpot"
                          globalMin={tpotGlobalMin}
                          globalMax={tpotGlobalMax}
                        />
                      </div>
                    {/if}
                    {#if analysisData.heatmap_data.itl_data}
                      <div class="heatmap-wrapper">
                        <ContextConcurrencyHeatmap
                          heatmapData={analysisData.heatmap_data}
                          quantization={compositeKey}
                          displayLabel={quant.quantization}
                          metric="itl"
                          globalMin={itlGlobalMin}
                          globalMax={itlGlobalMax}
                        />
                      </div>
                    {/if}
                    {#if analysisData.heatmap_data.efficiency_data}
                      <div class="heatmap-wrapper">
                        <ContextConcurrencyHeatmap
                          heatmapData={analysisData.heatmap_data}
                          quantization={compositeKey}
                          displayLabel={quant.quantization}
                          metric="efficiency"
                          globalMin={efficiencyGlobalMin}
                          globalMax={efficiencyGlobalMax}
                        />
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/each}
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
    color: var(--color-text-tertiary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid var(--color-spinner-track);
    border-top: 4px solid var(--color-spinner-active);
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
    color: var(--color-danger);
    background: var(--color-error-bg);
    border-radius: 8px;
    border: 1px solid var(--color-error-border);
  }

  .header {
    margin-bottom: 2rem;
  }

  .config-header {
    text-align: center;
    padding: 2rem;
    background: var(--color-bg-hero);
    color: #fafafa;
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
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
    margin-bottom: 3rem;
  }

  .score-card, .performance-card, .system-card {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 2rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
  }

  .score-card h3, .performance-card h3, .system-card h3 {
    margin: 0 0 1.5rem 0;
    color: var(--color-text-primary);
    font-size: 1.1rem;
  }

  .overall-score {
    font-size: 3rem;
    font-weight: bold;
    text-align: center;
    margin-bottom: 0.5rem;
  }

  .overall-score[data-tier="a-plus"],
  .overall-score[data-tier="a"],
  .overall-score[data-tier="a-minus"] { color: var(--color-success); }
  .overall-score[data-tier="b-plus"],
  .overall-score[data-tier="b"],
  .overall-score[data-tier="b-minus"] { color: var(--color-info); }
  .overall-score[data-tier="c-plus"],
  .overall-score[data-tier="c"],
  .overall-score[data-tier="c-minus"] { color: var(--color-warning); }
  .overall-score[data-tier="d-plus"],
  .overall-score[data-tier="d"],
  .overall-score[data-tier="d-minus"] { color: var(--color-orange); }
  .overall-score[data-tier="f"] { color: var(--color-danger); }

  .metrics-grid {
    display: grid;
    gap: 1rem;
  }

  .metric {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background: var(--color-bg-secondary);
    border-radius: 6px;
  }

  .metric-label {
    color: var(--color-text-tertiary);
    font-weight: 500;
  }

  .metric-value {
    font-weight: 600;
    color: var(--color-text-primary);
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
    border-bottom: 1px solid var(--color-border-secondary);
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-row .label {
    font-weight: 500;
    color: var(--color-text-tertiary);
    min-width: 100px;
  }

  .detail-row .value {
    color: var(--color-text-primary);
    text-align: right;
    font-family: monospace;
    font-size: 0.9rem;
  }

  .chart-section {
    margin-bottom: 3rem;
  }

  .categories-table {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 2rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
  }

  .categories-table h3 {
    margin: 0 0 1.5rem 0;
    color: var(--color-text-primary);
  }

  .table-container {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr;
    gap: 1rem;
  }

  .table-header {
    display: contents;
    font-weight: bold;
    color: var(--color-text-tertiary);
  }

  .table-header > div {
    padding-bottom: 0.75rem;
    border-bottom: 2px solid var(--color-border-primary);
  }

  .table-row {
    display: contents;
  }

  .table-row > div {
    padding: 0.75rem 0;
    border-bottom: 1px solid var(--color-border-secondary);
  }

  .category-name {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .category-score {
    font-weight: 600;
  }

  .category-score[data-tier="a-plus"],
  .category-score[data-tier="a"],
  .category-score[data-tier="a-minus"] { color: var(--color-success); }
  .category-score[data-tier="b-plus"],
  .category-score[data-tier="b"],
  .category-score[data-tier="b-minus"] { color: var(--color-info); }
  .category-score[data-tier="c-plus"],
  .category-score[data-tier="c"],
  .category-score[data-tier="c-minus"] { color: var(--color-warning); }
  .category-score[data-tier="d-plus"],
  .category-score[data-tier="d"],
  .category-score[data-tier="d-minus"] { color: var(--color-orange); }
  .category-score[data-tier="f"] { color: var(--color-danger); }


  .category-performance {
    font-weight: 500;
    font-size: 0.9rem;
  }

  .category-performance[data-tier="a-plus"],
  .category-performance[data-tier="a"],
  .category-performance[data-tier="a-minus"] { color: var(--color-success); }
  .category-performance[data-tier="b-plus"],
  .category-performance[data-tier="b"],
  .category-performance[data-tier="b-minus"] { color: var(--color-info); }
  .category-performance[data-tier="c-plus"],
  .category-performance[data-tier="c"],
  .category-performance[data-tier="c-minus"] { color: var(--color-warning); }
  .category-performance[data-tier="d-plus"],
  .category-performance[data-tier="d"],
  .category-performance[data-tier="d-minus"] { color: var(--color-orange); }
  .category-performance[data-tier="f"] { color: var(--color-danger); }

  /* System Configuration Section */
  .system-config-section {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 2rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
    margin-bottom: 3rem;
  }

  .system-config-section h2 {
    margin: 0 0 1.5rem 0;
    color: var(--color-text-primary);
    font-size: 1.5rem;
  }

  .system-details-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .detail-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .detail-item .label {
    font-weight: 600;
    color: var(--color-text-tertiary);
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .detail-item .value {
    color: var(--color-text-primary);
    font-family: monospace;
    font-size: 0.95rem;
  }

  /* Quantization Comparison Section */
  .quantization-comparison {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 2rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
    margin-bottom: 3rem;
  }

  .quantization-comparison h3 {
    margin: 0 0 1.5rem 0;
    color: var(--color-text-primary);
  }

  .quant-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .quant-card {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    padding: 1.5rem;
    border: 2px solid var(--color-border-primary);
    transition: all 0.2s;
  }

  .quant-card:hover {
    border-color: var(--color-accent);
    box-shadow: var(--shadow-lg);
  }

  .quant-header {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 1rem;
    font-family: monospace;
  }

  .quant-stats {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .stat {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .stat-label {
    color: var(--color-text-tertiary);
    font-size: 0.9rem;
  }

  .stat-value {
    font-weight: 600;
    color: var(--color-text-primary);
    font-family: monospace;
  }

  .heatmaps-section {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 2rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
  }

  .heatmaps-section h3 {
    margin: 0 0 0.5rem 0;
    color: var(--color-text-primary);
  }

  .backend-section {
    margin-bottom: 2rem;
  }

  .backend-header {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 1rem 0;
    padding: 0.5rem 0.75rem;
    border-left: 3px solid var(--color-accent);
    background: var(--color-bg-secondary);
    border-radius: 0 6px 6px 0;
  }

  .heatmap-description {
    color: var(--color-text-tertiary);
    margin-bottom: 2rem;
  }

  .heatmap-group {
    margin-bottom: 3rem;
  }

  .heatmap-group:last-child {
    margin-bottom: 0;
  }

  .heatmap-group h4 {
    color: var(--color-text-primary);
    font-size: 1.25rem;
    margin: 0 0 1rem 0;
    padding: 0.75rem;
    background: var(--color-bg-secondary);
    border-radius: 6px;
    font-family: monospace;
  }

  .heatmap-pair {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
    gap: 2rem;
  }

  .heatmap-triple {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 2rem;
  }

  .heatmap-wrapper {
    background: var(--color-bg-secondary);
    border-radius: 8px;
    padding: 1rem;
    border: 1px solid var(--color-border-primary);
  }

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
      border-bottom: 1px solid var(--color-border-secondary);
    }

    .quant-grid {
      grid-template-columns: 1fr;
    }

    .heatmap-pair,
    .heatmap-triple {
      grid-template-columns: 1fr;
    }
  }
</style>
