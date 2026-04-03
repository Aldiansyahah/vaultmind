<script lang="ts">
  import { selectedNotePath, noteContent, isLoading, error } from "$lib/stores/vault";
  import { saveNoteContent } from "$lib/stores/vault-actions";

  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  function handleContentChange(content: string) {
    noteContent.set(content);
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      if ($selectedNotePath) {
        saveNoteContent($selectedNotePath, content);
      }
    }, 1000);
  }
</script>

<div class="editor-container">
  {#if $isLoading}
    <div class="loading">Loading...</div>
  {:else if $selectedNotePath}
    <div class="editor-header">
      <span class="path">{$selectedNotePath}</span>
    </div>
    <textarea
      class="editor"
      bind:value={$noteContent}
      oninput={(e) => handleContentChange((e.target as HTMLTextAreaElement).value)}
      placeholder="Start writing..."
    ></textarea>
  {:else}
    <div class="placeholder">
      <p>Select a note from the file tree or create a new one.</p>
    </div>
  {/if}
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

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #8899a6;
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

  .editor {
    flex: 1;
    padding: 1rem;
    background: #0f1419;
    color: #e7e9ea;
    border: none;
    outline: none;
    resize: none;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.95rem;
    line-height: 1.6;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #556677;
    text-align: center;
  }

  .error-bar {
    padding: 0.5rem 1rem;
    background: #2d1a1a;
    color: #e74c3c;
    font-size: 0.85rem;
    border-top: 1px solid #4a2020;
  }
</style>
