<script>
  import { onMount } from 'svelte';
  import { theme } from '$lib/theme.js';
  import '../app.css';

  onMount(() => {
    theme.initialize();
  });
</script>

<nav class="navbar">
  <div class="nav-container">
    <a href="/" class="nav-brand">
      <h1>LLM Benchmarks</h1>
    </a>

    <div class="nav-links">
      <a href="/">Performance Grid</a>
      <a href="/about">About</a>
      <button class="theme-toggle" on:click={theme.toggle} aria-label="Toggle theme">
        {#if $theme === 'dark'}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
        {/if}
      </button>
    </div>
  </div>
</nav>

<main class="main-content">
  <slot />
</main>

<footer class="footer">
  <div class="footer-content">
    <p>Powered by Rust + SvelteKit + ECharts</p>
    <p>Data collected with <a href="https://github.com/iopsystems/rezolus" target="_blank" rel="noopener noreferrer">Rezolus</a></p>
  </div>
</footer>

<style>
  .navbar {
    background: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border-primary);
    padding: 1rem 0;
    box-shadow: var(--shadow-sm);
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .nav-container {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 2rem;
  }

  .nav-brand {
    text-decoration: none;
  }

  .nav-brand h1 {
    margin: 0;
    color: var(--color-text-primary);
    font-size: 1.5rem;
    font-weight: 600;
  }

  .nav-brand:hover h1 {
    color: var(--color-accent);
    transition: color 0.2s;
  }

  .nav-links {
    display: flex;
    gap: 2rem;
    align-items: center;
  }

  .nav-links a {
    text-decoration: none;
    color: var(--color-text-tertiary);
    font-weight: 500;
    transition: color 0.2s;
    padding: 0.5rem 0;
  }

  .nav-links a:hover {
    color: var(--color-accent);
  }

  .theme-toggle {
    background: none;
    border: 1px solid var(--color-border-primary);
    color: var(--color-text-secondary);
    padding: 0.4rem;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .theme-toggle:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .main-content {
    min-height: calc(100vh - 140px);
    padding: 0;
  }

  .footer {
    background: var(--color-footer-bg);
    color: var(--color-footer-text);
    padding: 2rem 0;
    text-align: center;
    margin-top: auto;
  }

  .footer-content {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 2rem;
  }

  .footer-content p {
    margin: 0.5rem 0;
    opacity: 0.8;
    font-size: 0.9rem;
  }

  .footer-content a {
    color: var(--color-footer-link);
    text-decoration: none;
  }

  .footer-content a:hover {
    text-decoration: underline;
  }

  /* Responsive design */
  @media (max-width: 768px) {
    .nav-container {
      padding: 0 1rem;
      flex-direction: column;
      gap: 1rem;
    }

    .nav-brand h1 {
      font-size: 1.25rem;
    }

    .nav-links {
      gap: 1.5rem;
    }

    .nav-links a {
      font-size: 0.9rem;
    }
  }
</style>
