<script lang="ts">
  import { appState } from '$lib/store.svelte';

  const menuItems = ['File', 'Edit'];
  let activeMenu = $state('File');
</script>

<header class="top-menu-bar">
  <div class="left">
    <span class="brand">AI Companion</span>
    <nav class="menu-nav">
      {#each menuItems as item}
        <button
          class="menu-item"
          class:active={activeMenu === item}
          onclick={() => (activeMenu = item)}
        >
          {item}
        </button>
      {/each}
    </nav>
  </div>
  <!--
  <div class="right">
    <div class="icon-group">
      <span class="material-symbols-outlined icon">account_tree</span>
      <span class="material-symbols-outlined icon">terminal</span>
      <span class="material-symbols-outlined icon filled">psychology</span>
    </div>
    <button class="deploy-btn">Deploy</button>
  </div>
  -->
</header>

<style>
  .top-menu-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 0 1rem;
    height: 48px;
    background: #0c0e12;
    border-bottom: 1px solid rgba(66, 72, 84, 0.15);
    flex-shrink: 0;
    z-index: 50;
  }

  .left {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }

  .brand {
    font-size: 18px;
    font-weight: 900;
    color: var(--color-primary);
    letter-spacing: -0.05em;
  }

  .menu-nav {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .menu-item {
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    letter-spacing: -0.02em;
    color: var(--color-secondary);
    padding-bottom: 4px;
    transition: color 150ms;
  }

  .menu-item:hover {
    color: var(--color-on-surface);
  }

  .menu-item.active {
    color: var(--color-primary);
    border-bottom: 2px solid var(--color-primary);
  }

  .right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .icon-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-right: 1rem;
  }

  .icon {
    font-size: 18px;
    color: var(--color-secondary);
    cursor: pointer;
  }

  .icon.filled {
    font-variation-settings: 'FILL' 1, 'wght' 400, 'GRAD' 0, 'opsz' 24;
    color: var(--color-primary);
  }

  .deploy-btn {
    background: var(--color-primary);
    color: var(--color-on-primary);
    border: none;
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    transition: transform 100ms ease-in-out;
  }

  .deploy-btn:hover {
    transform: scale(0.95);
  }
</style>
