<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte';
  import PRList from './lib/components/PRList.svelte';
  import Settings from './lib/components/Settings.svelte';
  import type { Repository, RepoGroup } from './lib/types';
  import { FolderGit2, GitPullRequest, Settings as SettingsIcon } from '@lucide/svelte';

  let activeTab: 'repos' | 'prs' | 'settings' = $state('repos');
  let repos: Repository[] = $state([]);
  let groups: RepoGroup[] = $state([]);
  let selectedRepo: Repository | null = $state(null);
  let currentTheme: string = $state('dark');

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
      const config = await invoke('get_config') as { theme: string };
      currentTheme = config.theme;
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
              <h2>{selectedRepo.name}</h2>
              <p class="repo-path">{selectedRepo.path}</p>
              <div class="repo-status">
                <span class="branch">{selectedRepo.current_branch}</span>
                {#if selectedRepo.sync_status}
                  <span
                    class="sync-badge"
                    class:up-to-date={selectedRepo.sync_status.up_to_date}
                    class:behind={selectedRepo.sync_status.behind > 0}
                    class:ahead={selectedRepo.sync_status.ahead > 0}
                  >
                    {#if selectedRepo.sync_status.up_to_date}
                      Up to date
                    {:else}
                      {selectedRepo.sync_status.behind} behind, {selectedRepo.sync_status.ahead} ahead
                    {/if}
                  </span>
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
</style>
