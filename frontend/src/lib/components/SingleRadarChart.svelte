<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import { getChartColors } from '$lib/chartTheme.js';
  import { theme } from '$lib/theme.js';

  export let categories;
  export let configName;

  let chartContainer;
  let chart;

  onMount(() => {
    chart = echarts.init(chartContainer);

    return () => {
      if (chart) {
        chart.dispose();
      }
    };
  });

  $: if (chart && categories && categories.length > 0 && $theme) {
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
        data: [configName],
        bottom: 10,
        textStyle: { color: c.text }
      },
      radar: {
        indicator: categories.map(cat => ({
          name: cat.name,
          max: 100
        })),
        center: ['50%', '55%'],
        radius: 140,
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
        data: [{
          value: categories.map(cat => cat.score),
          name: configName,
          lineStyle: { color: c.accent, width: 3 },
          areaStyle: { opacity: 0.2, color: c.accent },
          symbol: 'circle',
          symbolSize: 6
        }]
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
    height: 450px;
  }
</style>
