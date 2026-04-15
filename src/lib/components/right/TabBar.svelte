<script lang="ts">
  import { appState } from '$lib/store.svelte';

  function closeTab(path: string, e: MouseEvent) {
    e.stopPropagation();
    appState.openTabs = appState.openTabs.filter((t) => t !== path);
    if (appState.activeTab === path) {
      appState.activeTab = appState.openTabs[0] ?? null;
    }
  }

  function selectTab(path: string) {
    appState.activeTab = path;
  }

  function fileIcon(path: string): { icon: string; color: string } {
    const ext = path.split('.').pop() ?? '';
    const map: Record<string, { icon: string; color: string }> = {
      py: { icon: 'description', color: '#42b883' },
      rs: { icon: 'description', color: '#f46623' },
      ts: { icon: 'description', color: '#3178c6' },
      js: { icon: 'description', color: '#f7df1e' },
      go: { icon: 'description', color: '#00add8' },
      svelte: { icon: 'description', color: '#ff3e00' },
      json: { icon: 'data_object', color: '#c890ff' },
      toml: { icon: 'settings', color: '#909fb5' },
    };
    return map[ext] ?? { icon: 'description', color: '#909fb5' };
  }

  function shortName(path: string): string {
    return path.split('/').pop() ?? path;
  }
</script>

<div class="tab-bar">
  {#each appState.openTabs as tab (tab)}
    {@const active = tab === appState.activeTab}
    {@const fi = fileIcon(tab)}
    <div
      class="tab"
      class:active
      role="button"
      tabindex="0"
      onclick={() => selectTab(tab)}
      onkeydown={(e) => e.key === 'Enter' && selectTab(tab)}
    >
      {#if active}
        <div class="tab-indicator"></div>
      {/if}
      <span class="material-symbols-outlined tab-icon" style="color: {fi.color}">{fi.icon}</span>
      <span class="tab-name">{shortName(tab)}</span>
      <button
        class="close-btn"
        onclick={(e) => closeTab(tab, e)}
        title="Close"
      >
        <span class="material-symbols-outlined">close</span>
      </button>
    </div>
  {/each}
</div>

<style>
  .tab-bar {
    display: flex;
    flex-wrap: wrap;
    background: var(--color-surface-container);
    flex-shrink: 0;
  }

  .tab {
    display: flex;
    align-items: center;
    padding: 0 1rem;
    min-width: 140px;
    height: 40px;
    position: relative;
    cursor: pointer;
    opacity: 0.6;
    transition: opacity 200ms, background 200ms;
    gap: 6px;
    flex-shrink: 0;
  }

  .tab:hover {
    opacity: 1;
  }

  .tab.active {
    background: var(--color-surface-bright);
    opacity: 1;
  }

  .tab-indicator {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--color-primary);
  }

  .tab-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .tab-name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-on-surface);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-secondary-dim);
    display: flex;
    align-items: center;
    padding: 0;
    margin-left: auto;
    flex-shrink: 0;
    transition: color 150ms;
  }

  .close-btn:hover {
    color: var(--color-on-surface);
  }

  .close-btn .material-symbols-outlined {
    font-size: 12px;
  }

  .tab-spacer {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 0 1rem;
  }

  .ai-explain-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--color-tertiary);
    background: rgba(200, 144, 255, 0.1);
    border: 1px solid rgba(200, 144, 255, 0.2);
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 150ms;
    white-space: nowrap;
  }

  .ai-explain-btn:hover {
    background: rgba(200, 144, 255, 0.2);
  }

  .btn-icon {
    font-size: 14px;
  }

  .btn-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
</style>
