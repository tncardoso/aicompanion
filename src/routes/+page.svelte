<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import { appState } from '$lib/store.svelte';
  import { getCwd, getGitState, runAnalysis, getConfig, watchRepo, onRepoChanged } from '$lib/tauri';

  async function refresh(repoPath: string) {
    try {
      const gitState = await getGitState(repoPath);
      appState.gitState = gitState;

      if (gitState.diffs.length > 0 && appState.openTabs.length === 0) {
        appState.openTabs = gitState.diffs.map((d) => d.path);
        appState.activeTab = gitState.diffs[0].path;
      }
    } catch (e) {
      console.error('get_git_state failed:', e);
    }

    try {
      const analysis = await runAnalysis(repoPath);
      appState.analysis = analysis;
    } catch (e) {
      console.error('run_analysis failed:', e);
      appState.analysisError = String(e);
    }
  }

  onMount(async () => {
    try {
      const repoPath = await getCwd();
      appState.repoPath = repoPath;
      appState.loading = true;

      // Load config in parallel but don't block UI on analysis
      const [, config] = await Promise.all([
        refresh(repoPath),
        getConfig(repoPath).catch(() => null),
      ]);
      if (config) appState.config = config;

      appState.loading = false;

      await watchRepo(repoPath);
      await onRepoChanged(() => refresh(repoPath));
    } catch (e) {
      appState.error = String(e);
      appState.loading = false;
    }
  });
</script>

<AppShell />
