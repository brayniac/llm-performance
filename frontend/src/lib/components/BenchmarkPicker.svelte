<script>
  import { createEventDispatcher } from 'svelte';

  export let selectedBenchmark = 'mmlu';

  const dispatch = createEventDispatcher();

  const benchmarks = [
    { value: 'none', label: 'No Benchmark', description: 'Sort by performance metrics only' },
    { value: 'mmlu', label: 'MMLU Pro', description: 'Massive Multitask Language Understanding' },
    { value: 'gsm8k', label: 'GSM8K', description: 'Grade School Math' },
    { value: 'humaneval', label: 'HumanEval', description: 'Code Generation' },
    { value: 'hellaswag', label: 'HellaSwag', description: 'Commonsense Reasoning' },
    { value: 'truthfulqa', label: 'TruthfulQA', description: 'Truthfulness' }
  ];

  function handleChange() {
    dispatch('change', { benchmark: selectedBenchmark });
  }
</script>

<div class="benchmark-picker">
  <label>
    <span class="label-text">Quality Benchmark:</span>
    <select bind:value={selectedBenchmark} on:change={handleChange}>
      {#each benchmarks as benchmark}
        <option value={benchmark.value}>
          {benchmark.label} - {benchmark.description}
        </option>
      {/each}
    </select>
  </label>

  <div class="benchmark-info">
    {#each benchmarks as benchmark}
      {#if benchmark.value === selectedBenchmark}
        <p>{benchmark.description}</p>
      {/if}
    {/each}
  </div>
</div>

<style>
  .benchmark-picker {
    margin-bottom: 1rem;
  }

  label {
    display: block;
  }

  .label-text {
    font-weight: 600;
    display: block;
    margin-bottom: 0.5rem;
    color: var(--color-text-primary);
  }

  select {
    width: 100%;
    max-width: 400px;
    padding: 0.6rem;
    font-size: 1rem;
    border: 2px solid var(--color-border-primary);
    border-radius: 6px;
    background: var(--color-input-bg);
    color: var(--color-text-primary);
    cursor: pointer;
    transition: border-color 0.2s;
  }

  select:hover {
    border-color: var(--color-accent);
  }

  select:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px var(--color-input-focus-ring);
  }

  .benchmark-info {
    margin-top: 0.5rem;
    font-size: 0.9rem;
    color: var(--color-text-tertiary);
  }

  .benchmark-info p {
    margin: 0;
  }
</style>
