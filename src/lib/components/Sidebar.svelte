<script lang="ts">
  import FileTree from "./FileTree.svelte";
  import NewNoteButton from "./NewNoteButton.svelte";
  import { vaultEntries } from "$lib/stores/vault";
</script>

<div class="sidebar">
  <div class="sidebar-actions">
    <NewNoteButton />
  </div>
  <div class="sidebar-content">
    <FileTree entries={$vaultEntries} />
  </div>
</div>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1a2332;
    border-right: 1px solid #2d3f50;
    width: 260px;
    min-width: 200px;
  }

  .sidebar-actions {
    padding: 0.75rem;
    border-bottom: 1px solid #2d3f50;
  }

  .sidebar-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
