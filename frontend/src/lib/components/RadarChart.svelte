<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import { getChartColors } from '$lib/chartTheme.js';
  import { theme } from '$lib/theme.js';

  export let comparisonData;

  let chartContainer;
  let chart;

  onMount(() => {
    chart = echarts.init(chartContainer);

    // Cleanup on component destroy
    return () => {
      if (chart) {
        chart.dispose();
      }
    };
  });

  // Update chart when data or theme changes
  $: if (chart && comparisonData && $theme) {
    updateChart();
  }

  function updateChart() {
    const c = getChartColors();
    const option = {
      title: {
        text: 'MMLU-Pro Category Scores',
        left: 'center',
        textStyle: {
          fontSize: 18,
          fontWeight: 'bold',
          color: c.title
        }
      },
      legend: {
        data: [comparisonData.config_a.name, comparisonData.config_b.name],
        bottom: 10,
        textStyle: { color: c.text }
      },
      radar: {
        indicator: comparisonData.categories.map(cat => ({
          name: cat.name,
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
        data: [
          {
            value: comparisonData.categories.map(cat => cat.score_a),
            name: comparisonData.config_a.name,
            lineStyle: { color: c.seriesA, width: 3 },
            areaStyle: { opacity: 0.1, color: c.seriesA },
            itemStyle: { color: c.seriesA }
          },
          {
            value: comparisonData.categories.map(cat => cat.score_b),
            name: comparisonData.config_b.name,
            lineStyle: { color: c.seriesB, width: 3 },
            areaStyle: { opacity: 0.1, color: c.seriesB },
            itemStyle: { color: c.seriesB }
          }
        ]
      }]
    };

    chart.setOption(option);
  }
</script>

<div class="chart-section">
  <div bind:this={chartContainer} class="chart"></div>
</div>

<style>
  .chart-section {
    background: var(--color-bg-primary);
    border-radius: 8px;
    padding: 1rem;
    box-shadow: var(--shadow-md);
    border: 1px solid var(--color-border-primary);
  }

  .chart {
    width: 100%;
    height: 400px;
  }
</style>
