<script lang="ts">
  import { settings } from "$lib/stores/settings";
  import { createEventDispatcher } from "svelte";
  import { set_vault_path } from "$lib/stores/vault-actions";

  const dispatch = createEventDispatcher();

  let localVaultPath = $state($settings.vaultPath || "");
  let localFontSize = $state($settings.editorFontSize);
  let saveStatus = $state<"idle" | "saved" | "error">("idle");

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
      saveStatus = "saved";
      setTimeout(() => (saveStatus = "idle"), 2000);
    } catch {
      saveStatus = "error";
      setTimeout(() => (saveStatus = "idle"), 3000);
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
        <button onclick={updateVaultPath}>Set</button>
      </div>
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

    {#if saveStatus === "saved"}
      <p class="status saved">✓ Settings saved</p>
    {:else if saveStatus === "error"}
      <p class="status error">✕ Failed to save settings</p>
    {/if}
  </div>
</div>

<style>
  .settings-panel {
    background: #1a2332;
    border-left: 1px solid #2d3f50;
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
    border-bottom: 1px solid #2d3f50;
  }

  .settings-header h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: #8899a6;
    cursor: pointer;
    font-size: 1.1rem;
    padding: 0.25rem;
  }

  .close-btn:hover {
    color: #e7e9ea;
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
    color: #8899a6;
  }

  .input-row {
    display: flex;
    gap: 0.5rem;
  }

  .input-row input {
    flex: 1;
    padding: 0.5rem;
    background: #0f1419;
    border: 1px solid #2d3f50;
    border-radius: 6px;
    color: #e7e9ea;
    font-size: 0.85rem;
    outline: none;
    box-sizing: border-box;
  }

  .input-row input:focus {
    border-color: #2e86c1;
  }

  .input-row button {
    padding: 0.5rem 1rem;
    background: #2e86c1;
    border: none;
    color: #fff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
  }

  .input-row button:hover {
    background: #2471a3;
  }

  .current-value {
    margin: 0.5rem 0 0;
    font-size: 0.8rem;
    color: #556677;
    font-family: monospace;
  }

  .theme-toggle {
    display: flex;
    gap: 0.5rem;
  }

  .theme-toggle button {
    flex: 1;
    padding: 0.5rem;
    background: #0f1419;
    border: 1px solid #2d3f50;
    border-radius: 6px;
    color: #e7e9ea;
    cursor: pointer;
    font-size: 0.85rem;
    transition: all 0.15s;
  }

  .theme-toggle button.active {
    background: #2e86c1;
    border-color: #2e86c1;
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
    color: #8899a6;
  }

  .status {
    padding: 0.5rem;
    border-radius: 6px;
    font-size: 0.85rem;
    text-align: center;
  }

  .status.saved {
    background: #1a3a1a;
    color: #27ae60;
  }

  .status.error {
    background: #3a1a1a;
    color: #e74c3c;
  }
</style>
