<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let suggestions: string[] = [];
  export let selectedIndex = 0;
  export let visible = false;
  export let position = { top: 0, left: 0 };

  const dispatch = createEventDispatcher();

  function select(index: number) {
    dispatch("select", suggestions[index]);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % suggestions.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + suggestions.length) % suggestions.length;
    } else if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      if (suggestions.length > 0) {
        select(selectedIndex);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      dispatch("close");
    }
  }

  $: if (selectedIndex >= suggestions.length) {
    selectedIndex = 0;
  }
</script>

{#if visible && suggestions.length > 0}
  <div
    class="autocomplete-dropdown"
    style="top: {position.top}px; left: {position.left}px"
    on:keydown={handleKeydown}
  >
    {#each suggestions as suggestion, i (suggestion)}
      <button
        class="autocomplete-item"
        class:selected={i === selectedIndex}
        on:click={() => select(i)}
      >
        {suggestion}
      </button>
    {/each}
  </div>
{/if}

<style>
  .autocomplete-dropdown {
    position: fixed;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    max-height: 200px;
    overflow-y: auto;
    z-index: 1000;
    min-width: 200px;
  }

  .autocomplete-item {
    display: block;
    width: 100%;
    padding: 0.5rem 1rem;
    background: transparent;
    border: none;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.1s;
  }

  .autocomplete-item:hover,
  .autocomplete-item.selected {
    background: var(--accent-color);
    color: #fff;
  }
</style>
