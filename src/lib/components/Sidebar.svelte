<script lang="ts">
  import FileTree from "./FileTree.svelte";
  import NewNoteButton from "./NewNoteButton.svelte";
  import TagPanel from "./TagPanel.svelte";
  import { vaultEntries } from "$lib/stores/vault";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  let activeTag = $state<string | null>(null);
  let tags = $state<{ name: string; count: number }[]>([]);

  function handleTagFilter(e: CustomEvent<string | null>) {
    activeTag = e.detail;
  }
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <NewNoteButton />
    <button class="settings-btn" onclick={() => dispatch("open-settings")} title="Settings"
      >⚙️</button
    >
  </div>
  <div class="sidebar-content">
    <FileTree entries={$vaultEntries} />
  </div>
  <TagPanel {tags} {activeTag} on:filter={handleTagFilter} />
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

  .sidebar-header {
    display: flex;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid #2d3f50;
  }

  .sidebar-header > :first-child {
    flex: 1;
  }

  .settings-btn {
    background: none;
    border: 1px solid #2d3f50;
    color: #8899a6;
    cursor: pointer;
    font-size: 1rem;
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    transition: all 0.15s;
  }

  .settings-btn:hover {
    background: #2d3f50;
    color: #e7e9ea;
  }

  .sidebar-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
