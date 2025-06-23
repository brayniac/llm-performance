<script>
  export let comparisonData;
</script>

<div class="comparison-header">
  <div class="config-card config-a">
    <h3>{comparisonData.config_a.name}</h3>
    <div class="config-details">
      <div class="detail">
        <span class="label">Model:</span>
        <span class="value">{comparisonData.config_a.model}</span>
      </div>
      <div class="detail">
        <span class="label">Quantization:</span>
        <span class="value">{comparisonData.config_a.quantization}</span>
      </div>
      <div class="detail">
        <span class="label">Backend:</span>
        <span class="value">{comparisonData.config_a.backend}</span>
      </div>
      <div class="detail">
        <span class="label">Hardware:</span>
        <span class="value">{comparisonData.config_a.hardware}</span>
      </div>
    </div>
  </div>
  
  <div class="vs">VS</div>
  
  <div class="config-card config-b">
    <h3>{comparisonData.config_b.name}</h3>
    <div class="config-details">
      <div class="detail">
        <span class="label">Model:</span>
        <span class="value">{comparisonData.config_b.model}</span>
      </div>
      <div class="detail">
        <span class="label">Quantization:</span>
        <span class="value">{comparisonData.config_b.quantization}</span>
      </div>
      <div class="detail">
        <span class="label">Backend:</span>
        <span class="value">{comparisonData.config_b.backend}</span>
      </div>
      <div class="detail">
        <span class="label">Hardware:</span>
        <span class="value">{comparisonData.config_b.hardware}</span>
      </div>
    </div>
  </div>
</div>

<style>
  .comparison-header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    gap: 2rem;
    margin-bottom: 3rem;
    align-items: center;
  }
  
  .config-card {
    background: white;
    border: 2px solid #e1e5e9;
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }
  
  .config-card.config-a {
    border-color: #5470c6;
  }
  
  .config-card.config-b {
    border-color: #ee6666;
  }
  
  .config-card h3 {
    margin: 0 0 1rem 0;
    color: #2c3e50;
    font-size: 1.25rem;
  }
  
  .config-details {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .detail {
    display: flex;
    justify-content: space-between;
  }
  
  .detail .label {
    font-weight: 500;
    color: #6c757d;
  }
  
  .detail .value {
    font-family: monospace;
    font-weight: 600;
  }
  
  .vs {
    background: #f8f9fa;
    border-radius: 50%;
    width: 60px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 1.25rem;
    color: #6c757d;
    justify-self: center;
  }
</style>