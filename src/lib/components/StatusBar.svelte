<script lang="ts">
  import { appState } from '$lib/store.svelte';

  const warningCount = $derived.by(() => {
    const metrics = appState.analysis?.metrics;
    if (!metrics) return 0;
    const t = appState.config?.thresholds ?? { cyclomatic: 10, cognitive: 15, coupling: 5 };
    return metrics.filter((m) =>
      m.cyclomatic > t.cyclomatic || m.cognitive > t.cognitive || m.coupling > t.coupling
    ).length;
  });

  const activeFileExt = $derived(() => {
    if (!appState.activeTab) return 'Plain Text';
    const ext = appState.activeTab.split('.').pop() ?? '';
    const map: Record<string, string> = {
      rs: 'Rust', py: 'Python', ts: 'TypeScript', js: 'JavaScript',
      go: 'Go', svelte: 'Svelte', json: 'JSON', toml: 'TOML',
    };
    return map[ext] ?? (ext.toUpperCase() || 'Plain Text');
  });
</script>

<footer class="status-bar">
  <div class="left">
    <div class="status-item">
      <span class="material-symbols-outlined status-icon primary">source_environment</span>
      <span class="status-text">
        {#if appState.loading}loading…{:else}main*{/if}
      </span>
    </div>
    {#if appState.gitState?.repo_root}
      <div class="status-item">
        <span class="material-symbols-outlined status-icon secondary">folder_open</span>
        <span class="status-text cwd" title={appState.gitState?.repo_root}>{appState.gitState?.repo_root}</span>
      </div>
    {/if}
    <div class="status-item">
      <span class="material-symbols-outlined status-icon error">error</span>
      <span class="status-text">0</span>
      <span class="material-symbols-outlined status-icon secondary">warning</span>
      <span class="status-text">{warningCount}</span>
    </div>
  </div>
  <div class="right">
    <span class="status-text hoverable">{activeFileExt()}</span>
    <span class="status-text primary">v0.1</span>
  </div>
</footer>

<style>
  .status-bar {
    height: 24px;
    background: #000000;
    border-top: 1px solid rgba(66, 72, 84, 0.1);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 12px;
    width: 100%;
    flex-shrink: 0;
    z-index: 50;
  }

  .left,
  .right {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
    opacity: 0.8;
    cursor: pointer;
  }

  .status-item:hover {
    opacity: 1;
  }

  .status-icon {
    font-size: 12px;
  }

  .status-icon.primary { color: var(--color-primary); }
  .status-icon.error { color: var(--color-error); }
  .status-icon.secondary { color: var(--color-secondary); }

  .status-text {
    font-family: var(--font-body);
    font-size: 10px;
    font-weight: 500;
    color: var(--color-secondary);
  }

  .status-text.primary { color: var(--color-primary); }

  .status-text.cwd {
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }

  .status-text.hoverable:hover {
    color: var(--color-on-surface);
    cursor: pointer;
  }
</style>
