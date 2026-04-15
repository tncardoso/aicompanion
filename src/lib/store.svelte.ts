import type { GitState, Analysis, Config, MetricSort } from './types';

class AppState {
  repoPath = $state('');
  gitState = $state<GitState | null>(null);
  analysis = $state<Analysis | null>(null);
  config = $state<Config | null>(null);
  openTabs = $state<string[]>([]);
  activeTab = $state<string | null>(null);
  activeLine = $state<number | null>(null);
  sortOrder = $state<MetricSort>('cyclomatic_value');
  loading = $state(false);
  error = $state<string | null>(null);
  analysisError = $state<string | null>(null);
}

export const appState = new AppState();

export const SORT_ORDERS: MetricSort[] = [
  'cyclomatic_value',
  'cyclomatic_delta',
  'cognitive_value',
  'cognitive_delta',
  'coupling_value',
  'coupling_delta',
];

export const SORT_LABELS: Record<MetricSort, string> = {
  cyclomatic_value: 'Cyc ↓',
  cyclomatic_delta: 'ΔCyc ↓',
  cognitive_value: 'Cog ↓',
  cognitive_delta: 'ΔCog ↓',
  coupling_value: 'Cpl ↓',
  coupling_delta: 'ΔCpl ↓',
};

export function nextSort(current: MetricSort): MetricSort {
  const idx = SORT_ORDERS.indexOf(current);
  return SORT_ORDERS[(idx + 1) % SORT_ORDERS.length];
}
