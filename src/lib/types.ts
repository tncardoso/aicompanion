export type DiffLineKind = 'added' | 'removed' | 'context';

export interface DiffLine {
  kind: DiffLineKind;
  content: string;
}

export interface Hunk {
  header: string;
  lines: DiffLine[];
}

export interface FileDiff {
  path: string;
  added: number;
  removed: number;
  hunks: Hunk[];
  content: string;
}

export interface UntrackedFile {
  path: string;
  content: string;
}

export interface GitState {
  repo_root: string;
  start_dir: string;
  diffs: FileDiff[];
  untracked: UntrackedFile[];
}

export interface FnId {
  file: string;
  name: string;
}

export interface CallGraph {
  // Vec<(FnId, Vec<FnId>)> serialized as array of 2-element arrays
  edges: Array<[FnId, FnId[]]>;
  nodes: FnId[];
}

export interface FunctionMetricsDelta {
  file: string;
  name: string;
  cyclomatic: number;
  cognitive: number;
  coupling: number;
  cyclomatic_delta: number | null;
  cognitive_delta: number | null;
  coupling_delta: number | null;
}

export interface Analysis {
  metrics: FunctionMetricsDelta[];
  call_graph: CallGraph;
}

export interface Config {
  thresholds: {
    cyclomatic: number;
    cognitive: number;
    coupling: number;
  };
}

export type MetricSort =
  | 'cyclomatic_value'
  | 'cyclomatic_delta'
  | 'cognitive_value'
  | 'cognitive_delta'
  | 'coupling_value'
  | 'coupling_delta';
