<script lang="ts">
  import Sidebar from "$lib/components/Sidebar.svelte";
  import NoteEditor from "$lib/components/NoteEditor.svelte";
</script>

<div class="app">
  <Sidebar />
  <main class="main-content">
    <NoteEditor />
  </main>
</div>

<style>
  :global(body) {
    margin: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
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
