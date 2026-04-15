<script lang="ts">
  import CallGraph from './CallGraph.svelte';
  import FunctionsTable from './FunctionsTable.svelte';
</script>

<section class="left-panel">
  <CallGraph />
  <FunctionsTable />
</section>

<style>
  .left-panel {
    width: 40%;
    display: flex;
    flex-direction: column;
    background: var(--color-surface-container-low);
    overflow: hidden;
    flex-shrink: 0;
  }
</style>
