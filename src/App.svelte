<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let version = $state("...");
  let status = $state("checking...");

  async function checkHealth() {
    try {
      const result: any = await invoke("health_check");
      version = result.version;
      status = result.status;
    } catch (e) {
      status = "error";
      console.error(e);
    }
  }

  $effect(() => {
    checkHealth();
  });
</script>

<main>
  <div class="container">
    <h1>VaultMind</h1>
    <p class="subtitle">RAG-optimized personal knowledge management</p>

    <div class="status-card">
      <div class="status-row">
        <span class="label">Status</span>
        <span class="value {status}">{status}</span>
      </div>
      <div class="status-row">
        <span class="label">Version</span>
        <span class="value">{version}</span>
      </div>
    </div>

    <p class="hint">
      All systems operational. Start building Phase 1 features.
    </p>
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: #0f1419;
    color: #e7e9ea;
  }

  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 2rem;
  }

  h1 {
    font-size: 2.5rem;
    font-weight: 700;
    margin: 0 0 0.5rem;
    background: linear-gradient(135deg, #2e86c1, #27ae60);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .subtitle {
    color: #8899a6;
    margin: 0 0 2rem;
    font-size: 1.1rem;
  }

  .status-card {
    background: #1a2332;
    border: 1px solid #2d3f50;
    border-radius: 12px;
    padding: 1.5rem 2rem;
    min-width: 300px;
  }

  .status-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
  }

  .status-row + .status-row {
    border-top: 1px solid #2d3f50;
  }

  .label {
    color: #8899a6;
  }

  .value {
    font-weight: 600;
  }

  .value.ok {
    color: #27ae60;
  }

  .value.error {
    color: #e74c3c;
  }

  .hint {
    margin-top: 2rem;
    color: #556677;
    font-size: 0.9rem;
  }
</style>
