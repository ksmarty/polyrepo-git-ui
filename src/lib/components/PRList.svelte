<script lang="ts">
  import type { PullRequest } from '../types';
  import PRCard from './PRCard.svelte';
  import { RefreshCw, GitPullRequest } from '@lucide/svelte';
  import { app } from '../stores.svelte';

  let filterRepo: string = $state('all');
  let filterStatus: string = $state('all');

  function getFilteredPRs(): PullRequest[] {
    let filtered = app.prs;

    if (filterRepo !== 'all') {
      filtered = filtered.filter(pr => pr.repo_id === filterRepo);
    }

    if (filterStatus !== 'all') {
      filtered = filtered.filter(pr => {
        switch (filterStatus) {
          case 'behind': return pr.behind_count > 0;
          case 'clean': return pr.behind_count === 0;
          case 'conflicts': return !pr.mergeable;
          default: return true;
        }
      });
    }

    return filtered;
  }

  function getRepoName(repoId: string): string {
    const repo = app.repos.find(r => r.id === repoId);
    return repo?.name ?? 'Unknown';
  }

  app.loadPRs();
</script>

<div class="pr-view">
  <div class="pr-header">
    <h2>Pull Requests</h2>
    <div class="filters">
      <select bind:value={filterRepo}>
        <option value="all">All Repos</option>
        {#each app.repos as repo}
          <option value={repo.id}>{repo.name}</option>
        {/each}
      </select>

      <select bind:value={filterStatus}>
        <option value="all">All Status</option>
        <option value="behind">Behind Default</option>
        <option value="clean">Up to Date</option>
        <option value="conflicts">Has Conflicts</option>
      </select>

      <button class="refresh-btn" onclick={() => app.loadPRs(true)} disabled={app.loadingPrs}>
        <span class:spin={app.loadingPrs}><RefreshCw size={14} /></span>
      </button>
    </div>
  </div>

  {#if app.prsError}
    <div class="error">
      <p>{app.prsError}</p>
      <button onclick={() => app.loadPRs(true)}>Retry</button>
    </div>
  {:else if app.loadingPrs && !app.prsLoaded}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading pull requests...</p>
    </div>
  {:else if getFilteredPRs().length === 0}
    <div class="empty-state">
      <GitPullRequest size={40} />
      <p>No pull requests found</p>
      {#if filterRepo !== 'all' || filterStatus !== 'all'}
        <p class="hint">Try adjusting your filters</p>
      {/if}
    </div>
  {:else}
    <div class="pr-list">
      {#each getFilteredPRs() as pr (pr.id)}
        <PRCard {pr} repoName={getRepoName(pr.repo_id)} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .pr-view {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .pr-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .pr-header h2 {
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .filters {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .filters select {
    min-width: 150px;
    font-size: 13px;
  }

  .refresh-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    padding: 0;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    border-radius: 8px;
  }

  .refresh-btn:hover {
    background-color: var(--accent);
    color: white;
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  .error {
    background-color: rgba(242, 95, 76, 0.1);
    border: 1px solid var(--danger);
    border-radius: 10px;
    padding: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .error p {
    color: var(--danger);
    font-size: 14px;
  }

  .error button {
    background-color: var(--danger);
    color: white;
    font-size: 13px;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    gap: 16px;
    color: var(--text-secondary);
  }

  .spinner {
    width: 28px;
    height: 28px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .pr-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    gap: 12px;
    color: var(--text-secondary);
  }

  .hint {
    font-size: 13px;
    opacity: 0.6;
  }
</style>
