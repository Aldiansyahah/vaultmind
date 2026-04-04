<script lang="ts">
  import { selectedNotePath } from "$lib/stores/vault";
  import { openNote } from "$lib/stores/vault-actions";
  import type { VaultEntry } from "$lib/stores/vault";
  import FileTree from "./FileTree.svelte";

  interface Props {
    entries: VaultEntry[];
  }

  let { entries }: Props = $props();

  function handleEntryClick(entry: VaultEntry) {
    if (!entry.is_directory) {
      openNote(entry.path);
    }
  }

  function entryIcon(entry: VaultEntry): string {
    return entry.is_directory ? "📁" : "📄";
  }

  function entryClass(entry: VaultEntry): string {
    const base = "entry";
    if (entry.is_directory) return `${base} directory`;
    if ($selectedNotePath === entry.path) return `${base} selected`;
    return base;
  }
</script>

{#each entries as entry (entry.path)}
  <div>
    <div class={entryClass(entry)} onclick={() => handleEntryClick(entry)}>
      <span class="icon">{entryIcon(entry)}</span>
      <span class="name">{entry.name}</span>
    </div>
    {#if entry.is_directory && entry.children && entry.children.length > 0}
      <div class="children">
        <FileTree entries={entry.children} />
      </div>
    {/if}
  </div>
{/each}
{#if entries.length === 0}
  <div class="empty">No notes yet. Create your first note!</div>
{/if}

<style>
  .entry {
    display: flex;
    align-items: center;
    padding: 0.35rem 1rem;
    cursor: pointer;
    font-size: 0.9rem;
    color: var(--text-primary);
    transition: background 0.15s;
  }

  .entry:hover {
    background: var(--hover-bg);
  }

  .entry.selected {
    background: var(--accent-color);
    color: #fff;
  }

  .entry.directory {
    font-weight: 500;
  }

  .icon {
    margin-right: 0.5rem;
    font-size: 0.85rem;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .children {
    padding-left: 1rem;
  }

  .empty {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.85rem;
  }
</style>
