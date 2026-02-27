<script>
  import { onMount } from 'svelte';
  import * as echarts from 'echarts';

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

  $: if (chart && heatmapData) {
    renderChart();
  }

  function renderChart() {
    if (!chart || !heatmapData) return;

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
    // Only include data points that actually exist (no default to 0)
    const heatmapValues = [];
    const powerLimits = heatmapData.power_limits;
    const concurrentRequests = heatmapData.concurrent_requests;

    concurrentRequests.forEach((concurrent, concurrentIdx) => {
      powerLimits.forEach((powerLimit, powerIdx) => {
        const value = quantData[powerLimit]?.[concurrent];
        if (value !== undefined && value !== null) {
          // Scale efficiency values to millions for display
          const displayValue = metric === 'efficiency' ? value / 1000000 : value;
          heatmapValues.push([concurrentIdx, powerIdx, displayValue.toFixed(2)]);
        }
      });
    });

    // Calculate proper min/max for visualMap
    if (heatmapValues.length === 0) {
      console.warn('No data points to display in heatmap');
      return;
    }

    let visualMapMin, visualMapMax;

    // Use global scale if provided (for unified comparison across quantizations)
    if (globalMin !== null && globalMax !== null) {
      visualMapMin = globalMin;
      visualMapMax = globalMax;
    } else {
      // Fall back to per-quantization scale calculation
      const values = heatmapValues.map(v => parseFloat(v[2]));
      const maxValue = Math.max(...values);
      const minValue = Math.min(...values);

      // For single values or small ranges, create a better scale
      if (maxValue === minValue) {
        // Single value - create range around it
        visualMapMin = maxValue * 0.8;
        visualMapMax = maxValue * 1.2;
      } else if (maxValue - minValue < maxValue * 0.1) {
        // Small range - expand it
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
      },
      tooltip: {
        position: 'top',
        formatter: function (params) {
          const concurrent = concurrentRequests[params.data[0]];
          const powerLimit = powerLimits[params.data[1]];
          const value = params.data[2];
          const label = metric === 'speed' ? 'Speed' : metric === 'ttft' ? 'TTFT' : 'Efficiency';
          const unit = metric === 'speed' ? ' tok/s' : metric === 'ttft' ? ' ms' : ' M tok/kWh';
          // Value is already scaled to millions for efficiency
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
        inRange: {
          color: metric === 'ttft'
            ? // Yellow-Orange-Red for latency: Yellow (low/good) -> Red (high/bad)
              // Starting with viridis yellow (#fde724) for consistency
              ['#fde724', '#fdc518', '#fda30c', '#fd8100', '#f96f00', '#f45d00', '#ef4800', '#e63000', '#d91800', '#cc0000']
            : // Viridis for speed and efficiency: Dark purple (low/bad) -> Yellow (high/good)
              ['#440154', '#482878', '#3e4989', '#31688e', '#26828e', '#1f9e89', '#35b779', '#6ece58', '#b5de2b', '#fde724']
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
          borderColor: '#fff'
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
