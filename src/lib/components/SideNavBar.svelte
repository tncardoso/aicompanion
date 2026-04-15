<script lang="ts">
  const topIcons = [
    { icon: 'home', label: 'Home', active: true },
  ];

  const bottomIcons = [
    { icon: 'settings', label: 'Settings' },
  ];
</script>

<aside class="side-nav">
  <div class="top-icons">
    {#each topIcons as item}
      <div
        class="nav-icon"
        class:active={item.active}
        class:tertiary={item.tertiary}
        title={item.label}
      >
        <span
          class="material-symbols-outlined"
          style={item.filled ? "font-variation-settings: 'FILL' 1, 'wght' 400, 'GRAD' 0, 'opsz' 24" : ''}
        >{item.icon}</span>
      </div>
    {/each}
  </div>
  <div class="bottom-icons">
    {#each bottomIcons as item}
      <div class="nav-icon" title={item.label}>
        <span class="material-symbols-outlined">{item.icon}</span>
      </div>
    {/each}
  </div>
</aside>

<style>
  .side-nav {
    width: 64px;
    background: var(--color-surface-container-low);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem 0;
    flex-shrink: 0;
    z-index: 50;
  }

  .top-icons {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    width: 100%;
    align-items: center;
    flex: 1;
  }

  .bottom-icons {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 100%;
    align-items: center;
  }

  .nav-icon {
    width: 100%;
    display: flex;
    justify-content: center;
    padding: 8px 0;
    color: var(--color-secondary);
    opacity: 0.6;
    cursor: pointer;
    transition: opacity 200ms, background 200ms;
  }

  .nav-icon:hover {
    opacity: 1;
    background: var(--color-surface-container);
  }

  .nav-icon.active {
    opacity: 1;
    color: var(--color-primary);
    background: var(--color-surface-container);
    border-left: 2px solid var(--color-primary);
  }

  .nav-icon.tertiary {
    color: var(--color-tertiary);
  }

  .material-symbols-outlined {
    font-size: 22px;
  }
</style>
