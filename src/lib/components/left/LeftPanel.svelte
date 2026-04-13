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
    border-right: 1px solid rgba(66, 72, 84, 0.15);
    overflow: hidden;
    flex-shrink: 0;
  }
</style>
