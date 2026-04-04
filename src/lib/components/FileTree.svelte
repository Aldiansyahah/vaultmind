<script lang="ts">
  import { selectedNotePath } from "$lib/stores/vault";
  import { openNote, renameNote, deleteNoteByPath } from "$lib/stores/vault-actions";
  import type { VaultEntry } from "$lib/stores/vault";
  import FileTree from "./FileTree.svelte";

  interface Props {
    entries: VaultEntry[];
  }

  let { entries }: Props = $props();

  let contextMenu = $state<{ x: number; y: number; entry: VaultEntry } | null>(null);
  let renamingPath = $state<string | null>(null);
  let renameValue = $state("");
  let deleteConfirm = $state<VaultEntry | null>(null);

  function handleEntryClick(entry: VaultEntry) {
    if (!entry.is_directory && renamingPath !== entry.path) {
      openNote(entry.path);
    }
  }

  function handleContextMenu(e: MouseEvent, entry: VaultEntry) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, entry };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function startRename(entry: VaultEntry) {
    renamingPath = entry.path;
    renameValue = entry.name.replace(/\.md$/, "");
    closeContextMenu();
  }

  async function commitRename(entry: VaultEntry) {
    const trimmed = renameValue.trim();
    if (trimmed && trimmed !== entry.name.replace(/\.md$/, "")) {
      await renameNote(entry.path, trimmed);
    }
    renamingPath = null;
    renameValue = "";
  }

  function handleRenameKeydown(e: KeyboardEvent, entry: VaultEntry) {
    if (e.key === "Enter") {
      commitRename(entry);
    } else if (e.key === "Escape") {
      renamingPath = null;
      renameValue = "";
    }
  }

  function confirmDelete(entry: VaultEntry) {
    deleteConfirm = entry;
    closeContextMenu();
  }

  async function executeDelete() {
    if (deleteConfirm) {
      await deleteNoteByPath(deleteConfirm.path);
      deleteConfirm = null;
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

<svelte:window onclick={closeContextMenu} />

{#each entries as entry (entry.path)}
  <div>
    <div
      class={entryClass(entry)}
      onclick={() => handleEntryClick(entry)}
      oncontextmenu={(e) => handleContextMenu(e, entry)}
      ondblclick={() => !entry.is_directory && startRename(entry)}
      role="treeitem"
      tabindex="0"
    >
      <span class="icon">{entryIcon(entry)}</span>
      {#if renamingPath === entry.path}
        <input
          class="rename-input"
          type="text"
          bind:value={renameValue}
          onblur={() => commitRename(entry)}
          onkeydown={(e) => handleRenameKeydown(e, entry)}
          autofocus
        />
      {:else}
        <span class="name">{entry.name}</span>
      {/if}
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

{#if contextMenu}
  <div
    class="context-menu"
    style="top: {contextMenu.y}px; left: {contextMenu.x}px"
    role="menu"
    tabindex="-1"
  >
    {#if !contextMenu.entry.is_directory}
      <button class="menu-item" onclick={() => startRename(contextMenu.entry)} role="menuitem">
        ✏️ Rename
      </button>
    {/if}
    <button
      class="menu-item danger"
      onclick={() => confirmDelete(contextMenu.entry)}
      role="menuitem"
    >
      🗑️ Delete
    </button>
  </div>
{/if}

{#if deleteConfirm}
  <div class="modal-overlay" onclick={() => (deleteConfirm = null)} role="dialog" tabindex="-1">
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h3>Delete "{deleteConfirm.name}"?</h3>
      <p>This action cannot be undone.</p>
      <div class="modal-actions">
        <button class="cancel-btn" onclick={() => (deleteConfirm = null)}>Cancel</button>
        <button class="delete-btn" onclick={executeDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .entry {
    display: flex;
    align-items: center;
    padding: 0.4rem 0.75rem;
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--text-primary);
    transition: background 0.12s;
    border-radius: 4px;
    margin: 1px 4px;
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
    font-size: 0.8rem;
    flex-shrink: 0;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .children {
    padding-left: 0.75rem;
    border-left: 1px solid var(--border-color);
    margin-left: 1rem;
  }

  .empty {
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.85rem;
  }

  .rename-input {
    flex: 1;
    padding: 0.15rem 0.4rem;
    background: var(--bg-primary);
    border: 1px solid var(--accent-color);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.875rem;
    outline: none;
    min-width: 0;
  }

  .context-menu {
    position: fixed;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 4px;
    min-width: 150px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
    z-index: 2000;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.45rem 0.75rem;
    background: none;
    border: none;
    border-radius: 4px;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.85rem;
    text-align: left;
  }

  .menu-item:hover {
    background: var(--hover-bg);
  }

  .menu-item.danger {
    color: var(--error-color);
  }

  .menu-item.danger:hover {
    background: var(--error-bg);
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--overlay-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
  }

  .modal {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 1.5rem;
    min-width: 300px;
    max-width: 400px;
  }

  .modal h3 {
    margin: 0 0 0.5rem;
    font-size: 1rem;
    color: var(--text-primary);
  }

  .modal p {
    margin: 0 0 1rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }

  .cancel-btn {
    padding: 0.4rem 1rem;
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .cancel-btn:hover {
    background: var(--hover-bg);
  }

  .delete-btn {
    padding: 0.4rem 1rem;
    background: var(--error-color);
    border: none;
    color: #fff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .delete-btn:hover {
    opacity: 0.9;
  }
</style>
