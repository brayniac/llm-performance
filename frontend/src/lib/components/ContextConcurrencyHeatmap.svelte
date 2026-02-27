<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';
  import { getChartColors } from '$lib/chartTheme.js';
  import { theme } from '$lib/theme.js';

  export let heatmapData;
  export let quantization;
  export let metric = 'speed'; // 'speed', 'ttft', or 'efficiency'
  export let globalMin = null; // Optional: unified min across all quantizations
  export let globalMax = null; // Optional: unified max across all quantizations

  let chartContainer;
  let chart;

  onMount(() => {
    if (!heatmapData || !chartContainer) return;

    chart = echarts.init(chartContainer);
    renderChart();

    return () => {
      if (chart) {
        chart.dispose();
      }
    };
  });

  $: if (chart && heatmapData && $theme) {
    renderChart();
  }

  function renderChart() {
    if (!chart || !heatmapData) return;

    const c = getChartColors();

    const data = metric === 'speed'
      ? heatmapData.speed_data
      : metric === 'ttft'
        ? heatmapData.ttft_data
        : heatmapData.efficiency_data;
    const quantData = data[quantization];

    if (!quantData) {
      console.warn(`No data for quantization: ${quantization}`);
      return;
    }

    // Prepare heatmap data: [concurrency_index, power_limit_index, value]
    const heatmapValues = [];
    const powerLimits = heatmapData.power_limits;
    const concurrentRequests = heatmapData.concurrent_requests;

    concurrentRequests.forEach((concurrent, concurrentIdx) => {
      powerLimits.forEach((powerLimit, powerIdx) => {
        const value = quantData[powerLimit]?.[concurrent];
        if (value !== undefined && value !== null) {
          const displayValue = metric === 'efficiency' ? value / 1000000 : value;
          heatmapValues.push([concurrentIdx, powerIdx, displayValue.toFixed(2)]);
        }
      });
    });

    if (heatmapValues.length === 0) {
      console.warn('No data points to display in heatmap');
      return;
    }

    let visualMapMin, visualMapMax;

    if (globalMin !== null && globalMax !== null) {
      visualMapMin = globalMin;
      visualMapMax = globalMax;
    } else {
      const values = heatmapValues.map(v => parseFloat(v[2]));
      const maxValue = Math.max(...values);
      const minValue = Math.min(...values);

      if (maxValue === minValue) {
        visualMapMin = maxValue * 0.8;
        visualMapMax = maxValue * 1.2;
      } else if (maxValue - minValue < maxValue * 0.1) {
        const avg = (maxValue + minValue) / 2;
        visualMapMin = avg * 0.8;
        visualMapMax = avg * 1.2;
      } else {
        visualMapMin = minValue;
        visualMapMax = maxValue;
      }
    }

    const option = {
      title: {
        text: `${metric === 'speed' ? 'Token/s' : metric === 'ttft' ? 'TTFT (ms)' : 'Million Tokens/kWh'} - ${quantization}`,
        left: 'center',
        textStyle: { color: c.title }
      },
      tooltip: {
        position: 'top',
        formatter: function (params) {
          const concurrent = concurrentRequests[params.data[0]];
          const powerLimit = powerLimits[params.data[1]];
          const value = params.data[2];
          const label = metric === 'speed' ? 'Speed' : metric === 'ttft' ? 'TTFT' : 'Efficiency';
          const unit = metric === 'speed' ? ' tok/s' : metric === 'ttft' ? ' ms' : ' M tok/kWh';
          return `Concurrent: ${concurrent}<br/>Power: ${powerLimit}W<br/>${label}: ${value}${unit}`;
        }
      },
      grid: {
        top: '15%',
        bottom: '15%',
        left: '15%',
        right: '10%'
      },
      xAxis: {
        type: 'category',
        data: concurrentRequests.map(c => `${c} req`),
        name: 'Concurrent Requests',
        nameLocation: 'middle',
        nameGap: 30,
        axisLabel: { color: c.text },
        axisLine: { lineStyle: { color: c.grid } },
        splitArea: {
          show: true
        }
      },
      yAxis: {
        type: 'category',
        data: powerLimits.map(pw => `${pw}W`),
        name: 'GPU Power Limit',
        nameLocation: 'middle',
        nameGap: 50,
        axisLabel: { color: c.text },
        axisLine: { lineStyle: { color: c.grid } },
        splitArea: {
          show: true
        }
      },
      visualMap: {
        show: true,
        min: visualMapMin,
        max: visualMapMax,
        calculable: false,
        realtime: false,
        textStyle: { color: c.text },
        inRange: {
          color: metric === 'ttft'
            ? ['#fde724', '#fdc518', '#fda30c', '#fd8100', '#f96f00', '#f45d00', '#ef4800', '#e63000', '#d91800', '#cc0000']
            : ['#440154', '#482878', '#3e4989', '#31688e', '#26828e', '#1f9e89', '#35b779', '#6ece58', '#b5de2b', '#fde724']
        }
      },
      series: [{
        name: metric === 'speed' ? 'Token/s' : metric === 'ttft' ? 'TTFT' : 'Million Tokens/kWh',
        type: 'heatmap',
        data: heatmapValues,
        label: {
          show: true,
          fontSize: 14,
          fontWeight: 'bold'
        },
        itemStyle: {
          borderWidth: 2,
          borderColor: c.cellBorder
        },
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowColor: 'rgba(0, 0, 0, 0.5)'
          }
        }
      }]
    };

    chart.setOption(option);
  }
</script>

<div class="heatmap-container" bind:this={chartContainer}></div>

<style>
  .heatmap-container {
    width: 100%;
    height: 500px;
  }
</style>
