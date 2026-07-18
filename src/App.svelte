<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte';
  import PRList from './lib/components/PRList.svelte';
  import Settings from './lib/components/Settings.svelte';
  import { app } from './lib/stores.svelte';
  import type { Repository } from './lib/types';
  import { FolderGit2, GitPullRequest, Settings as SettingsIcon, RefreshCw, Download, ExternalLink, GitMerge, History, AlertTriangle, X, Code, CircleDot, Info, ArrowDownToLine, MoreHorizontal, FolderOpen } from '@lucide/svelte';

  let activeTab: 'repos' | 'prs' | 'settings' = $state('repos');
  let expandedCommit: string | null = $state(null);
  let showRepoInfo: boolean = $state(false);
  let editingDefaultBranch: boolean = $state(false);
  let newDefaultBranch: string = $state('');
  let githubConnected: boolean = $state(false);
  let showOpenMenu: boolean = $state(false);

  async function checkGitHubAuth() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const auth = await invoke('get_github_auth') as { token: string | null };
      githubConnected = !!auth.token;
    } catch {
      githubConnected = false;
    }
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

  function getGitHubBranchUrl(repo: Repository): string | null {
    const baseUrl = getGitHubUrl(repo);
    if (!baseUrl) return null;
    return `${baseUrl}/tree/${repo.current_branch}`;
  }

  function getCommitUrl(repo: Repository, hash: string): string | null {
    const baseUrl = getGitHubUrl(repo);
    if (!baseUrl) return null;
    return `${baseUrl}/commit/${hash}`;
  }

  async function openUrl(url: string) {
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(url);
    } catch (e) {
      console.error('Failed to open URL:', e);
    }
  }

  async function openGitHub() {
    if (!app.selectedRepo) return;
    const url = getGitHubBranchUrl(app.selectedRepo);
    if (url) await openUrl(url);
  }

  async function openInVSCode() {
    if (!app.selectedRepo) return;
    try {
      const { Command } = await import('@tauri-apps/plugin-shell');
      await Command.create('open', ['-a', 'Visual Studio Code', app.selectedRepo.path]).execute();
    } catch (e) {
      console.error('Failed to open VSCode:', e);
    }
  }

  async function openInIntelliJ() {
    if (!app.selectedRepo) return;
    try {
      const { Command } = await import('@tauri-apps/plugin-shell');
      await Command.create('open', ['-a', 'IntelliJ IDEA', app.selectedRepo.path]).execute();
    } catch (e) {
      console.error('Failed to open IntelliJ:', e);
    }
  }

  async function openCommitUrl(hash: string) {
    if (!app.selectedRepo) return;
    const url = getCommitUrl(app.selectedRepo, hash);
    if (url) await openUrl(url);
  }

  async function saveDefaultBranch() {
    if (!app.selectedRepo) return;
    const val = newDefaultBranch.trim() || null;
    await app.saveDefaultBranch(app.selectedRepo.id, val);
    editingDefaultBranch = false;
  }

  function handleThemeChange(event: CustomEvent<string>) {
    app.updateConfig({ ...app.config, theme: event.detail });
  }

  function handleDataChange() {
    app.loadAll();
  }

  function handleClickOutside(e: MouseEvent) {
    if (showOpenMenu) {
      const target = e.target as HTMLElement;
      if (!target.closest('.open-menu-wrapper')) {
        showOpenMenu = false;
      }
    }
  }

  app.loadAll();
  checkGitHubAuth();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="app" onclick={handleClickOutside}>
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
      {#if githubConnected}
        <button
          class="tab"
          class:active={activeTab === 'prs'}
          onclick={() => activeTab = 'prs'}
        >
          <GitPullRequest size={16} />
          Pull Requests
        </button>
      {/if}
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
    {#if activeTab === 'repos'}
      <Sidebar />
    {/if}

    <main class="content">
      {#if activeTab === 'repos'}
        <div class="repos-view">
          {#if app.selectedRepo}
            <div class="repo-detail">
              <div class="repo-header">
                <div>
                  <h2>{app.selectedRepo.name}</h2>
                </div>
                <div class="repo-actions">
                  <button
                    class="action-btn"
                    onclick={() => app.refreshRepo(app.selectedRepo!.id)}
                    disabled={app.refreshingRepo === app.selectedRepo.id}
                    title="Refresh repo status"
                  >
                    <RefreshCw size={14} class={app.refreshingRepo === app.selectedRepo.id ? 'spin' : ''} />
                    <span>Refresh</span>
                  </button>
                  <button
                    class="action-btn"
                    onclick={() => app.fetchRepo(app.selectedRepo!.id)}
                    disabled={app.fetchingRepo === app.selectedRepo.id}
                    title="Fetch from remote"
                  >
                    <Download size={14} class={app.fetchingRepo === app.selectedRepo.id ? 'spin' : ''} />
                    <span>Fetch</span>
                  </button>
                  <button
                    class="action-btn"
                    onclick={() => app.pullRepo(app.selectedRepo!.id)}
                    title="Pull latest changes"
                  >
                    <ArrowDownToLine size={14} />
                    <span>Pull</span>
                  </button>
                  {#if app.selectedRepo.sync_status && app.selectedRepo.sync_status.behind > 0}
                    <button
                      class="action-btn merge-btn"
                      onclick={() => app.mergeRepo(app.selectedRepo!.id)}
                      disabled={app.mergingRepo === app.selectedRepo.id}
                      title="Merge changes from base branch"
                    >
                      <GitMerge size={14} class={app.mergingRepo === app.selectedRepo.id ? 'spin' : ''} />
                      <span>Merge</span>
                    </button>
                  {/if}
                  <div class="action-divider"></div>
                  <div class="open-menu-wrapper">
                    <button
                      class="action-btn"
                      onclick={() => showOpenMenu = !showOpenMenu}
                      title="Open in..."
                    >
                      <FolderOpen size={14} />
                      <span>Open</span>
                      <MoreHorizontal size={12} />
                    </button>
                    {#if showOpenMenu}
                      <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
                      <div class="open-menu" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
                        <button class="menu-item" onclick={() => { openInVSCode(); showOpenMenu = false; }}>
                          <Code size={14} />
                          VS Code
                        </button>
                        <button class="menu-item" onclick={() => { openInIntelliJ(); showOpenMenu = false; }}>
                          <CircleDot size={14} />
                          IntelliJ
                        </button>
                        {#if getGitHubUrl(app.selectedRepo)}
                          <button class="menu-item" onclick={() => { openGitHub(); showOpenMenu = false; }}>
                            <ExternalLink size={14} />
                            GitHub
                          </button>
                        {/if}
                      </div>
                    {/if}
                  </div>
                  <button
                    class="icon-btn"
                    onclick={() => showRepoInfo = true}
                    title="Repo details"
                  >
                    <Info size={14} />
                  </button>
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

{#if showRepoInfo && app.selectedRepo}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={() => showRepoInfo = false} onkeydown={(e) => e.key === 'Escape' && (showRepoInfo = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') { showRepoInfo = false; } else { e.stopPropagation(); } }}>
      <div class="modal-header">
        <h3>
          <Info size={18} />
          {app.selectedRepo.name}
        </h3>
        <button class="modal-close" onclick={() => showRepoInfo = false}>
          <X size={16} />
        </button>
      </div>
      <div class="modal-body">
        <div class="info-section-label">Information</div>
        <div class="info-rows">
          <div class="info-row">
            <span class="info-label">Path</span>
            <span class="info-value mono">{app.selectedRepo.path}</span>
          </div>
          <div class="info-row">
            <span class="info-label">Current Branch</span>
            <span class="info-value mono">{app.selectedRepo.current_branch}</span>
          </div>
          {#if app.selectedRepo.remote_url}
            <div class="info-row">
              <span class="info-label">Remote</span>
              <span class="info-value mono">{app.selectedRepo.remote_url}</span>
            </div>
          {/if}
        </div>

        <div class="info-divider"></div>

        <div class="info-section-label">Settings</div>
        <div class="info-rows">
          <div class="info-row">
            <span class="info-label">Default Branch</span>
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
              <button class="info-value editable" onclick={() => { editingDefaultBranch = true; newDefaultBranch = app.selectedRepo?.default_branch ?? ''; }}>
                {app.selectedRepo.default_branch ?? app.config.default_branch}
                <span class="edit-hint">(click to edit)</span>
              </button>
            {/if}
            <p class="info-hint">Override the global default branch for this repo</p>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if app.showMergeConflict && app.mergeResult}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={() => app.showMergeConflict = false} onkeydown={(e) => e.key === 'Escape' && (app.showMergeConflict = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') { app.showMergeConflict = false; } else { e.stopPropagation(); } }}>
      <div class="modal-header">
        <h3>
          <AlertTriangle size={18} />
          {#if app.mergeResult.success}
            Merge Successful
          {:else if app.mergeResult.conflicts && app.mergeResult.conflicts.length > 0}
            Merge Conflict
          {:else}
            Merge Failed
          {/if}
        </h3>
        <button class="modal-close" onclick={() => app.showMergeConflict = false}>
          <X size={16} />
        </button>
      </div>
      <div class="modal-body">
        {#if !app.mergeResult.success}
          <div class="merge-error-box">
            <pre>{app.mergeResult.message}</pre>
          </div>
        {:else}
          <p class="merge-message">{app.mergeResult.message}</p>
        {/if}
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

{#if app.showError && app.errorMsg}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={() => app.dismissError()} onkeydown={(e) => e.key === 'Escape' && app.dismissError()}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') { app.dismissError(); } else { e.stopPropagation(); } }}>
      <div class="modal-header">
        <h3>
          <AlertTriangle size={18} />
          Error
        </h3>
        <button class="modal-close" onclick={() => app.dismissError()}>
          <X size={16} />
        </button>
      </div>
      <div class="modal-body">
        <p class="error-message">{app.errorMsg}</p>
      </div>
      <div class="modal-actions">
        <button class="close-btn" onclick={() => app.dismissError()}>Close</button>
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
    flex-wrap: wrap;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
    white-space: nowrap;
  }

  .action-btn:hover:not(:disabled) {
    background-color: var(--accent);
    color: white;
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .action-btn.merge-btn {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .action-btn.merge-btn:hover:not(:disabled) {
    background-color: var(--success);
    color: white;
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

  .info-rows {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .info-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .info-value {
    font-size: 13px;
    color: var(--text-primary);
  }

  .info-value.mono {
    font-family: monospace;
    font-size: 12px;
    word-break: break-all;
  }

  .info-value.editable {
    background: transparent;
    text-align: left;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
  }

  .info-value.editable:hover {
    background-color: var(--bg-tertiary);
  }

  .info-section-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-bottom: 10px;
  }

  .info-divider {
    height: 1px;
    background-color: var(--border);
    margin: 16px 0;
  }

  .edit-hint {
    font-size: 11px;
    color: var(--text-secondary);
    opacity: 0.6;
    margin-left: 6px;
  }

  .info-hint {
    font-size: 12px;
    color: var(--text-secondary);
    opacity: 0.7;
    margin-top: 4px;
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

  .merge-error-box {
    background-color: rgba(242, 95, 76, 0.08);
    border: 1px solid rgba(242, 95, 76, 0.3);
    border-radius: 8px;
    padding: 12px;
    margin-bottom: 12px;
    overflow-x: auto;
  }

  .merge-error-box pre {
    margin: 0;
    font-size: 12px;
    color: var(--danger);
    white-space: pre-wrap;
    word-break: break-word;
    font-family: monospace;
    line-height: 1.5;
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

  .action-divider {
    width: 1px;
    height: 24px;
    background-color: var(--border);
    margin: 0 4px;
    align-self: center;
  }

  .open-menu-wrapper {
    position: relative;
    display: flex;
  }

  .open-menu-wrapper .action-btn {
    height: 100%;
  }

  .open-menu {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    min-width: 140px;
    z-index: 50;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    border-radius: 6px;
    text-align: left;
  }

  .menu-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .error-message {
    font-size: 13px;
    color: var(--text-primary);
    white-space: pre-wrap;
    word-break: break-word;
    font-family: monospace;
    background-color: var(--bg-tertiary);
    padding: 12px;
    border-radius: 8px;
    max-height: 300px;
    overflow-y: auto;
  }
</style>
