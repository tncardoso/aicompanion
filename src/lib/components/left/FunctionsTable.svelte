<script lang="ts">
  import { appState, nextSort, SORT_LABELS } from '$lib/store.svelte';
  import type { FunctionMetricsDelta } from '$lib/types';

  function getSortValue(m: FunctionMetricsDelta): number {
    switch (appState.sortOrder) {
      case 'cyclomatic_value': return m.cyclomatic;
      case 'cyclomatic_delta': return m.cyclomatic_delta ?? m.cyclomatic;
      case 'cognitive_value':  return m.cognitive;
      case 'cognitive_delta':  return m.cognitive_delta ?? m.cognitive;
      case 'coupling_value':   return m.coupling;
      case 'coupling_delta':   return m.coupling_delta ?? m.coupling;
    }
  }

  const sortedMetrics = $derived(
    [...(appState.analysis?.metrics ?? [])].sort((a, b) => getSortValue(b) - getSortValue(a))
  );

  function getStatus(m: FunctionMetricsDelta): 'added' | 'modified' {
    return m.cyclomatic_delta === null ? 'added' : 'modified';
  }

  function hasWarning(m: FunctionMetricsDelta): boolean {
    const t = appState.config?.thresholds ?? { cyclomatic: 10, cognitive: 15, coupling: 5 };
    return m.cyclomatic > t.cyclomatic || m.cognitive > t.cognitive || m.coupling > t.coupling;
  }

  function formatDelta(delta: number | null): string {
    if (delta === null) return '—';
    if (delta > 0) return `+${delta}`;
    return String(delta);
  }

  function openTab(file: string, line: number) {
    if (!appState.openTabs.includes(file)) {
      appState.openTabs = [...appState.openTabs, file];
    }
    appState.activeTab = file;
    appState.activeLine = line;
  }

  function shortName(file: string): string {
    return file.split('/').pop() ?? file;
  }
</script>

<div class="table-section">
  <div class="section-header">
    <h3 class="section-title">Functions Changed</h3>
    <button class="sort-btn" onclick={() => (appState.sortOrder = nextSort(appState.sortOrder))}>
      {SORT_LABELS[appState.sortOrder]}
    </button>
  </div>

  <div class="table-body">
    {#if appState.analysisError}
      <div class="empty-state error-state" title={appState.analysisError}>
        <span class="material-symbols-outlined">error</span>
        <p>Analysis failed</p>
        <p class="error-detail">{appState.analysisError}</p>
      </div>
    {:else if sortedMetrics.length === 0}
      <div class="empty-state">
        <p>No functions changed</p>
      </div>
    {:else}
      <table>
        <thead>
          <tr>
            <th class="th-fn">Function</th>
            <th class="th-metric">Cycl</th>
            <th class="th-metric">Cog</th>
            <th class="th-metric">Cpl</th>
            <th class="th-status">Status</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedMetrics as m (m.file + '::' + m.name)}
            {@const status = getStatus(m)}
            {@const t = appState.config?.thresholds ?? { cyclomatic: 10, cognitive: 15, coupling: 5 }}
            <tr
              class="fn-row"
              class:warn={hasWarning(m)}
              role="button"
              tabindex="0"
              onclick={() => openTab(m.file, m.line)}
              onkeydown={(e) => e.key === 'Enter' && openTab(m.file, m.line)}
            >
              <td class="td-fn">
                <span class="dot" class:dot-modified={status === 'modified'} class:dot-added={status === 'added'}></span>
                <span class="fn-name" title="{m.file}::{m.name}">{m.name}</span>
                <span class="fn-file">{shortName(m.file)}</span>
              </td>
              <td class="td-metric" class:metric-warn={m.cyclomatic > t.cyclomatic}>
                {m.cyclomatic} <span class="delta">({formatDelta(m.cyclomatic_delta)})</span>
              </td>
              <td class="td-metric" class:metric-warn={m.cognitive > t.cognitive}>
                {m.cognitive} <span class="delta">({formatDelta(m.cognitive_delta)})</span>
              </td>
              <td class="td-metric" class:metric-warn={m.coupling > t.coupling}>
                {m.coupling} <span class="delta">({formatDelta(m.coupling_delta)})</span>
              </td>
              <td class="td-status">
                <span class="badge" class:badge-modified={status === 'modified'} class:badge-added={status === 'added'}>
                  {status === 'modified' ? 'Modified' : 'Added'}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

<style>
  .table-section {
    display: flex;
    flex-direction: column;
    flex: 4;
    min-height: 0;
    padding: 1rem;
    background: var(--color-surface-container);
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

  .sort-btn {
    background: none;
    border: 1px solid rgba(193, 199, 206, 0.4);
    color: var(--color-secondary);
    font-size: 10px;
    font-family: var(--font-body);
    padding: 2px 8px;
    border-radius: 0.25rem;
    cursor: pointer;
    transition: all 150ms;
  }

  .sort-btn:hover {
    color: var(--color-on-surface);
    border-color: var(--color-outline);
  }

  .table-body {
    flex: 1;
    overflow-y: auto;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  thead th {
    font-size: 10px;
    font-family: var(--font-body);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-secondary-dim);
    font-weight: 700;
    padding: 0 8px 6px;
    border-bottom: 1px solid var(--color-outline-variant);
  }

  .th-fn { text-align: left; width: auto; }
  .th-metric { text-align: right; width: 80px; }
  .th-status { text-align: right; width: 72px; }

  .fn-row {
    cursor: pointer;
    transition: background 150ms;
  }

  .fn-row:hover td {
    background: var(--color-surface-container-high);
  }

  .fn-row:hover td:first-child { border-radius: 0.25rem 0 0 0.25rem; }
  .fn-row:hover td:last-child { border-radius: 0 0.25rem 0.25rem 0; }

  .fn-row.warn .fn-name {
    color: var(--color-error);
  }

  .td-fn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    min-width: 0;
  }

  .td-metric {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--color-on-surface-variant);
    text-align: right;
    padding: 5px 8px;
    white-space: nowrap;
  }

  .td-metric.metric-warn {
    color: var(--color-error-dim);
  }

  .td-status {
    text-align: right;
    padding: 5px 8px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .dot-modified { background: var(--color-primary); }
  .dot-added { background: var(--color-secondary); }

  .fn-name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .fn-file {
    font-size: 10px;
    color: var(--color-on-surface-variant);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .delta {
    color: var(--color-secondary);
  }

  .badge {
    font-size: 10px;
    padding: 1px 8px;
    border-radius: 0.25rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: -0.02em;
    white-space: nowrap;
  }

  .badge-modified {
    background: rgba(0, 97, 164, 0.12);
    color: var(--color-primary);
  }

  .badge-added {
    background: rgba(152, 70, 40, 0.12);
    color: var(--color-secondary);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-on-surface-variant);
    font-size: 12px;
    gap: 4px;
    text-align: center;
    padding: 0.5rem;
  }

  .error-state {
    color: var(--color-error-dim);
  }

  .error-state .material-symbols-outlined {
    font-size: 20px;
    color: var(--color-error);
  }

  .error-detail {
    font-size: 10px;
    color: var(--color-outline);
    word-break: break-all;
    max-height: 60px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
