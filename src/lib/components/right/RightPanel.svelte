<script lang="ts">
  import TabBar from './TabBar.svelte';
  import DiffView from './DiffView.svelte';
  import { appState } from '$lib/store.svelte';
</script>

<section class="right-panel">
  {#if appState.openTabs.length > 0}
    <TabBar />
    <DiffView />
  {:else}
    <div class="empty-panel">
      <span class="material-symbols-outlined empty-icon" style="font-variation-settings:'FILL' 1,'wght' 300,'GRAD' 0,'opsz' 48">code_blocks</span>
      <p class="empty-title">No files open</p>
      <p class="empty-subtitle">Select a function from the left panel to view its diff</p>
    </div>
  {/if}
</section>

<style>
  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    overflow: hidden;
  }

  .empty-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 0.75rem;
    color: var(--color-on-surface-variant);
    text-align: center;
    padding: 2rem;
  }

  .empty-icon {
    font-size: 48px;
    color: var(--color-outline-variant);
    margin-bottom: 0.5rem;
  }

  .empty-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-on-surface-variant);
    font-family: var(--font-body);
  }

  .empty-subtitle {
    font-size: 12px;
    color: var(--color-outline);
    font-family: var(--font-body);
    max-width: 300px;
  }
</style>
