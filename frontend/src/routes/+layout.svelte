<script>
  import '../app.css';
</script>

<nav class="navbar">
  <div class="nav-container">
    <a href="/" class="nav-brand">
      <h1>LLM Benchmarks</h1>
    </a>
    
    <div class="nav-links">
      <a href="/">Performance Grid</a>
      <a href="/about">About</a>
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
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background-color: #f8f9fa;
    line-height: 1.5;
  }
  
  :global(*) {
    box-sizing: border-box;
  }
  
  .navbar {
    background: white;
    border-bottom: 1px solid #e1e5e9;
    padding: 1rem 0;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
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
    color: #2c3e50;
    font-size: 1.5rem;
    font-weight: 600;
  }
  
  .nav-brand:hover h1 {
    color: #2196f3;
    transition: color 0.2s;
  }
  
  .nav-links {
    display: flex;
    gap: 2rem;
  }
  
  .nav-links a {
    text-decoration: none;
    color: #6c757d;
    font-weight: 500;
    transition: color 0.2s;
    padding: 0.5rem 0;
  }
  
  .nav-links a:hover {
    color: #2196f3;
  }
  
  .main-content {
    min-height: calc(100vh - 140px);
    padding: 0;
  }
  
  .footer {
    background: #343a40;
    color: white;
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
    color: #7dd3fc;
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