<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  
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
  
  $: if (chart && comparisonData) {
    updateChart();
  }
  
  function updateChart() {
    const option = {
      title: {
        text: 'Performance Comparison',
        left: 'center',
        textStyle: {
          fontSize: 18,
          fontWeight: 'bold'
        }
      },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' }
      },
      legend: {
        data: [comparisonData.config_a.name, comparisonData.config_b.name],
        bottom: 10
      },
      xAxis: {
        type: 'category',
        data: ['Generation Speed (tok/s)', 'Prompt Processing (tok/s)']
      },
      yAxis: {
        type: 'value'
      },
      series: [
        {
          name: comparisonData.config_a.name,
          type: 'bar',
          data: [
            comparisonData.config_a.performance.speed,
            comparisonData.config_a.performance.prompt_speed
          ],
          itemStyle: { color: '#5470c6' }
        },
        {
          name: comparisonData.config_b.name,
          type: 'bar',
          data: [
            comparisonData.config_b.performance.speed,
            comparisonData.config_b.performance.prompt_speed
          ],
          itemStyle: { color: '#ee6666' }
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