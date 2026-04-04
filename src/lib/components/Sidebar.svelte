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
  <div class="sidebar-brand">
    <span class="brand-icon">V</span>
    <span class="brand-name">VaultMind</span>
    <button class="settings-btn" onclick={() => dispatch("open-settings")} title="Settings"
      >⚙</button
    >
  </div>
  <div class="sidebar-actions">
    <NewNoteButton />
  </div>
  <div class="sidebar-section-label">Notes</div>
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
    background: var(--bg-secondary);
    border-right: 1px solid var(--border-color);
    width: 260px;
    min-width: 200px;
  }

  .sidebar-brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border-color);
  }

  .brand-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    background: var(--accent-color);
    color: #fff;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 700;
    flex-shrink: 0;
  }

  .brand-name {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
    flex: 1;
  }

  .settings-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1rem;
    padding: 0.25rem;
    border-radius: 4px;
    transition: all 0.12s;
  }

  .settings-btn:hover {
    background: var(--hover-bg);
    color: var(--text-primary);
  }

  .sidebar-actions {
    padding: 0.5rem 0.75rem;
  }

  .sidebar-section-label {
    padding: 0.5rem 0.75rem 0.25rem;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-tertiary);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }
</style>
