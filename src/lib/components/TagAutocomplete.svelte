<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let tags: string[] = [];
  export let selectedIndex = 0;
  export let visible = false;
  export let position = { top: 0, left: 0 };

  const dispatch = createEventDispatcher();

  function select(index: number) {
    dispatch("select", tags[index]);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % tags.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + tags.length) % tags.length;
    } else if (e.key === "Enter" || e.key === "Tab") {
      e.preventDefault();
      if (tags.length > 0) {
        select(selectedIndex);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      dispatch("close");
    }
  }

  $: if (selectedIndex >= tags.length) {
    selectedIndex = 0;
  }
</script>

{#if visible && tags.length > 0}
  <div
    class="tag-dropdown"
    style="top: {position.top}px; left: {position.left}px"
    on:keydown={handleKeydown}
  >
    {#each tags as tag, i (tag)}
      <button class="tag-item" class:selected={i === selectedIndex} on:click={() => select(i)}>
        #{tag}
      </button>
    {/each}
  </div>
{/if}

<style>
  .tag-dropdown {
    position: fixed;
    background: #1a2332;
    border: 1px solid #2d3f50;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    max-height: 200px;
    overflow-y: auto;
    z-index: 1000;
    min-width: 150px;
  }

  .tag-item {
    display: block;
    width: 100%;
    padding: 0.4rem 0.8rem;
    background: transparent;
    border: none;
    color: #27ae60;
    text-align: left;
    cursor: pointer;
    font-size: 0.85rem;
    transition: background 0.1s;
  }

  .tag-item:hover,
  .tag-item.selected {
    background: #27ae60;
    color: #fff;
  }
</style>
