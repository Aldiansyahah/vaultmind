<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import SearchModal from "$lib/components/SearchModal.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import { openNote } from "$lib/stores/vault-actions";

  let showSearch = $state(false);
  let showSettings = $state(false);

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === "k") {
      e.preventDefault();
      showSearch = !showSearch;
    }
  }

  function handleSearchSelect(e: CustomEvent) {
    const result = e.detail;
    if (result && result.path) {
      openNote(result.path);
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

<div class="app">
  <Sidebar on:open-settings={() => (showSettings = true)} />
  <main class="main-content">
    <NoteEditor />
  </main>
  <SearchModal
    visible={showSearch}
    on:select={handleSearchSelect}
    on:close={() => (showSearch = false)}
  />
  {#if showSettings}
    <SettingsView on:close={() => (showSettings = false)} />
  {/if}
</div>

<style>
  :global([data-theme="light"]) {
    --bg-primary: #ffffff;
    --bg-secondary: #f5f5f5;
    --text-primary: #1a1a1a;
    --text-secondary: #666666;
    --text-tertiary: #999999;
    --border-color: #e0e0e0;
    --accent-color: #2e86c1;
    --accent-hover: #2471a3;
    --success-color: #27ae60;
    --success-bg: #e8f8e8;
    --error-color: #e74c3c;
    --error-bg: #fde8e8;
    --overlay-bg: rgba(0, 0, 0, 0.4);
    --hover-bg: #eaeaea;
  }

  :global([data-theme="dark"]) {
    --bg-primary: #0f1419;
    --bg-secondary: #1a2332;
    --text-primary: #e7e9ea;
    --text-secondary: #8899a6;
    --text-tertiary: #556677;
    --border-color: #2d3f50;
    --accent-color: #2e86c1;
    --accent-hover: #2471a3;
    --success-color: #27ae60;
    --success-bg: #1a3a1a;
    --error-color: #e74c3c;
    --error-bg: #3a1a1a;
    --overlay-bg: rgba(0, 0, 0, 0.7);
    --hover-bg: #2d3f50;
  }

  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
  }

  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow: hidden;
  }
</style>
