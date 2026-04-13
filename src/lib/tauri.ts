import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { GitState, Analysis, Config } from './types';

export const getCwd = () =>
  invoke<string>('get_cwd');

export const getGitState = (repoPath: string) =>
  invoke<GitState>('get_git_state', { repoPath });

export const runAnalysis = (repoPath: string) =>
  invoke<Analysis>('run_analysis', { repoPath });

export const getConfig = (repoPath: string) =>
  invoke<Config>('get_config', { repoPath });

export const watchRepo = (repoPath: string) =>
  invoke<void>('watch_repo', { repoPath });

export const onRepoChanged = (callback: () => void) =>
  listen('repo-changed', callback);
