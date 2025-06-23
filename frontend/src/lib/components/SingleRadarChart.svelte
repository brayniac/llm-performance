<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  
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
  
  $: if (chart && categories && categories.length > 0) {
    updateChart();
  }
  
  function updateChart() {
    const option = {
      title: {
        text: 'MMLU-Pro Category Scores',
        left: 'center',
        textStyle: {
          fontSize: 18,
          fontWeight: 'bold'
        }
      },
      legend: {
        data: [configName],
        bottom: 10
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
        axisLabel: {
          show: false  // Hide the percentage labels on spokes
        },
        splitLine: {
          lineStyle: {
            color: '#e0e6ed'
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
          lineStyle: { color: '#2196f3', width: 3 },
          areaStyle: { opacity: 0.2, color: '#2196f3' },
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
    background: white;
    border-radius: 8px;
    padding: 1rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .chart {
    width: 100%;
    height: 450px;
  }
</style>