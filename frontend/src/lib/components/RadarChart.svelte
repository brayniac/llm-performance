<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  
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
  
  // Update chart when data changes
  $: if (chart && comparisonData) {
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
        data: [comparisonData.config_a.name, comparisonData.config_b.name],
        bottom: 10
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
        axisLabel: {
          show: true,
          formatter: '{value}%'
        }
      },
      series: [{
        type: 'radar',
        data: [
          {
            value: comparisonData.categories.map(cat => cat.score_a),
            name: comparisonData.config_a.name,
            lineStyle: { color: '#5470c6', width: 3 },
            areaStyle: { opacity: 0.1, color: '#5470c6' }
          },
          {
            value: comparisonData.categories.map(cat => cat.score_b),
            name: comparisonData.config_b.name,
            lineStyle: { color: '#ee6666', width: 3 },
            areaStyle: { opacity: 0.1, color: '#ee6666' }
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
    background: white;
    border-radius: 8px;
    padding: 1rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .chart {
    width: 100%;
    height: 400px;
  }
</style>