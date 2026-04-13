<script lang="ts">
  import TopMenuBar from './TopMenuBar.svelte';
  import SideNavBar from './SideNavBar.svelte';
  import StatusBar from './StatusBar.svelte';
  import LeftPanel from './left/LeftPanel.svelte';
  import RightPanel from './right/RightPanel.svelte';
  import { appState } from '$lib/store.svelte';
</script>

<div class="shell">
  <TopMenuBar />
  <div class="workspace">
    <SideNavBar />
    <main class="main-area">
      {#if appState.loading}
        <div class="loading-overlay">
          <span class="material-symbols-outlined spin">sync</span>
          <p>Analysing repository…</p>
        </div>
      {:else if appState.error}
        <div class="error-overlay">
          <span class="material-symbols-outlined">error</span>
          <p>{appState.error}</p>
        </div>
      {:else}
        <LeftPanel />
        <RightPanel />
      {/if}
    </main>
  </div>
  <StatusBar />
</div>

<style>
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--color-background);
  }

  .workspace {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-area {
    display: flex;
    flex: 1;
    overflow: hidden;
    background: var(--color-background);
    position: relative;
  }

  .loading-overlay,
  .error-overlay {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    gap: 0.5rem;
    color: var(--color-on-surface-variant);
  }

  .loading-overlay .material-symbols-outlined,
  .error-overlay .material-symbols-outlined {
    font-size: 32px;
    color: var(--color-primary);
  }

  .error-overlay .material-symbols-outlined {
    color: var(--color-error);
  }

  .loading-overlay p,
  .error-overlay p {
    font-size: 13px;
    color: var(--color-on-surface-variant);
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .spin {
    animation: spin 1.5s linear infinite;
  }
</style>
