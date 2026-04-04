<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  interface Props {
    visible: boolean;
  }

  let { visible }: Props = $props();
  const dispatch = createEventDispatcher();

  let query = $state("");
  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);
  let isLoading = $state(false);
  let searchTimeout: ReturnType<typeof setTimeout> | null = null;

  interface SearchResult {
    path: string;
    title: string;
    score: number;
    snippet: string;
  }

  function performSearch() {
    if (searchTimeout) clearTimeout(searchTimeout);

    if (!query.trim()) {
      results = [];
      selectedIndex = 0;
      return;
    }

    isLoading = true;
    searchTimeout = setTimeout(async () => {
      try {
        results = await invoke<SearchResult[]>("search_notes", {
          query,
          limit: 20,
        });
        selectedIndex = 0;
      } catch (e) {
        console.error("Search failed:", e);
        results = [];
      } finally {
        isLoading = false;
      }
    }, 200);
  }

  function selectResult(index: number) {
    if (results[index]) {
      dispatch("select", results[index]);
      close();
    }
  }

  function close() {
    visible = false;
    query = "";
    results = [];
    selectedIndex = 0;
    dispatch("close");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;

    if (e.key === "Escape") {
      e.preventDefault();
      close();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (results.length > 0) {
        selectResult(selectedIndex);
      }
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    if (searchTimeout) clearTimeout(searchTimeout);
  });
</script>

{#if visible}
  <div class="modal-overlay" on:click={close}>
    <div class="search-modal" on:click|stopPropagation on:keydown={handleKeydown}>
      <div class="search-input-wrapper">
        <span class="search-icon">🔍</span>
        <input
          type="text"
          bind:value={query}
          on:input={performSearch}
          placeholder="Search notes..."
          autofocus
        />
        <kbd class="shortcut-hint">ESC</kbd>
      </div>

      {#if isLoading}
        <div class="loading">Searching...</div>
      {:else if results.length > 0}
        <div class="results">
          {#each results as result, i (result.path)}
            <button
              class="result-item"
              class:selected={i === selectedIndex}
              on:click={() => selectResult(i)}
            >
              <div class="result-title">{result.title}</div>
              <div class="result-snippet">{result.snippet}</div>
              <div class="result-path">{result.path}</div>
            </button>
          {/each}
        </div>
      {:else if query.trim()}
        <div class="no-results">No results found</div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 1000;
  }

  .search-modal {
    background: #1a2332;
    border: 1px solid #2d3f50;
    border-radius: 12px;
    width: 90%;
    max-width: 600px;
    max-height: 60vh;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #2d3f50;
  }

  .search-icon {
    margin-right: 0.75rem;
    font-size: 1.1rem;
  }

  .search-input-wrapper input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: #e7e9ea;
    font-size: 1rem;
  }

  .search-input-wrapper input::placeholder {
    color: #556677;
  }

  .shortcut-hint {
    background: #2d3f50;
    color: #8899a6;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.7rem;
    font-family: monospace;
  }

  .loading {
    padding: 2rem;
    text-align: center;
    color: #8899a6;
  }

  .results {
    max-height: calc(60vh - 70px);
    overflow-y: auto;
  }

  .result-item {
    display: block;
    width: 100%;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
    border-bottom: 1px solid #2d3f50;
  }

  .result-item:hover,
  .result-item.selected {
    background: #2d3f50;
  }

  .result-title {
    font-weight: 600;
    color: #e7e9ea;
    margin-bottom: 0.25rem;
  }

  .result-snippet {
    font-size: 0.85rem;
    color: #8899a6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-path {
    font-size: 0.75rem;
    color: #556677;
    font-family: monospace;
    margin-top: 0.25rem;
  }

  .no-results {
    padding: 2rem;
    text-align: center;
    color: #556677;
  }
</style>
