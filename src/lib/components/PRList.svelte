<script lang="ts">
  import type { PullRequest } from '../types';
  import PRCard from './PRCard.svelte';
  import PRDetail from './PRDetail.svelte';
  import Masonry from 'svelte-masonry';
  import { RefreshCw, GitPullRequest, LayoutGrid, List } from '@lucide/svelte';
  import { app } from '../stores.svelte';

  let filterRepo: string = $state('all');
  let filterState: string = $state('open');
  let filterStatus: string = $state('all');
  let groupByRepo: boolean = $state(true);
  let initialized: boolean = $state(false);
  let selectedPr: PullRequest | null = $state(null);

  const compact = $derived(app.config.pr_density === 'compact');

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

  function getGroupedPRs(): Map<string, PullRequest[]> {
    const groups = new Map<string, PullRequest[]>();
    for (const pr of getFilteredPRs()) {
      const name = getRepoName(pr.repo_id);
      const list = groups.get(name) ?? [];
      list.push(pr);
      groups.set(name, list);
    }
    return groups;
  }

  $effect(() => {
    const state = filterState;
    if (initialized) {
      app.loadPRs(true, state);
    } else {
      app.loadPRs(false, state);
      initialized = true;
    }
  });
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

      <select bind:value={filterState}>
        <option value="open">Open</option>
        <option value="closed">Closed</option>
        <option value="all">All States</option>
      </select>

      <select bind:value={filterStatus}>
        <option value="all">All Status</option>
        <option value="behind">Behind Default</option>
        <option value="clean">Up to Date</option>
        <option value="conflicts">Has Conflicts</option>
      </select>

      <button
        class="toggle-btn"
        class:active={groupByRepo}
        onclick={() => groupByRepo = !groupByRepo}
        title={groupByRepo ? 'Ungroup PRs' : 'Group by repo'}
      >
        {#if groupByRepo}
          <LayoutGrid size={14} />
        {:else}
          <List size={14} />
        {/if}
      </button>

      <button class="refresh-btn" onclick={() => app.loadPRs(true, filterState)} disabled={app.loadingPrs}>
        <span class="spin-wrapper" class:spin={app.loadingPrs}><RefreshCw size={14} /></span>
      </button>
    </div>
  </div>

  {#if app.prsError}
    <div class="error">
      <p>{app.prsError}</p>
      <button onclick={() => app.loadPRs(true, filterState)}>Retry</button>
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
  {:else if groupByRepo}
    <div class="pr-list">
      {#each [...getGroupedPRs()] as [repoName, prs] (repoName)}
        <div class="repo-group">
          <div class="repo-group-header">
            <span class="repo-group-name">{repoName}</span>
            <span class="repo-group-count">{prs.length}</span>
          </div>
          <div class="repo-group-prs">
            {#each prs as pr (pr.id)}
              <PRCard pr={pr} repoName="" {compact} onSelect={(p) => selectedPr = p} />
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="pr-list flat">
      <Masonry items={getFilteredPRs()} colWidth={'minmax(400px, 1fr)'} gridGap={'10px'}>
        {#each getFilteredPRs() as pr (pr.id)}
          <PRCard {pr} repoName={getRepoName(pr.repo_id)} {compact} onSelect={(p) => selectedPr = p} />
        {/each}
      </Masonry>
    </div>
  {/if}

  {#if selectedPr}
    <PRDetail
      pr={selectedPr}
      repoName={getRepoName(selectedPr.repo_id)}
      onClose={() => selectedPr = null}
    />
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

  .toggle-btn {
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

  .toggle-btn:hover {
    background-color: var(--accent);
    color: white;
  }

  .toggle-btn.active {
    background-color: var(--accent);
    color: white;
  }

  .spin-wrapper {
    display: inline-flex;
    align-items: center;
    justify-content: center;
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
    gap: 6px;
  }

  .repo-group {
    margin-bottom: 12px;
  }

  .repo-group-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .repo-group-name {
    flex: 1;
  }

  .repo-group-count {
    background-color: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 500;
  }

  .repo-group-prs {
    display: flex;
    flex-direction: column;
    gap: 6px;
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
