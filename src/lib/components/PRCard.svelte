<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { PullRequest } from '../types';
  import { ChevronDown, GitMerge, RotateCcw } from '@lucide/svelte';

  interface Props {
    pr: PullRequest;
    repoName: string;
  }

  let { pr, repoName }: Props = $props();

  let syncing: boolean = $state(false);
  let showSyncMenu: boolean = $state(false);
  let syncDropdownRef: HTMLDivElement | undefined = $state(undefined);

  function handleClickOutside(event: MouseEvent) {
    if (syncDropdownRef && !syncDropdownRef.contains(event.target as Node)) {
      showSyncMenu = false;
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
  });

  async function syncWithRebase() {
    syncing = true;
    showSyncMenu = false;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('sync_pr', { prId: pr.id, useRebase: true });
    } catch (e) {
      console.error('Failed to sync PR:', e);
    } finally {
      syncing = false;
    }
  }

  async function syncWithMerge() {
    syncing = true;
    showSyncMenu = false;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('sync_pr', { prId: pr.id, useRebase: false });
    } catch (e) {
      console.error('Failed to sync PR:', e);
    } finally {
      syncing = false;
    }
  }

  function getSyncStatusClass(): string {
    if (!pr.mergeable) return 'conflicts';
    if (pr.behind_count > 0) return 'behind';
    return 'clean';
  }

  function getSyncStatusText(): string {
    if (!pr.mergeable) return 'Has conflicts';
    if (pr.behind_count > 0) return `${pr.behind_count} commits behind`;
    return 'Up to date';
  }

  function getChecksClass(): string {
    switch (pr.checks_status) {
      case 'success': return 'success';
      case 'failure': return 'failure';
      case 'pending': return 'pending';
      case 'cancelled': return 'cancelled';
      default: return '';
    }
  }

  function getReviewClass(): string {
    switch (pr.review_decision) {
      case 'approved': return 'approved';
      case 'changes_requested': return 'changes-requested';
      default: return '';
    }
  }
</script>

<div class="pr-card">
  <div class="pr-main">
    <div class="pr-title-row">
      <span class="pr-number">#{pr.number}</span>
      <span class="pr-title">{pr.title}</span>
    </div>

    <div class="pr-meta">
      <span class="repo-badge">{repoName}</span>
      <span class="branch-info">
        <span class="head-branch">{pr.head_ref}</span>
        <span class="arrow">&rarr;</span>
        <span class="base-branch">{pr.base_ref}</span>
      </span>
    </div>

    <div class="pr-status">
      <span class="status-chip {getSyncStatusClass()}">
        {getSyncStatusText()}
      </span>

      {#if pr.checks_status}
        <span class="status-chip {getChecksClass()}">
          {pr.checks_status}
        </span>
      {/if}

      {#if pr.review_decision}
        <span class="status-chip {getReviewClass()}">
          {pr.review_decision}
        </span>
      {/if}
    </div>
  </div>

  <div class="pr-actions">
    {#if pr.behind_count > 0 || !pr.mergeable}
      <div class="sync-dropdown" bind:this={syncDropdownRef}>
        <button
          class="sync-button"
          onclick={(e) => { e.stopPropagation(); showSyncMenu = !showSyncMenu; }}
          disabled={syncing}
        >
          {syncing ? 'Syncing...' : 'Sync'}
          <span class:rotate={showSyncMenu}><ChevronDown size={14} /></span>
        </button>

        {#if showSyncMenu}
          <div class="sync-menu">
            <button class="menu-item" onclick={syncWithRebase}>
              <RotateCcw size={16} />
              <div class="menu-text">
                <span class="menu-label">Rebase</span>
                <span class="menu-desc">Rebase onto {pr.base_ref}</span>
              </div>
            </button>
            <div class="menu-divider"></div>
            <button class="menu-item" onclick={syncWithMerge}>
              <GitMerge size={16} />
              <div class="menu-text">
                <span class="menu-label">Merge</span>
                <span class="menu-desc">Merge {pr.base_ref} into branch</span>
              </div>
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .pr-card {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 16px;
    display: flex;
    justify-content: space-between;
    gap: 16px;
  }

  .pr-main {
    flex: 1;
    min-width: 0;
  }

  .pr-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .pr-number {
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 600;
  }

  .pr-title {
    font-weight: 600;
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pr-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
    font-size: 12px;
  }

  .repo-badge {
    background-color: var(--bg-tertiary);
    padding: 2px 8px;
    border-radius: 4px;
    font-weight: 500;
    font-size: 11px;
  }

  .branch-info {
    color: var(--text-secondary);
    font-family: monospace;
    font-size: 11px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .head-branch { color: var(--info); }
  .arrow { opacity: 0.4; }
  .base-branch { color: var(--text-secondary); }

  .pr-status {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .status-chip {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
  }

  .status-chip.clean {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .status-chip.behind {
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
  }

  .status-chip.conflicts {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .status-chip.success {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .status-chip.failure {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .status-chip.pending {
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
  }

  .status-chip.cancelled {
    background-color: rgba(167, 169, 190, 0.15);
    color: var(--text-secondary);
  }

  .status-chip.approved {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .status-chip.changes-requested {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .pr-actions {
    display: flex;
    align-items: flex-start;
  }

  .sync-dropdown {
    position: relative;
  }

  .sync-button {
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: var(--accent);
    color: white;
    padding: 8px 12px;
    font-size: 13px;
    font-weight: 600;
    border-radius: 8px;
  }

  .rotate {
    transform: rotate(180deg);
  }

  .sync-menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
    min-width: 240px;
    z-index: 100;
    overflow: hidden;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 12px 14px;
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    border-radius: 0;
    font-size: 13px;
  }

  .menu-item:hover {
    background-color: var(--bg-tertiary);
  }

  .menu-item :global(svg) {
    color: var(--accent);
    flex-shrink: 0;
  }

  .menu-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .menu-label {
    font-weight: 600;
    font-size: 13px;
  }

  .menu-desc {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .menu-divider {
    height: 1px;
    background-color: var(--border);
    margin: 0;
  }
</style>
