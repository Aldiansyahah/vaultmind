<script lang="ts">
  import { createEventDispatcher } from "svelte";

  export let tags: { name: string; count: number }[] = [];
  export let activeTag: string | null = null;

  const dispatch = createEventDispatcher();

  function toggleTag(tag: string) {
    if (activeTag === tag) {
      dispatch("filter", null);
    } else {
      dispatch("filter", tag);
    }
  }
</script>

<div class="tag-panel">
  <div class="tag-panel-header">
    <span>Tags</span>
    {#if activeTag}
      <button class="clear-btn" onclick={() => dispatch("filter", null)}>✕</button>
    {/if}
  </div>
  <div class="tag-list">
    {#each tags as tag (tag.name)}
      <button
        class="tag-chip"
        class:active={activeTag === tag.name}
        onclick={() => toggleTag(tag.name)}
      >
        #{tag.name}
        <span class="count">{tag.count}</span>
      </button>
    {/each}
    {#if tags.length === 0}
      <div class="empty">No tags yet</div>
    {/if}
  </div>
</div>

<style>
  .tag-panel {
    border-top: 1px solid var(--border-color);
  }

  .tag-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1rem;
    font-weight: 600;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }

  .clear-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.15rem;
  }

  .clear-btn:hover {
    color: var(--text-primary);
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    padding: 0.5rem;
    max-height: 150px;
    overflow-y: auto;
  }

  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.6rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    color: var(--success-color);
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.15s;
  }

  .tag-chip:hover {
    background: var(--hover-bg);
  }

  .tag-chip.active {
    background: var(--success-color);
    border-color: var(--success-color);
    color: #fff;
  }

  .count {
    font-size: 0.7rem;
    color: var(--text-secondary);
    background: var(--bg-primary);
    padding: 0.1rem 0.35rem;
    border-radius: 8px;
    min-width: 1.2rem;
    text-align: center;
  }

  .tag-chip.active .count {
    background: rgba(0, 0, 0, 0.2);
    color: #fff;
  }

  .empty {
    padding: 0.5rem;
    color: var(--text-tertiary);
    font-size: 0.8rem;
  }
</style>
