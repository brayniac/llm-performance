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

    return () => {
      if (chart) {
        chart.dispose();
      }
    };
  });

  $: if (chart && comparisonData && $theme) {
    updateChart();
  }

  function updateChart() {
    const c = getChartColors();
    const option = {
      title: {
        text: 'Performance Comparison',
        left: 'center',
        textStyle: {
          fontSize: 18,
          fontWeight: 'bold',
          color: c.title
        }
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' }
      },
      legend: {
        data: [comparisonData.config_a.name, comparisonData.config_b.name],
        bottom: 10,
        textStyle: { color: c.text }
      },
      xAxis: {
        type: 'category',
        data: ['Generation Speed (tok/s)', 'Prompt Processing (tok/s)'],
        axisLabel: { color: c.text },
        axisLine: { lineStyle: { color: c.grid } }
      },
      yAxis: {
        type: 'value',
        axisLabel: { color: c.text },
        axisLine: { lineStyle: { color: c.grid } },
        splitLine: { lineStyle: { color: c.grid } }
      },
      series: [
        {
          name: comparisonData.config_a.name,
          type: 'bar',
          data: [
            comparisonData.config_a.performance.speed,
            comparisonData.config_a.performance.prompt_speed
          ],
          itemStyle: { color: c.seriesA }
        },
        {
          name: comparisonData.config_b.name,
          type: 'bar',
          data: [
            comparisonData.config_b.performance.speed,
            comparisonData.config_b.performance.prompt_speed
          ],
          itemStyle: { color: c.seriesB }
        }
      ]
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
