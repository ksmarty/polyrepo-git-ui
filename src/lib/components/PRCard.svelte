<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { PullRequest } from '../types';
  import { ChevronDown, GitMerge, RotateCcw, CheckCircle, XCircle, Clock, Users } from '@lucide/svelte';

  interface Props {
    pr: PullRequest;
    repoName: string;
    compact?: boolean;
    onSelect?: (pr: PullRequest) => void;
  }

  let { pr, repoName, compact = false, onSelect }: Props = $props();

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

  async function openInGitHub() {
    if (!pr.html_url) return;
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(pr.html_url);
    } catch (e) {
      console.error('Failed to open URL:', e);
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

  function getChecksIcon() {
    switch (pr.checks_status) {
      case 'success': return CheckCircle;
      case 'failure': return XCircle;
      default: return Clock;
    }
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

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pr-card" class:compact onclick={() => onSelect?.(pr)}>
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
        {@const Icon = getChecksIcon()}
        <span class="status-chip checks {getChecksClass()}">
          <Icon size={12} />
          {pr.checks_status}
        </span>
      {/if}

      {#if pr.review_decision}
        <span class="status-chip {getReviewClass()}">
          {pr.review_decision}
        </span>
      {/if}

      {#if pr.requested_reviewers.length > 0}
        <span class="status-chip review-badge">
          <Users size={12} />
          {pr.requested_reviewers.length}
        </span>
      {/if}
    </div>
  </div>

  <div class="pr-actions">
    {#if pr.html_url}
      <button
        class="github-btn"
        onclick={(e) => { e.stopPropagation(); openInGitHub(); }}
        title="Open in GitHub"
      >
        <svg viewBox="0 0 24 24" width="16" height="16"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" fill="currentColor" /></svg>
      </button>
    {/if}

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
    cursor: pointer;
    transition: border-color 0.15s;
  }

  .pr-card:hover {
    border-color: var(--accent);
  }

  .pr-card.compact {
    padding: 10px 12px;
    gap: 12px;
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

  .compact .pr-title-row {
    margin-bottom: 4px;
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

  .compact .pr-meta {
    margin-bottom: 6px;
    gap: 8px;
    font-size: 11px;
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

  .status-chip.review-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: rgba(127, 90, 240, 0.15);
    color: var(--accent);
  }

  .status-chip.checks {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .pr-actions {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .github-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    border-radius: 8px;
    flex-shrink: 0;
  }

  .github-btn:hover {
    background-color: var(--accent);
    color: white;
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
