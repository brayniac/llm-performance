import { browser } from '$app/environment';

const LIGHT_DEFAULTS = {
  text: '#52525b',
  grid: '#e4e4e7',
  bg: '#ffffff',
  label: '#52525b',
  cellBorder: '#ffffff',
  seriesA: '#6366f1',
  seriesB: '#f43f5e',
  palette: [
    '#6366f1', '#16a34a', '#ca8a04', '#f43f5e',
    '#0891b2', '#059669', '#ea580c', '#9333ea'
  ],
  accent: '#3b82f6',
  title: '#18181b'
};

export function getChartColors() {
  if (!browser) return { ...LIGHT_DEFAULTS };

  const s = getComputedStyle(document.documentElement);
  /** @param {string} prop */
  const get = (prop) => s.getPropertyValue(prop).trim();

  return {
    text: get('--color-chart-text') || LIGHT_DEFAULTS.text,
    grid: get('--color-chart-grid') || LIGHT_DEFAULTS.grid,
    bg: get('--color-chart-bg') || LIGHT_DEFAULTS.bg,
    label: get('--color-chart-text') || LIGHT_DEFAULTS.label,
    cellBorder: get('--color-chart-cell-border') || LIGHT_DEFAULTS.cellBorder,
    seriesA: get('--color-series-a') || LIGHT_DEFAULTS.seriesA,
    seriesB: get('--color-series-b') || LIGHT_DEFAULTS.seriesB,
    palette: [
      get('--color-chart-1') || LIGHT_DEFAULTS.palette[0],
      get('--color-chart-2') || LIGHT_DEFAULTS.palette[1],
      get('--color-chart-3') || LIGHT_DEFAULTS.palette[2],
      get('--color-chart-4') || LIGHT_DEFAULTS.palette[3],
      get('--color-chart-5') || LIGHT_DEFAULTS.palette[4],
      get('--color-chart-6') || LIGHT_DEFAULTS.palette[5],
      get('--color-chart-7') || LIGHT_DEFAULTS.palette[6],
      get('--color-chart-8') || LIGHT_DEFAULTS.palette[7],
    ],
    accent: get('--color-accent') || LIGHT_DEFAULTS.accent,
    title: get('--color-text-primary') || LIGHT_DEFAULTS.title
  };
}
