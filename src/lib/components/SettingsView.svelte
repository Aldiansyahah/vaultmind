<script lang="ts">
  import { settings } from "$lib/stores/settings";
  import { createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { set_vault_path, loadVaultEntries } from "$lib/stores/vault-actions";
  import { open } from "@tauri-apps/plugin-dialog";

  const dispatch = createEventDispatcher();

  let localVaultPath = $state($settings.vaultPath || "");
  let localFontSize = $state($settings.editorFontSize);
  let localBaseUrl = $state($settings.llm.baseUrl || "");
  let localApiKey = $state($settings.llm.apiKey || "");
  let localModel = $state($settings.llm.model || "");
  let saveStatus = $state<"idle" | "saved" | "error">("idle");
  let reindexStatus = $state<"idle" | "running" | "done">("idle");
  let reindexResult = $state<string>("");

  function updateTheme(theme: "light" | "dark") {
    settings.update((s) => ({ ...s, theme }));
  }

  function updateFontSize(size: number) {
    localFontSize = size;
    settings.update((s) => ({ ...s, editorFontSize: size }));
    saveStatus = "saved";
    setTimeout(() => (saveStatus = "idle"), 2000);
  }

  async function updateVaultPath() {
    if (!localVaultPath.trim()) return;
    try {
      await set_vault_path(localVaultPath.trim());
      settings.update((s) => ({ ...s, vaultPath: localVaultPath.trim() }));
      await loadVaultEntries();
      saveStatus = "saved";
      setTimeout(() => (saveStatus = "idle"), 2000);
    } catch {
      saveStatus = "error";
      setTimeout(() => (saveStatus = "idle"), 3000);
    }
  }

  async function browseFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Vault Folder",
      });
      if (selected) {
        localVaultPath = selected as string;
        await updateVaultPath();
      }
    } catch {
      saveStatus = "error";
      setTimeout(() => (saveStatus = "idle"), 3000);
    }
  }

  function saveLlmConfig() {
    settings.update((s) => ({
      ...s,
      llm: {
        baseUrl: localBaseUrl.trim(),
        apiKey: localApiKey.trim(),
        model: localModel.trim(),
      },
    }));
    saveStatus = "saved";
    setTimeout(() => (saveStatus = "idle"), 2000);
  }

  async function triggerReindex() {
    reindexStatus = "running";
    reindexResult = "";
    try {
      const result: { indexed: number; skipped: number; errors: number } =
        await invoke("reindex_vault");
      reindexResult = `Indexed: ${result.indexed}, Skipped: ${result.skipped}, Errors: ${result.errors}`;
      reindexStatus = "done";
      setTimeout(() => (reindexStatus = "idle"), 5000);
    } catch (e) {
      reindexResult = `Failed: ${e}`;
      reindexStatus = "done";
      setTimeout(() => (reindexStatus = "idle"), 5000);
    }
  }

  function close() {
    dispatch("close");
  }
</script>

<div class="settings-panel">
  <div class="settings-header">
    <h2>Settings</h2>
    <button class="close-btn" onclick={close}>✕</button>
  </div>

  <div class="settings-content">
    <section class="setting-group">
      <h3>Vault Path</h3>
      <div class="input-row">
        <input type="text" bind:value={localVaultPath} placeholder="/path/to/vault" />
        <button class="btn-primary" onclick={updateVaultPath}>Set</button>
      </div>
      <button class="btn-browse" onclick={browseFolder}>Browse...</button>
      {#if $settings.vaultPath}
        <p class="current-value">Current: {$settings.vaultPath}</p>
      {/if}
    </section>

    <section class="setting-group">
      <h3>Theme</h3>
      <div class="theme-toggle">
        <button class:active={$settings.theme === "light"} onclick={() => updateTheme("light")}>
          ☀️ Light
        </button>
        <button class:active={$settings.theme === "dark"} onclick={() => updateTheme("dark")}>
          🌙 Dark
        </button>
      </div>
    </section>

    <section class="setting-group">
      <h3>Editor Font Size</h3>
      <div class="font-size-control">
        <input
          type="range"
          min="12"
          max="24"
          step="1"
          bind:value={localFontSize}
          oninput={() => updateFontSize(localFontSize)}
        />
        <span class="font-size-value">{localFontSize}px</span>
      </div>
    </section>

    <section class="setting-group">
      <h3>AI Model (LLM)</h3>
      <div class="llm-field">
        <label>API Base URL</label>
        <input type="text" bind:value={localBaseUrl} placeholder="https://api.openai.com/v1" />
      </div>
      <div class="llm-field">
        <label>API Key</label>
        <input type="password" bind:value={localApiKey} placeholder="sk-..." />
      </div>
      <div class="llm-field">
        <label>Model</label>
        <input type="text" bind:value={localModel} placeholder="gpt-4o-mini" />
      </div>
      <button class="btn-primary" onclick={saveLlmConfig}>Save LLM Config</button>
      <p class="current-value">
        {$settings.llm.baseUrl
          ? `Connected: ${$settings.llm.model} @ ${$settings.llm.baseUrl}`
          : "Not configured — Chat uses search-only mode"}
      </p>
    </section>

    <section class="setting-group">
      <h3>Indexing</h3>
      <button
        class="btn-reindex"
        onclick={triggerReindex}
        disabled={reindexStatus === "running" || !$settings.vaultPath}
      >
        {reindexStatus === "running" ? "Indexing..." : "Reindex Vault"}
      </button>
      {#if reindexResult}
        <p class="reindex-result">{reindexResult}</p>
      {/if}
    </section>

    {#if saveStatus === "saved"}
      <p class="status saved">✓ Settings saved</p>
    {:else if saveStatus === "error"}
      <p class="status error">✕ Failed to save settings</p>
    {/if}
  </div>
</div>

<style>
  .settings-panel {
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-color);
    width: 320px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1.1rem;
    padding: 0.25rem;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .setting-group {
    margin-bottom: 1.5rem;
  }

  .setting-group h3 {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-row input {
    flex: 1;
    padding: 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 0.85rem;
    outline: none;
    box-sizing: border-box;
  }

  .input-row input:focus {
    border-color: var(--accent-color);
  }

  .btn-primary {
    padding: 0.5rem 1rem;
    background: var(--accent-color);
    border: none;
    color: #fff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-browse {
    width: 100%;
    margin-top: 0.5rem;
    padding: 0.5rem;
    background: var(--bg-primary);
    border: 1px dashed var(--border-color);
    border-radius: 6px;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.15s;
  }

  .btn-browse:hover {
    border-color: var(--accent-color);
    color: var(--accent-color);
  }

  .current-value {
    margin: 0.5rem 0 0;
    font-size: 0.8rem;
    color: var(--text-tertiary);
    font-family: monospace;
  }

  .theme-toggle {
    display: flex;
    gap: 0.5rem;
  }

  .theme-toggle button {
    flex: 1;
    padding: 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.15s;
  }

  .theme-toggle button.active {
    background: var(--accent-color);
    border-color: var(--accent-color);
    color: #fff;
  }

  .font-size-control {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .font-size-control input[type="range"] {
    flex: 1;
  }

  .font-size-value {
    min-width: 3rem;
    text-align: right;
    font-family: monospace;
    color: var(--text-secondary);
  }

  .llm-field {
    margin-bottom: 0.5rem;
  }

  .llm-field label {
    display: block;
    font-size: 0.8rem;
    color: var(--text-secondary);
    margin-bottom: 0.2rem;
  }

  .llm-field input {
    width: 100%;
    padding: 0.4rem 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 0.85rem;
    outline: none;
    box-sizing: border-box;
  }

  .llm-field input:focus {
    border-color: var(--accent-color);
  }

  .btn-reindex {
    width: 100%;
    padding: 0.5rem;
    background: var(--accent-color);
    border: none;
    color: #fff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    transition: all 0.15s;
  }

  .btn-reindex:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-reindex:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .reindex-result {
    margin: 0.5rem 0 0;
    font-size: 0.8rem;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .status {
    padding: 0.5rem;
    border-radius: 6px;
    font-size: 0.85rem;
    text-align: center;
  }

  .status.saved {
    background: var(--success-bg);
    color: var(--success-color);
  }

  .status.error {
    background: var(--error-bg);
    color: var(--error-color);
  }
</style>
