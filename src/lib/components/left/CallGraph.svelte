<script lang="ts">
  import { graphStratify, sugiyama } from 'd3-dag';
  import { Svelvet, Node } from 'svelvet';
  import { appState } from '$lib/store.svelte';
  import type { FnId } from '$lib/types';

  function fnLabel(fn: FnId): string {
    const stem = fn.file.split('/').pop()?.replace(/\.[^/.]+$/, '') ?? fn.file;
    return `${stem}::${fn.name}`;
  }

  function fnKey(fn: FnId): string {
    return `${fn.file}::${fn.name}`;
  }

  const changedKeys = $derived(
    new Set((appState.analysis?.metrics ?? []).map((m) => `${m.file}::${m.name}`))
  );

  // Svelvet's CSSColorString type doesn't accept var(...) — cast helper
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function cssColor(s: string): any { return s; }

  const nodeW = 160;
  const nodeH = 40;
  const hGap = 40;
  const vGap = 80;
  const padX = 24;
  const padY = 24;

  type NodeDatum = { id: string; parentIds: string[] };

  const layout = $derived.by(() => {
    const graph = appState.analysis?.call_graph;
    if (!graph || graph.nodes.length === 0) return null;

    const nodes = graph.nodes;
    const edges = graph.edges;

    // ── 1. Stable numeric IDs (1-based; 0 is falsy in Svelvet) ──────────
    const idMap = new Map<string, number>();
    nodes.forEach((n, i) => idMap.set(fnKey(n), i + 1));

    // ── 2. Svelvet connections map (caller → callee numeric IDs) ─────────
    const connections = new Map<string, number[]>();
    for (const n of nodes) connections.set(fnKey(n), []);
    for (const [caller, callees] of edges) {
      const ck = fnKey(caller);
      for (const callee of callees) {
        const tid = idMap.get(fnKey(callee));
        if (tid !== undefined) connections.get(ck)!.push(tid);
      }
    }

    // ── 3. Predecessor map (callers = "parents" in the DAG) ──────────────
    const pred = new Map<string, string[]>();
    for (const n of nodes) pred.set(fnKey(n), []);
    for (const [caller, callees] of edges) {
      const ck = fnKey(caller);
      for (const callee of callees) pred.get(fnKey(callee))?.push(ck);
    }

    // ── 4. d3-dag Sugiyama layout ─────────────────────────────────────────
    // graphStratify supports isolated nodes (parentIds: []) unlike graphConnect.
    // nodeSize is the node bounding box; gap adds spacing between boxes.
    // node.x / node.y are CENTER coordinates — subtract half-dimensions for Svelvet.
    const dagData: NodeDatum[] = nodes.map(n => ({
      id: fnKey(n),
      parentIds: pred.get(fnKey(n)) ?? [],
    }));

    let positions: Map<string, { x: number; y: number }>;

    try {
      const dag = graphStratify()
        .id((d: NodeDatum) => d.id)
        .parentIds((d: NodeDatum) => d.parentIds)(dagData);

      sugiyama().nodeSize([nodeW, nodeH]).gap([hGap, vGap])(dag);

      positions = new Map();
      for (const node of dag.nodes()) {
        positions.set((node.data as NodeDatum).id, {
          x: padX + node.x - nodeW / 2,
          y: padY + node.y - nodeH / 2,
        });
      }
    } catch {
      // Fallback grid if d3-dag rejects the graph (e.g. cycle)
      positions = new Map();
      nodes.forEach((n, i) => {
        positions.set(fnKey(n), {
          x: padX + (i % 5) * (nodeW + hGap),
          y: padY + Math.floor(i / 5) * (nodeH + vGap),
        });
      });
    }

    return { nodes, positions, connections, idMap };
  });
</script>

<div class="graph-section">
  <div class="section-header">
    <h3 class="section-title">Call Graph</h3>
    <span class="material-symbols-outlined refresh-icon">refresh</span>
  </div>
  <div class="graph-container">
    {#if !layout}
      <!-- Empty state: animated abstract graph -->
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
      {#key layout}
        <Svelvet TD theme="dark" editable={false}>
          {#each layout.nodes as node (fnKey(node))}
            {@const pos = layout.positions.get(fnKey(node))!}
            {@const isChanged = changedKeys.has(fnKey(node))}
            {@const nodeId = layout.idMap.get(fnKey(node))!}
            {@const calleeIds = layout.connections.get(fnKey(node)) ?? []}
            <Node
              id={nodeId}
              label={fnLabel(node)}
              position={{ x: pos.x, y: pos.y }}
              connections={calleeIds}
              bgColor={cssColor(isChanged ? 'var(--color-primary-container)' : 'var(--color-surface-container-highest)')}
              textColor={cssColor(isChanged ? 'var(--color-primary)' : 'var(--color-on-surface-variant)')}
              borderColor={cssColor(isChanged ? 'var(--color-primary-dim)' : 'var(--color-outline-variant)')}
              width={nodeW}
              height={nodeH}
              inputs={1}
              outputs={1}
              editable={false}
            />
          {/each}
        </Svelvet>
      {/key}
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

  .graph-container :global(.svelvet-wrapper) {
    width: 100%;
    height: 100%;
    background: transparent;
  }

  .bg-svg {
    position: absolute;
    inset: 0;
    opacity: 0.4;
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
</style>
