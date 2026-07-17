<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte';
  import PRList from './lib/components/PRList.svelte';
  import Settings from './lib/components/Settings.svelte';
  import type { Repository, RepoGroup } from './lib/types';
  import { FolderGit2, GitPullRequest, Settings as SettingsIcon, RefreshCw, Download, ExternalLink } from '@lucide/svelte';

  let activeTab: 'repos' | 'prs' | 'settings' = $state('repos');
  let repos: Repository[] = $state([]);
  let groups: RepoGroup[] = $state([]);
  let selectedRepo: Repository | null = $state(null);
  let currentTheme: string = $state('midnight');
  let globalDefaultBranch: string = $state('main');
  let refreshingAll: boolean = $state(false);
  let refreshingRepo: string | null = $state(null);
  let editingDefaultBranch: boolean = $state(false);
  let newDefaultBranch: string = $state('');

  async function loadRepos() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      repos = await invoke('get_repos');
      groups = await invoke('get_groups');
    } catch (error) {
      console.error('Failed to load repos:', error);
    }
  }

  async function loadTheme() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const config = await invoke('get_config') as { theme: string; default_branch: string };
      currentTheme = config.theme;
      globalDefaultBranch = config.default_branch || 'main';
      applyTheme(currentTheme);
    } catch (e) {
      console.error('Failed to load theme:', e);
    }
  }

  function applyTheme(theme: string) {
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
      document.documentElement.setAttribute('data-theme', theme);
    }
  }

  function handleRepoSelect(event: CustomEvent<Repository>) {
    selectedRepo = event.detail;
    activeTab = 'repos';
    refreshRepo(event.detail.id);
  }

  async function refreshRepo(id: string) {
    refreshingRepo = id;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const refreshed = await invoke('refresh_repo', { id }) as Repository;
      repos = repos.map(r => r.id === id ? refreshed : r);
      if (selectedRepo?.id === id) {
        selectedRepo = refreshed;
      }
    } catch (e) {
      console.error('Failed to refresh repo:', e);
    } finally {
      refreshingRepo = null;
    }
  }

  async function refreshAllRepos() {
    refreshingAll = true;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const refreshed = await invoke('refresh_all_repos') as Repository[];
      repos = refreshed;
      if (selectedRepo) {
        const updated = refreshed.find(r => r.id === selectedRepo!.id);
        if (updated) selectedRepo = updated;
      }
    } catch (e) {
      console.error('Failed to refresh all repos:', e);
    } finally {
      refreshingAll = false;
    }
  }

  async function fetchRepo(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('fetch_repo', { id });
      await refreshRepo(id);
    } catch (e) {
      console.error('Failed to fetch repo:', e);
    }
  }

  async function saveDefaultBranch() {
    if (!selectedRepo) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const val = newDefaultBranch.trim() || null;
      await invoke('update_repo_default_branch', { id: selectedRepo.id, defaultBranch: val });
      repos = repos.map(r => r.id === selectedRepo!.id ? { ...r, default_branch: val } : r);
      selectedRepo = { ...selectedRepo, default_branch: val };
      editingDefaultBranch = false;
    } catch (e) {
      console.error('Failed to save default branch:', e);
    }
  }

  function startEditDefaultBranch() {
    newDefaultBranch = selectedRepo?.default_branch ?? '';
    editingDefaultBranch = true;
  }

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

  async function openGitHub() {
    if (!selectedRepo) return;
    const url = getGitHubUrl(selectedRepo);
    if (!url) return;
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(url);
    } catch (e) {
      console.error('Failed to open URL:', e);
    }
  }

  function handleThemeChange(event: CustomEvent<string>) {
    currentTheme = event.detail;
    applyTheme(currentTheme);
  }

  function handleDataChange() {
    loadRepos();
  }

  loadRepos();
  loadTheme();
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
      onclick={refreshAllRepos}
      disabled={refreshingAll || repos.length === 0}
      title="Refresh all repos"
    >
      <RefreshCw size={14} class={refreshingAll ? 'spin' : ''} />
      Refresh All
    </button>
  </header>

  <div class="main">
    <Sidebar
      {repos}
      {groups}
      {selectedRepo}
      on:select={handleRepoSelect}
      on:dataChange={handleDataChange}
    />

    <main class="content">
      {#if activeTab === 'repos'}
        <div class="repos-view">
          {#if selectedRepo}
            <div class="repo-detail">
              <div class="repo-header">
                <div>
                  <h2>{selectedRepo.name}</h2>
                  <p class="repo-path">{selectedRepo.path}</p>
                </div>
                <div class="repo-actions">
                  <button
                    class="icon-btn"
                    onclick={() => refreshRepo(selectedRepo!.id)}
                    disabled={refreshingRepo === selectedRepo.id}
                    title="Refresh repo status"
                  >
                    <RefreshCw size={14} class={refreshingRepo === selectedRepo.id ? 'spin' : ''} />
                  </button>
                  <button
                    class="icon-btn"
                    onclick={() => fetchRepo(selectedRepo!.id)}
                    title="Fetch from remote"
                  >
                    <Download size={14} />
                  </button>
                  {#if getGitHubUrl(selectedRepo)}
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
                <span class="branch">{selectedRepo.current_branch}</span>
                {#if selectedRepo.sync_status}
                  <span
                    class="sync-badge"
                    class:up-to-date={selectedRepo.sync_status.up_to_date}
                    class:behind={selectedRepo.sync_status.behind > 0}
                    class:ahead={selectedRepo.sync_status.ahead > 0}
                    class:dirty={selectedRepo.sync_status.is_dirty}
                  >
                    {#if selectedRepo.sync_status.is_dirty}
                      Modified
                    {:else if selectedRepo.sync_status.up_to_date}
                      Up to date
                    {:else}
                      {selectedRepo.sync_status.behind} behind, {selectedRepo.sync_status.ahead} ahead
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
                      {selectedRepo.default_branch ?? globalDefaultBranch}
                    </button>
                  {/if}
                </div>
                {#if selectedRepo.remote_url}
                  <div class="meta-row">
                    <span class="meta-label">Remote</span>
                    <span class="meta-value mono">{selectedRepo.remote_url}</span>
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
        <PRList {repos} />
      {:else if activeTab === 'settings'}
        <Settings on:themeChange={handleThemeChange} on:dataChange={handleDataChange} />
      {/if}
    </main>
  </div>
</div>

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

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
