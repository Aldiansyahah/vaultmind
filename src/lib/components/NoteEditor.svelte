<script lang="ts">
  import TipTapEditor from "./TipTapEditor.svelte";
  import { selectedNotePath, error } from "$lib/stores/vault";
</script>

<div class="editor-container">
  {#if $selectedNotePath}
    <div class="editor-header">
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
    padding: 0.5rem 1rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .path {
    font-family: monospace;
  }

  .error-bar {
    padding: 0.5rem 1rem;
    background: var(--error-bg);
    color: var(--error-color);
    font-size: 0.85rem;
    border-top: 1px solid var(--error-color);
  }
</style>
