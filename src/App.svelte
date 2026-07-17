<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte';
  import PRList from './lib/components/PRList.svelte';
  import Settings from './lib/components/Settings.svelte';
  import { app } from './lib/stores.svelte';
  import type { Repository } from './lib/types';
  import { FolderGit2, GitPullRequest, Settings as SettingsIcon, RefreshCw, Download, ExternalLink, GitMerge, History, AlertTriangle, X } from '@lucide/svelte';

  let activeTab: 'repos' | 'prs' | 'settings' = $state('repos');
  let editingDefaultBranch: boolean = $state(false);
  let newDefaultBranch: string = $state('');
  let expandedCommit: string | null = $state(null);

  function getGitHubUrl(repo: Repository): string | null {
    if (repo.github_owner && repo.github_repo) {
      return `https://github.com/${repo.github_owner}/${repo.github_repo}`;
    }
    if (repo.remote_url) {
      return repo.remote_url
        .replace('git@github.com:', 'https://github.com/')
        .replace('.git', '');
    }
    return null;
  }

  function getCommitUrl(repo: Repository, hash: string): string | null {
    const baseUrl = getGitHubUrl(repo);
    if (!baseUrl) return null;
    return `${baseUrl}/commit/${hash}`;
  }

  async function openGitHub() {
    if (!app.selectedRepo) return;
    const url = getGitHubUrl(app.selectedRepo);
    if (!url) return;
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(url);
    } catch (e) {
      console.error('Failed to open URL:', e);
    }
  }

  async function openCommitUrl(hash: string) {
    if (!app.selectedRepo) return;
    const url = getCommitUrl(app.selectedRepo, hash);
    if (!url) return;
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(url);
    } catch (e) {
      console.error('Failed to open URL:', e);
    }
  }

  async function saveDefaultBranch() {
    if (!app.selectedRepo) return;
    const val = newDefaultBranch.trim() || null;
    await app.saveDefaultBranch(app.selectedRepo.id, val);
    editingDefaultBranch = false;
  }

  function startEditDefaultBranch() {
    newDefaultBranch = app.selectedRepo?.default_branch ?? '';
    editingDefaultBranch = true;
  }

  function handleThemeChange(event: CustomEvent<string>) {
    app.updateConfig({ ...app.config, theme: event.detail });
  }

  function handleDataChange() {
    app.loadAll();
  }

  app.loadAll();
</script>

<div class="app">
  <header class="header">
    <h1 class="logo">
      <FolderGit2 size={20} />
      Polyrepo
    </h1>
    <nav class="tabs">
      <button
        class="tab"
        class:active={activeTab === 'repos'}
        onclick={() => activeTab = 'repos'}
      >
        <FolderGit2 size={16} />
        Repos
      </button>
      <button
        class="tab"
        class:active={activeTab === 'prs'}
        onclick={() => activeTab = 'prs'}
      >
        <GitPullRequest size={16} />
        Pull Requests
      </button>
      <button
        class="tab"
        class:active={activeTab === 'settings'}
        onclick={() => activeTab = 'settings'}
      >
        <SettingsIcon size={16} />
        Settings
      </button>
    </nav>
    <button
      class="refresh-all-btn"
      onclick={() => app.refreshAllRepos()}
      disabled={app.refreshingAll || app.repos.length === 0}
      title="Refresh all repos"
    >
      <RefreshCw size={14} class={app.refreshingAll ? 'spin' : ''} />
      Refresh All
    </button>
  </header>

  <div class="main">
    <Sidebar />

    <main class="content">
      {#if activeTab === 'repos'}
        <div class="repos-view">
          {#if app.selectedRepo}
            <div class="repo-detail">
              <div class="repo-header">
                <div>
                  <h2>{app.selectedRepo.name}</h2>
                  <p class="repo-path">{app.selectedRepo.path}</p>
                </div>
                <div class="repo-actions">
                  <button
                    class="icon-btn"
                    onclick={() => app.refreshRepo(app.selectedRepo!.id)}
                    disabled={app.refreshingRepo === app.selectedRepo.id}
                    title="Refresh repo status"
                  >
                    <RefreshCw size={14} class={app.refreshingRepo === app.selectedRepo.id ? 'spin' : ''} />
                  </button>
                  <button
                    class="icon-btn"
                    onclick={() => app.fetchRepo(app.selectedRepo!.id)}
                    title="Fetch from remote"
                  >
                    <Download size={14} />
                  </button>
                  {#if app.selectedRepo.sync_status && app.selectedRepo.sync_status.behind > 0}
                    <button
                      class="icon-btn merge-btn"
                      onclick={() => app.mergeRepo(app.selectedRepo!.id)}
                      disabled={app.mergingRepo === app.selectedRepo.id}
                      title="Merge changes from base branch"
                    >
                      <GitMerge size={14} class={app.mergingRepo === app.selectedRepo.id ? 'spin' : ''} />
                    </button>
                  {/if}
                  {#if getGitHubUrl(app.selectedRepo)}
                    <button
                      class="icon-btn"
                      onclick={openGitHub}
                      title="Open on GitHub"
                    >
                      <ExternalLink size={14} />
                    </button>
                  {/if}
                </div>
              </div>

              <div class="repo-status">
                <span class="branch">{app.selectedRepo.current_branch}</span>
                {#if app.selectedRepo.sync_status}
                  <span
                    class="sync-badge"
                    class:up-to-date={app.selectedRepo.sync_status.up_to_date}
                    class:behind={app.selectedRepo.sync_status.behind > 0}
                    class:ahead={app.selectedRepo.sync_status.ahead > 0}
                    class:dirty={app.selectedRepo.sync_status.is_dirty}
                  >
                    {#if app.selectedRepo.sync_status.is_dirty}
                      Modified
                    {:else if app.selectedRepo.sync_status.up_to_date}
                      Up to date
                    {:else}
                      {app.selectedRepo.sync_status.behind} behind, {app.selectedRepo.sync_status.ahead} ahead
                    {/if}
                  </span>
                {/if}
              </div>

              <div class="repo-meta">
                <div class="meta-row">
                  <span class="meta-label">Default Branch</span>
                  {#if editingDefaultBranch}
                    <div class="inline-edit">
                      <input
                        type="text"
                        bind:value={newDefaultBranch}
                        placeholder="e.g. main"
                        onkeydown={(e) => {
                          if (e.key === 'Enter') saveDefaultBranch();
                          if (e.key === 'Escape') editingDefaultBranch = false;
                        }}
                      />
                      <button class="save-btn" onclick={saveDefaultBranch}>Save</button>
                      <button class="cancel-btn" onclick={() => editingDefaultBranch = false}>Cancel</button>
                    </div>
                  {:else}
                    <button class="meta-value" onclick={startEditDefaultBranch}>
                      {app.selectedRepo.default_branch ?? app.config.default_branch}
                    </button>
                  {/if}
                </div>
                {#if app.selectedRepo.remote_url}
                  <div class="meta-row">
                    <span class="meta-label">Remote</span>
                    <span class="meta-value mono">{app.selectedRepo.remote_url}</span>
                  </div>
                {/if}
              </div>

              <div class="git-history">
                <h3 class="section-title">
                  <History size={14} />
                  Recent History
                </h3>
                {#if app.loadingGitLog}
                  <p class="loading-text">Loading...</p>
                {:else if app.gitLog.length === 0}
                  <p class="empty-text">No commits found</p>
                {:else}
                  <div class="commit-list">
                    {#each app.gitLog as commit (commit.hash)}
                      <div class="commit-row" class:expanded={expandedCommit === commit.hash}>
                        <button
                          class="commit-expand-btn"
                          onclick={() => expandedCommit = expandedCommit === commit.hash ? null : commit.hash}
                        >
                          <span class="commit-hash">{commit.short_hash}</span>
                        </button>
                        <span class="commit-message">{commit.message}</span>
                        <span class="commit-meta">{commit.author} &middot; {commit.date}</span>
                        {#if getCommitUrl(app.selectedRepo, commit.hash)}
                          <button
                            class="commit-link-btn"
                            onclick={() => openCommitUrl(commit.hash)}
                            title="View on GitHub"
                          >
                            <ExternalLink size={12} />
                          </button>
                        {/if}
                      </div>
                      {#if expandedCommit === commit.hash}
                        <div class="commit-details">
                          <div class="commit-detail-row">
                            <span class="detail-label">Full Hash</span>
                            <span class="detail-value mono">{commit.hash}</span>
                          </div>
                          <div class="commit-detail-row">
                            <span class="detail-label">Author</span>
                            <span class="detail-value">{commit.author}</span>
                          </div>
                          <div class="commit-detail-row">
                            <span class="detail-label">Date</span>
                            <span class="detail-value">{commit.date}</span>
                          </div>
                          <div class="commit-detail-row">
                            <span class="detail-label">Message</span>
                            <span class="detail-value">{commit.message}</span>
                          </div>
                        </div>
                      {/if}
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {:else}
            <div class="empty-state">
              <FolderGit2 size={48} />
              <p>Select a repo from the sidebar</p>
            </div>
          {/if}
        </div>
      {:else if activeTab === 'prs'}
        <PRList />
      {:else if activeTab === 'settings'}
        <Settings on:themeChange={handleThemeChange} on:dataChange={handleDataChange} />
      {/if}
    </main>
  </div>
</div>

{#if app.showMergeConflict && app.mergeResult}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={() => app.showMergeConflict = false}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h3>
          <AlertTriangle size={18} />
          {app.mergeResult.success ? 'Merge Successful' : 'Merge Conflict'}
        </h3>
        <button class="modal-close" onclick={() => app.showMergeConflict = false}>
          <X size={16} />
        </button>
      </div>
      <div class="modal-body">
        <p class="merge-message">{app.mergeResult.message}</p>
        {#if app.mergeResult.conflicts && app.mergeResult.conflicts.length > 0}
          <div class="conflict-list">
            <p class="conflict-title">Conflicting files:</p>
            {#each app.mergeResult.conflicts as conflict}
              <div class="conflict-item">
                <span class="conflict-file">{conflict.file}</span>
                <span class="conflict-status">{conflict.status}</span>
              </div>
            {/each}
          </div>
          <p class="conflict-hint">Resolve conflicts in your editor, then commit the merge.</p>
        {/if}
      </div>
      <div class="modal-actions">
        {#if app.mergeResult.conflicts && app.mergeResult.conflicts.length > 0}
          <button class="abort-btn" onclick={() => app.abortMerge(app.selectedRepo!.id)}>
            Abort Merge
          </button>
        {/if}
        <button class="close-btn" onclick={() => app.showMergeConflict = false}>
          {app.mergeResult.success ? 'Done' : 'Close'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    height: 52px;
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    gap: 16px;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: 700;
    color: var(--accent);
    letter-spacing: -0.02em;
  }

  .tabs {
    display: flex;
    gap: 2px;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    color: var(--text-secondary);
    padding: 8px 14px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
  }

  .tab:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .tab.active {
    background-color: var(--accent);
    color: white;
  }

  .main {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .content {
    flex: 1;
    overflow: auto;
    padding: 24px;
  }

  .repos-view {
    height: 100%;
  }

  .repo-detail {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
  }

  .repo-detail h2 {
    font-size: 22px;
    font-weight: 700;
    margin-bottom: 6px;
    letter-spacing: -0.02em;
  }

  .repo-path {
    color: var(--text-secondary);
    font-size: 13px;
    font-family: monospace;
    margin-bottom: 16px;
  }

  .repo-status {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .branch {
    background-color: var(--bg-tertiary);
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 12px;
    font-family: monospace;
    font-weight: 500;
  }

  .sync-badge {
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 600;
  }

  .sync-badge.up-to-date {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .sync-badge.behind {
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
  }

  .sync-badge.ahead {
    background-color: rgba(127, 90, 240, 0.15);
    color: var(--info);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
    color: var(--text-secondary);
  }

  .refresh-all-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
    white-space: nowrap;
  }

  .refresh-all-btn:hover:not(:disabled) {
    background-color: var(--accent);
    color: white;
  }

  .refresh-all-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .repo-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 16px;
  }

  .repo-header h2 {
    font-size: 22px;
    font-weight: 700;
    margin-bottom: 4px;
    letter-spacing: -0.02em;
  }

  .repo-actions {
    display: flex;
    gap: 4px;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    border: none;
    border-radius: 6px;
    padding: 0;
  }

  .icon-btn:hover:not(:disabled) {
    background-color: var(--accent);
    color: white;
  }

  .icon-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .sync-badge.dirty {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .repo-meta {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .meta-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .meta-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    min-width: 100px;
  }

  .meta-value {
    font-size: 13px;
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .meta-value:hover {
    background-color: var(--bg-tertiary);
  }

  .meta-value.mono {
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .inline-edit {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .inline-edit input {
    width: 200px;
    padding: 4px 8px;
    font-size: 13px;
  }

  .save-btn {
    padding: 4px 10px;
    background-color: var(--accent);
    color: white;
    font-size: 12px;
    border-radius: 6px;
  }

  .cancel-btn {
    padding: 4px 10px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    border-radius: 6px;
  }

  .cancel-btn:hover {
    background-color: var(--bg-tertiary);
  }

  .merge-btn {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .merge-btn:hover:not(:disabled) {
    background-color: var(--success);
    color: white;
  }

  .git-history {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 10px;
  }

  .commit-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .commit-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 8px;
    border-radius: 6px;
    font-size: 12px;
  }

  .commit-row:hover {
    background-color: var(--bg-tertiary);
  }

  .commit-expand-btn {
    background: transparent;
    padding: 0;
    display: flex;
    align-items: center;
  }

  .commit-hash {
    font-family: monospace;
    color: var(--accent);
    font-weight: 500;
    flex-shrink: 0;
  }

  .commit-message {
    flex: 1;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .commit-meta {
    color: var(--text-secondary);
    font-size: 11px;
    flex-shrink: 0;
  }

  .commit-link-btn {
    background: transparent;
    color: var(--text-secondary);
    padding: 2px;
    display: flex;
    align-items: center;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .commit-row:hover .commit-link-btn {
    opacity: 1;
  }

  .commit-link-btn:hover {
    color: var(--accent);
  }

  .commit-details {
    padding: 8px 12px 8px 32px;
    background-color: var(--bg-tertiary);
    border-radius: 6px;
    margin: 2px 0 4px;
  }

  .commit-detail-row {
    display: flex;
    gap: 12px;
    padding: 3px 0;
    font-size: 12px;
  }

  .detail-label {
    color: var(--text-secondary);
    min-width: 70px;
    flex-shrink: 0;
  }

  .detail-value {
    color: var(--text-primary);
    word-break: break-all;
  }

  .detail-value.mono {
    font-family: monospace;
  }

  .loading-text, .empty-text {
    font-size: 12px;
    color: var(--text-secondary);
    padding: 8px;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal-content {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: 480px;
    max-height: 80vh;
    overflow: auto;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .modal-close {
    background: transparent;
    color: var(--text-secondary);
    padding: 4px;
    display: flex;
    align-items: center;
  }

  .modal-close:hover {
    color: var(--text-primary);
  }

  .modal-body {
    padding: 16px 20px;
  }

  .merge-message {
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 12px;
    white-space: pre-wrap;
    font-family: monospace;
  }

  .conflict-list {
    margin-bottom: 12px;
  }

  .conflict-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .conflict-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    background-color: var(--bg-tertiary);
    border-radius: 4px;
    margin-bottom: 4px;
    font-size: 12px;
  }

  .conflict-file {
    font-family: monospace;
    color: var(--text-primary);
  }

  .conflict-status {
    color: var(--danger);
    font-size: 11px;
  }

  .conflict-hint {
    font-size: 12px;
    color: var(--text-secondary);
    font-style: italic;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--border);
  }

  .abort-btn {
    padding: 6px 12px;
    background-color: var(--danger);
    color: white;
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
  }

  .abort-btn:hover {
    opacity: 0.9;
  }

  .close-btn {
    padding: 6px 12px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
  }

  .close-btn:hover {
    background-color: var(--accent);
    color: white;
  }
</style>
