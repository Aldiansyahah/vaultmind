<script lang="ts">
  import { onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  interface GraphNode {
    path: string;
    title: string;
    tags: string[];
    x?: number;
    y?: number;
    vx?: number;
    vy?: number;
  }

  interface GraphEdge {
    source: string;
    target: string;
    kind: string;
  }

  let canvas = $state<HTMLCanvasElement | null>(null);
  let nodes = $state<GraphNode[]>([]);
  let edges = $state<GraphEdge[]>([]);
  let isLoading = $state(true);
  let animFrame: number | null = null;
  let width = 0;
  let height = 0;

  async function loadGraph() {
    isLoading = true;
    try {
      const data: { nodes: GraphNode[]; edges: GraphEdge[] } = await invoke("get_graph_data");
      nodes = data.nodes;
      edges = data.edges;
      initPositions();
      startSimulation();
    } catch (e) {
      console.error("Failed to load graph:", e);
      nodes = [];
      edges = [];
    } finally {
      isLoading = false;
    }
  }

  function initPositions() {
    const cx = width / 2 || 300;
    const cy = height / 2 || 250;
    for (const node of nodes) {
      node.x = cx + (Math.random() - 0.5) * 300;
      node.y = cy + (Math.random() - 0.5) * 300;
      node.vx = 0;
      node.vy = 0;
    }
  }

  function startSimulation() {
    let iterations = 0;
    const maxIterations = 200;

    function tick() {
      if (iterations >= maxIterations) {
        draw();
        return;
      }
      iterations++;
      simulate();
      draw();
      animFrame = requestAnimationFrame(tick);
    }

    tick();
  }

  function simulate() {
    const nodeMap = new Map(nodes.map((n) => [n.path, n]));
    const repulsion = 2000;
    const attraction = 0.01;
    const damping = 0.9;

    // Repulsion between all nodes
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i];
        const b = nodes[j];
        const dx = (a.x || 0) - (b.x || 0);
        const dy = (a.y || 0) - (b.y || 0);
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const force = repulsion / (dist * dist);
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        a.vx = (a.vx || 0) + fx;
        a.vy = (a.vy || 0) + fy;
        b.vx = (b.vx || 0) - fx;
        b.vy = (b.vy || 0) - fy;
      }
    }

    // Attraction along edges
    for (const edge of edges) {
      const a = nodeMap.get(edge.source);
      const b = nodeMap.get(edge.target);
      if (!a || !b) continue;
      const dx = (b.x || 0) - (a.x || 0);
      const dy = (b.y || 0) - (a.y || 0);
      const fx = dx * attraction;
      const fy = dy * attraction;
      a.vx = (a.vx || 0) + fx;
      a.vy = (a.vy || 0) + fy;
      b.vx = (b.vx || 0) - fx;
      b.vy = (b.vy || 0) - fy;
    }

    // Apply velocity with damping
    for (const node of nodes) {
      node.vx = (node.vx || 0) * damping;
      node.vy = (node.vy || 0) * damping;
      node.x = (node.x || 0) + (node.vx || 0);
      node.y = (node.y || 0) + (node.vy || 0);

      // Keep within bounds
      node.x = Math.max(30, Math.min(width - 30, node.x || 0));
      node.y = Math.max(30, Math.min(height - 30, node.y || 0));
    }
  }

  function draw() {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const nodeMap = new Map(nodes.map((n) => [n.path, n]));
    const isDark = document.documentElement.getAttribute("data-theme") !== "light";

    ctx.clearRect(0, 0, width, height);

    // Draw edges
    ctx.strokeStyle = isDark ? "#2d3f50" : "#e3ddd3";
    ctx.lineWidth = 1;
    for (const edge of edges) {
      const a = nodeMap.get(edge.source);
      const b = nodeMap.get(edge.target);
      if (!a || !b) continue;
      ctx.beginPath();
      ctx.moveTo(a.x || 0, a.y || 0);
      ctx.lineTo(b.x || 0, b.y || 0);
      ctx.stroke();
    }

    // Draw nodes
    for (const node of nodes) {
      ctx.beginPath();
      ctx.arc(node.x || 0, node.y || 0, 8, 0, Math.PI * 2);
      ctx.fillStyle = isDark ? "#2e86c1" : "#45413c";
      ctx.fill();
      ctx.strokeStyle = isDark ? "#1a2332" : "#faf8f5";
      ctx.lineWidth = 2;
      ctx.stroke();

      // Label
      ctx.fillStyle = isDark ? "#e7e9ea" : "#37352f";
      ctx.font = "11px -apple-system, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(node.title || node.path, node.x || 0, (node.y || 0) + 20);
    }
  }

  function handleClick(e: MouseEvent) {
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;

    for (const node of nodes) {
      const dx = (node.x || 0) - mx;
      const dy = (node.y || 0) - my;
      if (dx * dx + dy * dy < 100) {
        dispatch("open-note", { path: node.path });
        break;
      }
    }
  }

  $effect(() => {
    if (canvas) {
      width = canvas.parentElement?.clientWidth || 600;
      height = canvas.parentElement?.clientHeight || 500;
      canvas.width = width;
      canvas.height = height;
      loadGraph();
    }
  });

  onDestroy(() => {
    if (animFrame) cancelAnimationFrame(animFrame);
  });
</script>

<div class="graph-view">
  <div class="graph-header">
    <h3>Knowledge Graph</h3>
    <div class="graph-actions">
      <button class="refresh-btn" onclick={loadGraph}>Refresh</button>
      <button class="close-btn" onclick={() => dispatch("close")}>✕</button>
    </div>
  </div>

  <div class="graph-canvas-wrapper">
    {#if isLoading}
      <div class="graph-loading">Loading graph...</div>
    {:else if nodes.length === 0}
      <div class="graph-empty">
        <p>No notes in the graph yet.</p>
        <p class="hint">Create some notes and reindex to see connections.</p>
      </div>
    {:else}
      <canvas bind:this={canvas} onclick={handleClick}></canvas>
      <div class="graph-stats">
        {nodes.length} notes, {edges.length} connections
      </div>
    {/if}
  </div>
</div>

<style>
  .graph-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .graph-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .graph-header h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .graph-actions {
    display: flex;
    gap: 0.5rem;
  }

  .refresh-btn {
    padding: 0.3rem 0.6rem;
    background: var(--accent-color);
    border: none;
    color: #fff;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1rem;
  }

  .graph-canvas-wrapper {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  canvas {
    display: block;
    cursor: pointer;
  }

  .graph-loading,
  .graph-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
  }

  .graph-empty p {
    margin: 0.25rem 0;
  }

  .hint {
    font-size: 0.8rem;
  }

  .graph-stats {
    position: absolute;
    bottom: 0.5rem;
    right: 0.75rem;
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }
</style>
