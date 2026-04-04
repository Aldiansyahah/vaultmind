<script lang="ts">
  import TipTapEditor from "./TipTapEditor.svelte";
  import { selectedNotePath, error } from "$lib/stores/vault";

  function getTitle(path: string): string {
    const name = path.split("/").pop() || path;
    return name.replace(/\.md$/, "");
  }
</script>

<div class="editor-container">
  {#if $selectedNotePath}
    <div class="editor-header">
      <span class="title">{getTitle($selectedNotePath)}</span>
      <span class="path">{$selectedNotePath}</span>
    </div>
  {/if}
  <TipTapEditor />
  {#if $error}
    <div class="error-bar">{$error}</div>
  {/if}
</div>

<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .editor-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    padding: 0.6rem 1rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }

  .title {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .path {
    font-family: monospace;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .error-bar {
    padding: 0.5rem 1rem;
    background: var(--error-bg);
    color: var(--error-color);
    font-size: 0.85rem;
    border-top: 1px solid var(--error-color);
  }
</style>
