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
    background: #0f1419;
  }

  .editor-header {
    padding: 0.5rem 1rem;
    background: #1a2332;
    border-bottom: 1px solid #2d3f50;
    font-size: 0.8rem;
    color: #8899a6;
  }

  .path {
    font-family: monospace;
  }

  .error-bar {
    padding: 0.5rem 1rem;
    background: #2d1a1a;
    color: #e74c3c;
    font-size: 0.85rem;
    border-top: 1px solid #4a2020;
  }
</style>
