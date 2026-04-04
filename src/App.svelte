<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
  import SearchModal from "$lib/components/SearchModal.svelte";
  import { openNote } from "$lib/stores/vault-actions";

  let showSearch = $state(false);

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
  <Sidebar />
  <main class="main-content">
    <NoteEditor />
  </main>
  <SearchModal
    visible={showSearch}
    on:select={handleSearchSelect}
    on:close={() => (showSearch = false)}
  />
</div>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #0f1419;
    color: #e7e9ea;
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
