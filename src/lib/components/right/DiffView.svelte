<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import type { FileDiff, Hunk, DiffLine } from '$lib/types';

  interface LineEntry {
    leftNum: number | null;
    rightNum: number | null;
    content: string;
    kind: 'added' | 'removed' | 'context' | 'empty' | 'hunk';
  }

  function buildLines(diff: FileDiff): LineEntry[] {
    const result: LineEntry[] = [];
    const fileLines = diff.content.split('\n');
    if (fileLines[fileLines.length - 1] === '') fileLines.pop();

    let oldLine = 1;
    let newLine = 1;

    for (const hunk of diff.hunks) {
      const match = hunk.header.match(/@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
      if (!match) continue;
      const hunkNewStart = parseInt(match[2]);

      // Unchanged lines before this hunk
      while (newLine < hunkNewStart) {
        result.push({ leftNum: oldLine++, rightNum: newLine++, content: fileLines[newLine - 2] ?? '', kind: 'context' });
      }

      // Hunk lines
      for (const line of hunk.lines) {
        if (line.kind === 'removed') {
          result.push({ leftNum: oldLine++, rightNum: null, content: line.content, kind: 'removed' });
        } else if (line.kind === 'added') {
          result.push({ leftNum: null, rightNum: newLine++, content: line.content, kind: 'added' });
        } else {
          result.push({ leftNum: oldLine++, rightNum: newLine++, content: line.content, kind: 'context' });
        }
      }
    }

    // Remaining unchanged lines after last hunk
    while (newLine <= fileLines.length) {
      result.push({ leftNum: oldLine++, rightNum: newLine, content: fileLines[newLine - 1] ?? '', kind: 'context' });
      newLine++;
    }

    return result;
  }

  function buildUntrackedLines(content: string): LineEntry[] {
    const fileLines = content.split('\n');
    if (fileLines[fileLines.length - 1] === '') fileLines.pop();
    return fileLines.map((line, i) => ({
      leftNum: null,
      rightNum: i + 1,
      content: line,
      kind: 'added' as const,
    }));
  }

  const activeDiff = $derived(
    appState.gitState?.diffs.find((d) => d.path === appState.activeTab) ?? null
  );

  const activeUntracked = $derived(
    appState.gitState?.untracked.find(u => u.path === appState.activeTab) ?? null
  );

  const isUntracked = $derived(activeUntracked != null);

  const lines = $derived(
    activeDiff
      ? buildLines(activeDiff)
      : activeUntracked
        ? buildUntrackedLines(activeUntracked.content)
        : []
  );
</script>

<div class="diff-view">
  {#if !appState.activeTab}
    <div class="no-file">
      <span class="material-symbols-outlined">code_off</span>
      <p>Select a file to view its diff</p>
    </div>
  {:else if !activeDiff && !isUntracked}
    <div class="no-file">
      <span class="material-symbols-outlined">check_circle</span>
      <p>No changes in this file</p>
    </div>
  {:else}
    <div class="code-area">
      <!-- Line numbers column -->
      <div class="line-numbers" aria-hidden="true">
        {#each lines as line, i (i)}
          <div
            class="line-num"
            class:line-added={line.kind === 'added'}
            class:line-removed={line.kind === 'removed'}
            class:line-hunk={line.kind === 'hunk'}
          >
            {#if line.kind === 'hunk'}
              <span>···</span>
            {:else if line.kind === 'removed'}
              <span>{line.leftNum ?? ''}</span>
            {:else if line.kind === 'added'}
              <span>{line.rightNum ?? ''}</span>
            {:else}
              <span>{line.leftNum ?? ''}</span>
            {/if}
          </div>
        {/each}
      </div>

      <!-- Code column -->
      <div class="code-lines">
        {#each lines as line, i (i)}
          <div
            class="code-line"
            class:code-added={line.kind === 'added'}
            class:code-removed={line.kind === 'removed'}
            class:code-hunk={line.kind === 'hunk'}
          >
            {#if line.kind === 'added'}
              <span class="diff-prefix added-prefix">+</span>
            {:else if line.kind === 'removed'}
              <span class="diff-prefix removed-prefix">-</span>
            {:else if line.kind === 'hunk'}
              <span class="hunk-text">{line.content}</span>
            {:else}
              <span class="diff-prefix context-prefix">&nbsp;</span>
            {/if}
            {#if line.kind !== 'hunk'}
              <span class="code-content">{line.content}</span>
            {/if}
          </div>
        {/each}

      </div>
    </div>
  {/if}
</div>

<style>
  .diff-view {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.6;
    background: var(--color-surface);
  }

  .no-file {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 0.5rem;
    color: var(--color-on-surface-variant);
  }

  .no-file .material-symbols-outlined {
    font-size: 32px;
  }

  .no-file p {
    font-family: var(--font-body);
    font-size: 13px;
  }

  .code-area {
    display: flex;
    min-width: 100%;
  }

  /* Line numbers */
  .line-numbers {
    width: 48px;
    background: var(--color-surface-container-low);
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    padding: 1rem 0;
    border-right: 1px solid rgba(66, 72, 84, 0.05);
    flex-shrink: 0;
    user-select: none;
  }

  .line-num {
    padding: 0 12px;
    font-size: 12px;
    color: rgba(144, 159, 181, 0.4);
    height: calc(13px * 1.6);
    display: flex;
    align-items: center;
    justify-content: flex-end;
    width: 100%;
  }

  .line-num.line-added {
    background: rgba(47, 58, 163, 0.1);
    color: var(--color-primary-dim);
  }

  .line-num.line-removed {
    background: rgba(127, 39, 55, 0.1);
    color: var(--color-error-dim);
  }

  .line-num.line-hunk {
    color: var(--color-outline);
  }

  /* Code lines */
  .code-lines {
    flex: 1;
    padding: 1rem 0;
    min-width: 0;
  }

  .code-line {
    display: flex;
    align-items: baseline;
    padding: 0 1rem;
    height: calc(13px * 1.6);
    white-space: pre;
    color: rgba(224, 229, 245, 0.9);
  }

  .code-line.code-added {
    background: rgba(47, 58, 163, 0.2);
    border-left: 2px solid var(--color-primary);
    color: var(--color-on-primary-container);
  }

  .code-line.code-removed {
    background: rgba(127, 39, 55, 0.2);
    border-left: 2px solid var(--color-error);
    color: var(--color-on-error-container);
  }

  .code-line.code-hunk {
    color: var(--color-outline);
    font-size: 11px;
    background: rgba(66, 72, 84, 0.1);
    height: auto;
    padding: 2px 1rem;
  }

  .diff-prefix {
    margin-right: 8px;
    flex-shrink: 0;
    user-select: none;
    font-size: 13px;
  }

  .added-prefix { opacity: 0.6; color: var(--color-primary-dim); }
  .removed-prefix { opacity: 0.6; color: var(--color-error-dim); }
  .context-prefix { opacity: 0; }

  .code-content {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hunk-text {
    font-size: 11px;
    color: var(--color-outline);
  }

  /* AI Suggestion */
  .ai-suggestion {
    margin: 1.5rem 1rem;
    padding: 1rem;
    border-radius: 12px;
    background: rgba(32, 38, 49, 0.4);
    border: 1px solid rgba(200, 144, 255, 0.2);
    backdrop-filter: blur(12px);
    position: relative;
  }

  .ai-label {
    position: absolute;
    top: -10px;
    left: 24px;
    padding: 0 8px;
    background: var(--color-tertiary);
    color: var(--color-on-tertiary);
    font-size: 10px;
    font-weight: 700;
    border-radius: 2px;
    letter-spacing: 0.1em;
    font-family: var(--font-body);
  }

  .ai-body {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .ai-icon {
    font-size: 22px;
    color: var(--color-tertiary);
    flex-shrink: 0;
  }

  .ai-content {
    flex: 1;
  }

  .ai-text {
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--color-on-surface-variant);
    margin-bottom: 0.5rem;
    line-height: 1.5;
  }

  .ai-actions {
    display: flex;
    gap: 0.75rem;
  }

  .ai-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 11px;
    font-weight: 700;
    font-family: var(--font-body);
    transition: text-decoration 150ms;
  }

  .ai-btn.accept {
    color: var(--color-primary);
  }

  .ai-btn.ignore {
    color: var(--color-secondary);
  }

  .ai-btn:hover {
    text-decoration: underline;
  }
</style>
