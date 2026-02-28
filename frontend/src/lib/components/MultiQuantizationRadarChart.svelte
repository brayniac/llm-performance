<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import { getChartColors } from '$lib/chartTheme.js';
  import { theme } from '$lib/theme.js';

  export let analysisData;
  export let lora = '';

  let chartContainer;
  let chart;

  onMount(() => {
    if (chartContainer) {
      chart = echarts.init(chartContainer);
    }

    // Cleanup on component destroy
    return () => {
      if (chart) {
        chart.dispose();
      }
    };
  });

  // Update chart when data or theme changes
  $: if (chart && analysisData && analysisData.quantizations && $theme) {
    updateChart();
  }

  function updateChart() {
    const c = getChartColors();

    // Get all unique categories across all quantizations
    const categorySet = new Set();
    analysisData.quantizations.forEach(quant => {
      if (quant.category_scores) {
        Object.keys(quant.category_scores).forEach(cat => categorySet.add(cat));
      }
    });

    const categories = Array.from(categorySet).sort();

    // Filter out quantizations that have no category scores
    const quantsWithScores = analysisData.quantizations.filter(
      quant => quant.category_scores && Object.keys(quant.category_scores).length > 0
    );

    if (quantsWithScores.length === 0) {
      return;
    }

    const option = {
      title: {
        text: lora ? `MMLU Category Scores by Quantization (LoRA: ${lora})` : 'MMLU Category Scores by Quantization',
        left: 'center',
        textStyle: {
          fontSize: 18,
          fontWeight: 'bold',
          color: c.title
        }
      },
      legend: {
        data: quantsWithScores.map(quant => quant.backend ? `${quant.quantization} (${quant.backend})` : quant.quantization),
        bottom: 10,
        type: 'scroll',
        textStyle: { color: c.text }
      },
      tooltip: {
        trigger: 'item',
        formatter: function(params) {
          if (params.componentSubType === 'radar') {
            const quantization = params.seriesName;
            let result = `<strong>${quantization}</strong><br/>`;
            categories.forEach((cat, idx) => {
              const score = params.value[idx];
              result += `${cat.charAt(0).toUpperCase() + cat.slice(1)}: ${score.toFixed(1)}%<br/>`;
            });
            return result;
          }
          return params.seriesName;
        }
      },
      radar: {
        indicator: categories.map(cat => ({
          name: cat.charAt(0).toUpperCase() + cat.slice(1),
          max: 100
        })),
        center: ['50%', '55%'],
        radius: 120,
        nameGap: 15,
        splitNumber: 5,
        axisName: {
          color: c.text
        },
        axisLabel: {
          show: false
        },
        splitLine: {
          lineStyle: {
            color: c.grid
          }
        },
        splitArea: {
          show: false
        }
      },
      series: [{
        type: 'radar',
        data: quantsWithScores.map((quant, idx) => {
          const color = c.palette[idx % c.palette.length];
          return {
            value: categories.map(cat => quant.category_scores[cat] || 0),
            name: quant.backend ? `${quant.quantization} (${quant.backend})` : quant.quantization,
            lineStyle: { color: color, width: 2 },
            areaStyle: { opacity: 0.05, color: color },
            itemStyle: { color: color }
          };
        })
      }]
    };

    chart.setOption(option);
  }
</script>

<div class="chart-section">
  {#if analysisData && analysisData.quantizations.some(q => q.category_scores && Object.keys(q.category_scores).length > 0)}
    <div bind:this={chartContainer} class="chart"></div>
  {:else}
    <div class="no-data">
      <p>No MMLU category data available for these quantizations.</p>
      <p class="hint">Upload benchmark scores to see quality comparisons.</p>
    </div>
  {/if}
</div>

<style>
  .chart-section {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 1rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
    margin-bottom: 2rem;
  }

  .chart {
    width: 100%;
    height: 400px;
  }

  .no-data {
    padding: 3rem 2rem;
    text-align: center;
    color: var(--color-text-tertiary);
  }

  .no-data p {
    margin: 0.5rem 0;
  }

  .no-data .hint {
    font-size: 0.9rem;
    color: var(--color-text-tertiary);
  }
</style>
