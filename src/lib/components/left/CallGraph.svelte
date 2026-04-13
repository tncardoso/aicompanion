<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import type { FnId } from '$lib/types';

  let containerEl = $state<HTMLDivElement | undefined>(undefined);
  let width = $state(400);
  let height = $state(300);

  // Label helper matching Rust's FnId::label()
  function fnLabel(fn: FnId): string {
    const stem = fn.file.split('/').pop()?.replace(/\.[^/.]+$/, '') ?? fn.file;
    return `${stem}::${fn.name}`;
  }

  function fnKey(fn: FnId): string {
    return `${fn.file}::${fn.name}`;
  }

  // Set of changed function keys (from metrics)
  const changedKeys = $derived(
    new Set((appState.analysis?.metrics ?? []).map((m) => `${m.file}::${m.name}`))
  );

  // Compute SVG layout using layered/Sugiyama approach
  const layout = $derived.by(() => {
    const graph = appState.analysis?.call_graph;
    if (!graph || graph.nodes.length === 0) return null;

    const nodes = graph.nodes;
    const edges = graph.edges; // Array<[FnId, FnId[]]>

    // Build adjacency map
    const adj = new Map<string, string[]>();
    for (const [caller, callees] of edges) {
      const ck = fnKey(caller);
      if (!adj.has(ck)) adj.set(ck, []);
      for (const callee of callees) {
        adj.get(ck)!.push(fnKey(callee));
      }
    }

    // Compute in-degree
    const inDeg = new Map<string, number>();
    for (const n of nodes) inDeg.set(fnKey(n), 0);
    for (const [, callees] of edges) {
      for (const c of callees) {
        const ck = fnKey(c);
        inDeg.set(ck, (inDeg.get(ck) ?? 0) + 1);
      }
    }

    // BFS layer assignment (longest path from any source)
    const layer = new Map<string, number>();
    const queue: string[] = [];
    for (const n of nodes) {
      if ((inDeg.get(fnKey(n)) ?? 0) === 0) {
        layer.set(fnKey(n), 0);
        queue.push(fnKey(n));
      }
    }

    const visited = new Set<string>();
    let qi = 0;
    while (qi < queue.length) {
      const nk = queue[qi++];
      if (visited.has(nk)) continue;
      visited.add(nk);
      const targets = adj.get(nk) ?? [];
      for (const tk of targets) {
        const newLayer = (layer.get(nk) ?? 0) + 1;
        if (newLayer > (layer.get(tk) ?? 0)) {
          layer.set(tk, newLayer);
        }
        if (!visited.has(tk)) queue.push(tk);
      }
    }
    // Nodes without a layer get layer 0
    for (const n of nodes) {
      if (!layer.has(fnKey(n))) layer.set(fnKey(n), 0);
    }

    // Group by layer
    const byLayer = new Map<number, FnId[]>();
    for (const n of nodes) {
      const l = layer.get(fnKey(n)) ?? 0;
      if (!byLayer.has(l)) byLayer.set(l, []);
      byLayer.get(l)!.push(n);
    }

    const maxLayer = Math.max(...byLayer.keys());
    const padding = 20;
    const nodeW = 130;
    const nodeH = 28;
    const layerH = (height - padding * 2) / (maxLayer + 1);

    const positions = new Map<string, { x: number; y: number }>();
    for (const [l, layerNodes] of byLayer) {
      const slotW = (width - padding * 2) / (layerNodes.length + 1);
      layerNodes.forEach((n, i) => {
        positions.set(fnKey(n), {
          x: padding + slotW * (i + 1) - nodeW / 2,
          y: padding + layerH * l,
        });
      });
    }

    // Build edge paths (cubic bezier)
    const edgePaths: Array<{ d: string; key: string }> = [];
    for (const [caller, callees] of edges) {
      const ck = fnKey(caller);
      const cp = positions.get(ck);
      if (!cp) continue;
      for (const callee of callees) {
        const tk = fnKey(callee);
        const tp = positions.get(tk);
        if (!tp) continue;
        const x1 = cp.x + nodeW / 2;
        const y1 = cp.y + nodeH;
        const x2 = tp.x + nodeW / 2;
        const y2 = tp.y;
        const midY = (y1 + y2) / 2;
        edgePaths.push({
          key: `${ck}->${tk}`,
          d: `M ${x1},${y1} C ${x1},${midY} ${x2},${midY} ${x2},${y2}`,
        });
      }
    }

    return { nodes, positions, edgePaths, nodeW, nodeH };
  });

  // Resize observer
  $effect(() => {
    if (!containerEl) return;
    const ro = new ResizeObserver((entries) => {
      const entry = entries[0];
      width = entry.contentRect.width || 400;
      height = entry.contentRect.height || 300;
    });
    ro.observe(containerEl);
    return () => ro.disconnect();
  });
</script>

<div class="graph-section">
  <div class="section-header">
    <h3 class="section-title">Call Graph Visualization</h3>
    <span class="material-symbols-outlined refresh-icon">refresh</span>
  </div>
  <div class="graph-container" bind:this={containerEl}>
    {#if !layout}
      <!-- Empty state: animated abstract graph from design.html -->
      <svg class="bg-svg" width="100%" height="100%">
        <defs>
          <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style="stop-color:#bdc2ff;stop-opacity:1" />
            <stop offset="100%" style="stop-color:#c890ff;stop-opacity:1" />
          </linearGradient>
        </defs>
        <line stroke="#424854" stroke-width="1" x1="20%" y1="20%" x2="50%" y2="50%" />
        <line stroke="#424854" stroke-width="1" x1="50%" y1="50%" x2="80%" y2="30%" />
        <line stroke="#424854" stroke-width="1" x1="50%" y1="50%" x2="40%" y2="80%" />
        <circle cx="20%" cy="20%" r="4" fill="#bdc2ff" />
        <circle cx="50%" cy="50%" r="6" fill="#c890ff" />
        <circle cx="80%" cy="30%" r="4" fill="#bdc2ff" />
        <circle cx="40%" cy="80%" r="4" fill="#bdc2ff" />
      </svg>
      <div class="empty-state">
        <span class="material-symbols-outlined hub-icon" style="font-variation-settings:'FILL' 1,'wght' 400,'GRAD' 0,'opsz' 24">hub</span>
        <p class="empty-label">
          {#if appState.loading}Analysing…{:else}No changes detected{/if}
        </p>
      </div>
    {:else}
      <svg width={width} height={height} class="graph-svg">
        <defs>
          <marker id="arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
            <path d="M0,0 L0,6 L8,3 z" fill="var(--color-outline)" />
          </marker>
          <marker id="arrow-changed" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
            <path d="M0,0 L0,6 L8,3 z" fill="var(--color-primary-dim)" />
          </marker>
        </defs>

        <!-- Edges -->
        {#each layout.edgePaths as edge (edge.key)}
          <path
            d={edge.d}
            fill="none"
            stroke="var(--color-outline-variant)"
            stroke-width="1.5"
            marker-end="url(#arrow)"
          />
        {/each}

        <!-- Nodes -->
        {#each layout.nodes as node (fnKey(node))}
          {@const pos = layout.positions.get(fnKey(node))}
          {@const isChanged = changedKeys.has(fnKey(node))}
          {#if pos}
            <g class="node" role="button" tabindex="0">
              <rect
                x={pos.x}
                y={pos.y}
                width={layout.nodeW}
                height={layout.nodeH}
                rx="3"
                fill={isChanged ? 'var(--color-primary-container)' : 'var(--color-surface-container-highest)'}
                stroke={isChanged ? 'var(--color-primary-dim)' : 'var(--color-outline-variant)'}
                stroke-width="1"
              />
              <text
                x={pos.x + layout.nodeW / 2}
                y={pos.y + layout.nodeH / 2 + 4}
                text-anchor="middle"
                font-family="var(--font-mono)"
                font-size="9"
                fill={isChanged ? 'var(--color-primary)' : 'var(--color-on-surface-variant)'}
              >{fnLabel(node)}</text>
            </g>
          {/if}
        {/each}
      </svg>
    {/if}
  </div>
</div>

<style>
  .graph-section {
    display: flex;
    flex-direction: column;
    height: 60%;
    padding: 1rem;
    overflow: hidden;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
    flex-shrink: 0;
  }

  .section-title {
    font-size: 11px;
    font-family: var(--font-body);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-secondary-dim);
    font-weight: 700;
  }

  .refresh-icon {
    font-size: 14px;
    color: var(--color-secondary);
    cursor: pointer;
    transition: color 150ms;
  }

  .refresh-icon:hover {
    color: var(--color-on-surface);
  }

  .graph-container {
    flex: 1;
    background: var(--color-surface-container-low);
    border-radius: 2px;
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .bg-svg {
    position: absolute;
    inset: 0;
    opacity: 0.4;
  }

  .graph-svg {
    display: block;
    overflow: visible;
  }

  .empty-state {
    z-index: 10;
    text-align: center;
    pointer-events: none;
  }

  .hub-icon {
    font-size: 36px;
    color: var(--color-tertiary-dim);
    display: block;
    margin-bottom: 8px;
  }

  .empty-label {
    font-size: 10px;
    color: var(--color-secondary-dim);
    text-transform: uppercase;
    letter-spacing: -0.03em;
  }

  .node {
    cursor: pointer;
  }

  .node:hover rect {
    opacity: 0.8;
  }
</style>
