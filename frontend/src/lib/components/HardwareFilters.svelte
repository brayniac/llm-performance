<script>
  import { createEventDispatcher } from 'svelte';
  
  export let selectedCategories = [];
  
  const dispatch = createEventDispatcher();
  
  const categories = [
    { value: 'consumer_gpu', label: 'Consumer GPU' },
    { value: 'consumer_cpu', label: 'Consumer CPU' },
    { value: 'datacenter_gpu', label: 'Datacenter GPU' },
    { value: 'datacenter_cpu', label: 'Datacenter CPU' }
  ];
  
  function handleCategoryChange(category) {
    if (selectedCategories.includes(category)) {
      selectedCategories = selectedCategories.filter(c => c !== category);
    } else {
      selectedCategories = [...selectedCategories, category];
    }
    
    dispatch('change', { selectedCategories });
  }
</script>

<div class="hardware-filters">
  <h4>Hardware Categories</h4>
  <div class="category-list">
    {#each categories as category}
      <label class="category-option">
        <input
          type="checkbox"
          checked={selectedCategories.includes(category.value)}
          on:change={() => handleCategoryChange(category.value)}
        />
        <span>{category.label}</span>
      </label>
    {/each}
  </div>
</div>

<style>
  .hardware-filters {
    margin-bottom: 1rem;
  }
  
  h4 {
    margin: 0 0 0.75rem 0;
    font-size: 0.9rem;
    font-weight: 600;
    color: #495057;
  }
  
  .category-list {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 0.5rem;
  }
  
  .category-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }
  
  .category-option:hover {
    background-color: #f8f9fa;
  }
  
  .category-option input[type="checkbox"] {
    cursor: pointer;
  }
  
  .category-option span {
    font-size: 0.9rem;
    color: #2c3e50;
    user-select: none;
  }
  
  @media (max-width: 768px) {
    .category-list {
      grid-template-columns: 1fr;
    }
  }
</style>