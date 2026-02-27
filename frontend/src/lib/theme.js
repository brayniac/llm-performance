import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const STORAGE_KEY = 'theme-preference';

function createThemeStore() {
  const { subscribe, set, update } = writable('system');

  function getSystemTheme() {
    if (!browser) return 'light';
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  /** @param {string} preference */
  function applyTheme(preference) {
    if (!browser) return;
    const resolved = preference === 'system' ? getSystemTheme() : preference;
    document.documentElement.setAttribute('data-theme', resolved);
  }

  function initialize() {
    if (!browser) return;
    const stored = localStorage.getItem(STORAGE_KEY) || 'system';
    set(stored);
    applyTheme(stored);

    const mql = window.matchMedia('(prefers-color-scheme: dark)');
    mql.addEventListener('change', () => {
      let current;
      subscribe(v => { current = v; })();
      if (current === 'system') {
        applyTheme('system');
      }
    });
  }

  function toggle() {
    update(current => {
      const resolved = current === 'system' ? getSystemTheme() : current;
      const next = resolved === 'light' ? 'dark' : 'light';
      localStorage.setItem(STORAGE_KEY, next);
      applyTheme(next);
      return next;
    });
  }

  return {
    subscribe,
    initialize,
    toggle,
    /** @param {string} value */
    set: (value) => {
      if (browser) {
        localStorage.setItem(STORAGE_KEY, value);
      }
      applyTheme(value);
      set(value);
    }
  };
}

export const theme = createThemeStore();
