<script lang="ts">
  import { createNewNote } from "$lib/stores/vault-actions";

  let showPrompt = $state(false);
  let newName = $state("");
  let errorMsg = $state("");

  async function handleCreate() {
    errorMsg = "";
    const trimmed = newName.trim();
    if (!trimmed) {
      errorMsg = "Name cannot be empty";
      return;
    }
    await createNewNote(trimmed);
    newName = "";
    showPrompt = false;
  }

  function openPrompt() {
    newName = "";
    errorMsg = "";
    showPrompt = true;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      handleCreate();
    } else if (e.key === "Escape") {
      showPrompt = false;
    }
  }
</script>

<button class="new-note-btn" onclick={openPrompt}>+ New Note</button>

{#if showPrompt}
  <div class="modal-overlay" onclick={() => (showPrompt = false)}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h3>Create New Note</h3>
      <input
        type="text"
        bind:value={newName}
        placeholder="note-name"
        onkeydown={handleKeydown}
        autofocus
      />
      {#if errorMsg}
        <p class="error">{errorMsg}</p>
      {/if}
      <div class="modal-actions">
        <button class="cancel-btn" onclick={() => (showPrompt = false)}>Cancel</button>
        <button class="create-btn" onclick={handleCreate}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .new-note-btn {
    width: 100%;
    padding: 0.5rem;
    background: #2e86c1;
    color: #fff;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    transition: background 0.15s;
  }

  .new-note-btn:hover {
    background: #2471a3;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #1a2332;
    border: 1px solid #2d3f50;
    border-radius: 12px;
    padding: 1.5rem;
    min-width: 320px;
  }

  .modal h3 {
    margin: 0 0 1rem;
    color: #e7e9ea;
    font-size: 1rem;
  }

  .modal input {
    width: 100%;
    padding: 0.5rem;
    background: #0f1419;
    border: 1px solid #2d3f50;
    border-radius: 6px;
    color: #e7e9ea;
    font-size: 0.9rem;
    outline: none;
    box-sizing: border-box;
  }

  .modal input:focus {
    border-color: #2e86c1;
  }

  .error {
    color: #e74c3c;
    font-size: 0.8rem;
    margin: 0.5rem 0 0;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .cancel-btn {
    padding: 0.4rem 1rem;
    background: transparent;
    border: 1px solid #2d3f50;
    color: #8899a6;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .cancel-btn:hover {
    background: #2d3f50;
  }

  .create-btn {
    padding: 0.4rem 1rem;
    background: #2e86c1;
    border: none;
    color: #fff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .create-btn:hover {
    background: #2471a3;
  }
</style>
