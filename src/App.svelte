<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte';
  import PRList from './lib/components/PRList.svelte';
  import Settings from './lib/components/Settings.svelte';
  import { app } from './lib/stores.svelte';
  import type { Repository } from './lib/types';
  import type { MergeResult } from './lib/tauri';
  import { DiffView, DiffModeEnum } from '@git-diff-view/svelte';
  import '@git-diff-view/svelte/styles/diff-view-pure.css';
  import { FolderGit2, GitPullRequest, Settings as SettingsIcon, RefreshCw, Download, ExternalLink, GitMerge, GitBranch, History, AlertTriangle, X, Code, CircleDot, Info, ArrowDownToLine, MoreHorizontal, FolderOpen, Upload, SquareStack, Check, ArrowRightLeft, CheckCircle, XCircle, ChevronDown, FileDiff } from '@lucide/svelte';

  let activeTab: 'repos' | 'prs' | 'settings' = $state('repos');
  let expandedCommit: string | null = $state(null);
  let showRepoInfo: boolean = $state(false);
  let editingDefaultBranch: boolean = $state(false);
  let newDefaultBranch: string = $state('');
  let githubConnected: boolean = $state(false);
  let showOpenMenu: boolean = $state(false);
  let showBranchMenu: boolean = $state(false);
  let commitMessage: string = $state('');
  let committing: boolean = $state(false);
  let pushing: boolean = $state(false);
  let mergeTargetBranch: string = $state('');
  let showDiffModal: boolean = $state(false);
  let diffViewMode: 'split' | 'unified' = $state('split');

  function parseDiffContent(rawDiff: string) {
    if (!rawDiff) return null;
    
    let oldFileName = '';
    let newFileName = '';
    
    const diffMatch = rawDiff.match(/diff --git a\/(.+?) b\/(.+)/);
    if (diffMatch) {
      oldFileName = diffMatch[1];
      newFileName = diffMatch[2];
    }
    
    return {
      oldFile: { fileName: oldFileName || app.diffFile || 'file' },
      newFile: { fileName: newFileName || app.diffFile || 'file' },
      hunks: [rawDiff]
    };
  }

  let parsedDiff = $derived(parseDiffContent(app.diffContent));

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
    if (showBranchMenu) {
      const target = e.target as HTMLElement;
      if (!target.closest('.branch-menu-wrapper')) {
        showBranchMenu = false;
      }
    }
  }

  async function handleCommit() {
    if (!app.selectedRepo || !commitMessage.trim()) return;
    committing = true;
    try {
      await app.commit(app.selectedRepo.id, commitMessage.trim());
      commitMessage = '';
    } finally {
      committing = false;
    }
  }

  async function handlePush() {
    if (!app.selectedRepo) return;
    pushing = true;
    try {
      await app.pushRepo(app.selectedRepo.id);
    } finally {
      pushing = false;
    }
  }

  async function handleMergeTarget() {
    if (!app.selectedRepo || !mergeTargetBranch.trim()) return;
    app.showMergeConflict = false;
    // Try to merge the specified branch
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const result = await invoke('merge_repo_with_target', {
        id: app.selectedRepo.id,
        target: mergeTargetBranch.trim()
      }) as MergeResult;
      app.mergeResult = result;
      if (result.success) {
        app.showNotification('success', `Merged ${mergeTargetBranch} successfully`);
        await app.refreshRepo(app.selectedRepo.id);
        await app.loadGitLog(app.selectedRepo.id);
      } else {
        app.showMergeConflict = true;
      }
    } catch (e) {
      app.mergeResult = { success: false, message: String(e) };
      app.showMergeConflict = true;
    }
    mergeTargetBranch = '';
  }

  async function handlePullInstead() {
    if (!app.selectedRepo) return;
    app.showMergeConflict = false;
    await app.pullRepo(app.selectedRepo.id);
  }

  async function handleStashAndMerge() {
    if (!app.selectedRepo) return;
    try {
      await app.stashRepo(app.selectedRepo.id);
      app.showMergeConflict = false;
      await app.mergeRepo(app.selectedRepo.id);
    } catch (e) {
      app.showNotification('error', `Stash failed: ${e}`);
    }
  }

  async function handlePullStrategy(rebase: boolean) {
    if (!app.selectedRepo) return;
    app.showPullStrategy = false;
    await app.pullRepo(app.selectedRepo.id, rebase);
  }

  app.loadAll();
  checkGitHubAuth().then(() => {
    if (githubConnected) app.loadPRs();
  });
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
              <div class="repo-card">
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
                    <button
                      class="action-btn"
                      onclick={handlePush}
                      disabled={pushing}
                      title="Push to remote"
                    >
                      <Upload size={14} class={pushing ? 'spin' : ''} />
                      <span>Push</span>
                    </button>
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
                  <div class="branch-menu-wrapper">
                    <button
                      class="branch-btn"
                      onclick={() => showBranchMenu = !showBranchMenu}
                      title="Switch branch"
                    >
                      <span class="branch">{app.selectedRepo.current_branch}</span>
                      <ChevronDown size={12} />
                    </button>
                    {#if showBranchMenu}
                      <div class="branch-menu" onclick={(e) => e.stopPropagation()}>
                        {#if app.selectedRepo.local_branches.length > 0}
                          <div class="branch-section-label">
                            <span class="section-icon">L</span>
                            Local
                          </div>
                          {#each app.selectedRepo.local_branches as br}
                            <button
                              class="branch-menu-item"
                              class:active={br === app.selectedRepo.current_branch}
                              onclick={() => { app.switchBranch(app.selectedRepo!.id, br); showBranchMenu = false; }}
                            >
                              <span class="branch-name">{br}</span>
                              {#if br === app.selectedRepo.current_branch}
                                <Check size={12} />
                              {/if}
                            </button>
                          {/each}
                        {/if}
                        {#if app.selectedRepo.remote_branches.length > 0}
                          <div class="branch-section-label remote-section">
                            <span class="section-icon">R</span>
                            Remote
                          </div>
                          {#each app.selectedRepo.remote_branches as br}
                            {@const isLocal = app.selectedRepo.local_branches.includes(br)}
                            <button
                              class="branch-menu-item remote-item"
                              class:active={!isLocal && br === app.selectedRepo.current_branch}
                              onclick={() => { app.switchBranch(app.selectedRepo!.id, br); showBranchMenu = false; }}
                            >
                              <span class="branch-name">{br}</span>
                              {#if isLocal}
                                <span class="branch-tag local-tag">local</span>
                              {:else if br === app.selectedRepo.current_branch}
                                <Check size={12} />
                              {/if}
                            </button>
                          {/each}
                        {/if}
                      </div>
                    {/if}
                  </div>
                  {#if app.selectedRepo.sync_status}
                    <span
                      class="sync-badge"
                      class:up-to-date={app.selectedRepo.sync_status.up_to_date}
                      class:behind={app.selectedRepo.sync_status.behind > 0}
                      class:ahead={app.selectedRepo.sync_status.ahead > 0}
                      class:dirty={app.selectedRepo.sync_status.is_dirty}
                    >
                      {#if app.selectedRepo.sync_status.behind > 0}
                        {app.selectedRepo.sync_status.behind} behind
                      {:else if app.selectedRepo.sync_status.is_dirty}
                        Uncommitted changes
                      {:else}
                        Up to date
                      {/if}
                    </span>
                  {/if}
                </div>
              </div>

              {#if app.gitStatus && (app.gitStatus.staged.length > 0 || app.gitStatus.unstaged.length > 0)}
                <div class="repo-card">
                  <div class="file-changes">
                    <h3 class="section-title">
                      <FileDiff size={14} />
                      Changes
                      {#if app.gitStatus.merge_in_progress}
                        <span class="merge-tag">MERGING</span>
                      {/if}
                    </h3>
                    <div class="file-changes-split">
                      {#if app.gitStatus.unstaged.length > 0}
                        <div class="file-group">
                          <span class="file-group-label">Unstaged</span>
                          {#each app.gitStatus.unstaged as file (file.path)}
                            <div class="file-row clickable" onclick={() => { app.loadDiff(app.selectedRepo!.id, file.path, false); showDiffModal = true; }}>
                              <span class="file-change-tag unstaged">{file.change}</span>
                              <span class="file-path">{file.path}</span>
                              {#if file.change === 'U'}
                                <button
                                  class="file-action-btn accept"
                                  onclick={(e) => { e.stopPropagation(); app.resolveFileConflict(app.selectedRepo!.id, file.path, 'ours'); }}
                                  title="Accept ours"
                                >
                                  <CheckCircle size={12} />
                                </button>
                                <button
                                  class="file-action-btn accept"
                                  onclick={(e) => { e.stopPropagation(); app.resolveFileConflict(app.selectedRepo!.id, file.path, 'theirs'); }}
                                  title="Accept theirs"
                                >
                                  <XCircle size={12} />
                                </button>
                              {:else}
                                <button
                                  class="file-action-btn"
                                  onclick={(e) => { e.stopPropagation(); app.stageFile(app.selectedRepo!.id, file.path); }}
                                  title="Stage"
                                >
                                  <ArrowRightLeft size={12} />
                                </button>
                                <button
                                  class="file-action-btn danger"
                                  onclick={(e) => { e.stopPropagation(); app.discardFile(app.selectedRepo!.id, file.path); }}
                                  title="Discard changes"
                                >
                                  <X size={12} />
                                </button>
                              {/if}
                            </div>
                          {/each}
                        </div>
                      {/if}
                      {#if app.gitStatus.staged.length > 0}
                        <div class="file-group">
                          <span class="file-group-label">Staged</span>
                          {#each app.gitStatus.staged as file (file.path)}
                            <div class="file-row clickable" onclick={() => { app.loadDiff(app.selectedRepo!.id, file.path, true); showDiffModal = true; }}>
                              <span class="file-change-tag staged">{file.change}</span>
                              <span class="file-path">{file.path}</span>
                              <button
                                class="file-action-btn"
                                onclick={(e) => { e.stopPropagation(); app.unstageFile(app.selectedRepo!.id, file.path); }}
                                title="Unstage"
                              >
                                <X size={12} />
                              </button>
                            </div>
                          {/each}
                        </div>
                      {/if}
                    </div>

                    <div class="commit-area">
                      <div class="stage-all-actions">
                        <button class="small-btn" onclick={() => app.stageAll(app.selectedRepo!.id)}>
                          Stage All
                        </button>
                        <button class="small-btn" onclick={() => app.stashRepo(app.selectedRepo!.id)}>
                          <SquareStack size={12} />
                          Stash
                        </button>
                      </div>
                      <div class="commit-input-row">
                        <input
                          class="commit-input"
                          type="text"
                          bind:value={commitMessage}
                          placeholder="Commit message..."
                          onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleCommit(); } }}
                        />
                        <button
                          class="commit-btn"
                          onclick={handleCommit}
                          disabled={committing || !commitMessage.trim()}
                        >
                          {committing ? '...' : 'Commit'}
                        </button>
                      </div>
                      {#if app.gitStatus.merge_in_progress && !app.gitStatus.has_conflicts}
                        <button
                          class="continue-merge-btn"
                          onclick={() => app.continueMerge(app.selectedRepo!.id)}
                        >
                          Complete Merge
                        </button>
                      {/if}
                    </div>
                  </div>
                </div>
              {/if}

              <div class="repo-card">
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

              {#if app.selectedRepo.github_owner && app.selectedRepo.github_repo}
                {@const repoPrs = app.prs.filter(pr => pr.repo_id === app.selectedRepo!.id)}
                {#if repoPrs.length > 0}
                  <div class="repo-card">
                    <div class="repo-prs">
                      <h3 class="section-title">
                        <GitPullRequest size={14} />
                        Pull Requests
                        <a
                          class="view-all-link"
                          href="https://github.com/{app.selectedRepo.github_owner}/{app.selectedRepo.github_repo}/pulls"
                          onclick={(e) => { e.preventDefault(); openUrl(`https://github.com/${app.selectedRepo!.github_owner}/${app.selectedRepo!.github_repo}/pulls`); }}
                        >
                          View all
                          <ExternalLink size={10} />
                        </a>
                      </h3>
                      <div class="repo-pr-list">
                        {#each repoPrs as pr (pr.id)}
                          <div class="repo-pr-row">
                            <span class="repo-pr-status" class:success={pr.checks_status === 'success'} class:failure={pr.checks_status === 'failure'} class:pending={pr.checks_status === 'pending'}></span>
                            <span class="repo-pr-number">#{pr.number}</span>
                            <span class="repo-pr-title">{pr.title}</span>
                            {#if pr.html_url}
                              <button
                                class="repo-pr-link"
                                onclick={() => openUrl(pr.html_url)}
                                title="Open in GitHub"
                              >
                                <ExternalLink size={12} />
                              </button>
                            {/if}
                          </div>
                        {/each}
                      </div>
                    </div>
                  </div>
                {/if}
              {/if}
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
        <Settings on:themeChange={handleThemeChange} on:dataChange={handleDataChange} on:authChange={checkGitHubAuth} />
      {/if}
    </main>
  </div>
</div>

{#if app.notification}
  <div class="notification-toast" class:success={app.notification.type === 'success'} class:error={app.notification.type === 'error'}>
    {#if app.notification.type === 'success'}
      <CheckCircle size={14} />
    {:else}
      <XCircle size={14} />
    {/if}
    <span>{app.notification.message}</span>
  </div>
{/if}

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
          {#if app.mergeResult.message.includes('not something we can merge')}
            <div class="merge-options">
              <p>The target branch doesn't exist or can't be merged. Try:</p>
              <div class="merge-option-actions">
                <div class="merge-option-row">
                  <input
                    class="merge-target-input"
                    type="text"
                    bind:value={mergeTargetBranch}
                    placeholder="branch name (e.g. develop)"
                  />
                  <button
                    class="merge-option-btn"
                    onclick={() => handleMergeTarget()}
                    disabled={!mergeTargetBranch.trim()}
                  >
                    Merge Branch
                  </button>
                </div>
                <button class="merge-option-btn secondary" onclick={() => handlePullInstead()}>
                  <ArrowDownToLine size={14} />
                  Pull Instead
                </button>
              </div>
            </div>
          {:else if app.mergeResult.message.includes('uncommitted changes')}
            <div class="merge-options">
              <p>You have uncommitted changes. Resolve them first:</p>
              <div class="merge-option-actions">
                <button class="merge-option-btn" onclick={() => handleStashAndMerge()}>
                  <SquareStack size={14} />
                  Stash & Merge
                </button>
              </div>
            </div>
          {:else}
            <div class="merge-options">
              <div class="merge-option-actions">
                <button class="merge-option-btn secondary" onclick={() => handlePullInstead()}>
                  <ArrowDownToLine size={14} />
                  Pull Instead
                </button>
              </div>
            </div>
          {/if}
        {:else}
          <p class="merge-message">{app.mergeResult.message}</p>
        {/if}
        {#if app.mergeResult.conflicts && app.mergeResult.conflicts.length > 0}
          <div class="conflict-list">
            <p class="conflict-title">Conflicting files — click to resolve:</p>
            {#each app.mergeResult.conflicts as conflict}
              <div class="conflict-item">
                <span class="conflict-file">{conflict.file}</span>
                <div class="conflict-actions">
                  <button
                    class="conflict-resolve-btn ours"
                    onclick={() => app.resolveFileConflict(app.selectedRepo!.id, conflict.file, 'ours')}
                    title="Accept our version"
                  >
                    Ours
                  </button>
                  <button
                    class="conflict-resolve-btn theirs"
                    onclick={() => app.resolveFileConflict(app.selectedRepo!.id, conflict.file, 'theirs')}
                    title="Accept their version"
                  >
                    Theirs
                  </button>
                </div>
              </div>
            {/each}
          </div>
          <p class="conflict-hint">After resolving all conflicts, click Complete Merge.</p>
        {/if}
      </div>
      <div class="modal-actions">
        {#if app.mergeResult.conflicts && app.mergeResult.conflicts.length > 0}
          <button class="abort-btn" onclick={() => app.abortMerge(app.selectedRepo!.id)}>
            Abort Merge
          </button>
          <button class="continue-btn" onclick={() => app.continueMerge(app.selectedRepo!.id)}>
            Complete Merge
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

{#if app.showPullStrategy && app.pullResult}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={() => app.showPullStrategy = false} onkeydown={(e) => e.key === 'Escape' && (app.showPullStrategy = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') { app.showPullStrategy = false; } else { e.stopPropagation(); } }}>
      <div class="modal-header">
        <h3>
          <AlertTriangle size={18} />
          Divergent Branches
        </h3>
        <button class="modal-close" onclick={() => app.showPullStrategy = false}>
          <X size={16} />
        </button>
      </div>
      <div class="modal-body">
        <div class="merge-error-box">
          <pre>{app.pullResult.message}</pre>
        </div>
        <p class="merge-options-title">Choose a pull strategy:</p>
        <div class="pull-strategy-options">
          <button class="strategy-btn" onclick={() => handlePullStrategy(false)}>
            <GitMerge size={16} />
            <div class="strategy-info">
              <span class="strategy-name">Merge</span>
              <span class="strategy-desc">Create a merge commit</span>
            </div>
          </button>
          <button class="strategy-btn" onclick={() => handlePullStrategy(true)}>
            <GitBranch size={16} />
            <div class="strategy-info">
              <span class="strategy-name">Rebase</span>
              <span class="strategy-desc">Replay commits on top</span>
            </div>
          </button>
        </div>
      </div>
      <div class="modal-actions">
        <button class="close-btn" onclick={() => app.showPullStrategy = false}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

{#if showDiffModal && app.diffFile}
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal-overlay" onclick={() => { showDiffModal = false; app.diffFile = null; }} onkeydown={(e) => e.key === 'Escape' && (showDiffModal = false)}>
    <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content diff-modal" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') { showDiffModal = false; } else { e.stopPropagation(); } }}>
      <div class="modal-header">
        <h3>
          <FileDiff size={18} />
          {app.diffStaged ? 'Staged' : 'Unstaged'}: {app.diffFile}
        </h3>
        <div class="diff-view-controls">
          <button 
            class="diff-mode-btn" 
            class:active={diffViewMode === 'split'}
            onclick={() => diffViewMode = 'split'}
          >
            Split
          </button>
          <button 
            class="diff-mode-btn" 
            class:active={diffViewMode === 'unified'}
            onclick={() => diffViewMode = 'unified'}
          >
            Unified
          </button>
        </div>
        <button class="modal-close" onclick={() => { showDiffModal = false; app.diffFile = null; }}>
          <X size={16} />
        </button>
      </div>
      <div class="modal-body diff-body">
        {#if app.loadingDiff}
          <p class="loading-text">Loading diff...</p>
        {:else if !app.diffContent || !parsedDiff}
          <p class="empty-text">No changes to display</p>
        {:else}
          <DiffView 
            data={parsedDiff}
            diffViewMode={diffViewMode === 'split' ? DiffModeEnum.Split : DiffModeEnum.Unified}
            diffViewTheme="dark"
            diffViewHighlight={true}
          />
        {/if}
      </div>
      <div class="modal-actions">
        {#if !app.diffStaged && app.diffFile}
          <button class="merge-option-btn" onclick={() => { app.stageFile(app.selectedRepo!.id, app.diffFile!); showDiffModal = false; }}>
            <ArrowRightLeft size={14} />
            Stage
          </button>
        {:else if app.diffStaged && app.diffFile}
          <button class="merge-option-btn secondary" onclick={() => { app.unstageFile(app.selectedRepo!.id, app.diffFile!); showDiffModal = false; }}>
            <X size={14} />
            Unstage
          </button>
        {/if}
        <button class="close-btn" onclick={() => { showDiffModal = false; app.diffFile = null; }}>Close</button>
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
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .repo-card {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
  }

  .repo-card h2 {
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
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
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
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
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

  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 10px;
  }

  .view-all-link {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    font-weight: 500;
    color: var(--accent);
    text-decoration: none;
  }

  .view-all-link:hover {
    text-decoration: underline;
  }

  .repo-prs {
    display: flex;
    flex-direction: column;
  }

  .repo-pr-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .repo-pr-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 12px;
  }

  .repo-pr-row:hover {
    background-color: var(--bg-tertiary);
  }

  .repo-pr-status {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
    background-color: var(--text-secondary);
  }

  .repo-pr-status.success { background-color: var(--success); }
  .repo-pr-status.failure { background-color: var(--danger); }
  .repo-pr-status.pending { background-color: var(--warning); }

  .repo-pr-number {
    color: var(--text-secondary);
    font-weight: 600;
    flex-shrink: 0;
  }

  .repo-pr-title {
    flex: 1;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .repo-pr-link {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s;
  }

  .repo-pr-row:hover .repo-pr-link {
    opacity: 1;
  }

  .repo-pr-link:hover {
    background-color: var(--accent);
    color: white;
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
    width: 90vw;
    max-width: 700px;
    max-height: 85vh;
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

  .notification-toast {
    position: fixed;
    bottom: 20px;
    right: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 500;
    z-index: 200;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    animation: slideUp 0.2s ease-out;
  }

  .notification-toast.success {
    background-color: rgba(44, 182, 125, 0.95);
    color: white;
  }

  .notification-toast.error {
    background-color: rgba(242, 95, 76, 0.95);
    color: white;
  }

  @keyframes slideUp {
    from { transform: translateY(20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .branch-menu-wrapper {
    position: relative;
  }

  .branch-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 12px;
    font-family: monospace;
    font-weight: 500;
  }

  .branch-btn:hover {
    background-color: var(--border);
  }

  .branch-menu {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    min-width: 200px;
    max-height: 250px;
    overflow-y: auto;
    z-index: 50;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
    padding: 4px;
  }

  .branch-menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    font-family: monospace;
    border-radius: 4px;
  }

  .branch-menu-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .branch-menu-item.active {
    background: var(--bg-tertiary);
    font-weight: 600;
  }
  .branch-menu-item:hover {
    background: var(--bg-tertiary);
  }
  .branch-section-label {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 7px 10px 3px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-secondary);
    user-select: none;
  }
  .branch-section-label:first-child {
    padding-top: 4px;
  }
  .section-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 3px;
    font-size: 8px;
    font-weight: 800;
    background: var(--text-secondary);
    color: var(--bg-primary);
  }
  .remote-section .section-icon {
    background: var(--info);
    color: white;
  }
  .remote-item {
    opacity: 0.75;
  }
  .remote-item:hover {
    opacity: 1;
  }
  .branch-name {
    flex: 1;
    text-align: left;
  }
  .branch-tag {
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 4px;
    font-weight: 600;
    margin-left: 6px;
  }
  .local-tag {
    background: var(--bg-tertiary);
    color: var(--text-secondary);
  }

  .file-changes-split {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  @media (min-width: 900px) {
    .file-changes-split {
      flex-direction: row;
    }
    .file-changes-split .file-group {
      flex: 1;
      min-width: 0;
    }
  }

  .merge-tag {
    background-color: rgba(249, 188, 96, 0.2);
    color: var(--warning);
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 4px;
    letter-spacing: 0.05em;
  }

  .file-group {
    margin-bottom: 8px;
  }

  .file-group-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: block;
    margin-bottom: 4px;
    padding-left: 4px;
  }

  .file-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 4px;
    border-radius: 4px;
    font-size: 12px;
  }

  .file-row:hover {
    background-color: var(--bg-tertiary);
  }

  .file-row.clickable {
    cursor: pointer;
  }

  .file-change-tag {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 18px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    font-family: monospace;
    flex-shrink: 0;
  }

  .file-change-tag.staged {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .file-change-tag.unstaged {
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
  }

  .file-path {
    flex: 1;
    font-family: monospace;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: transparent;
    color: var(--text-secondary);
    border-radius: 4px;
    padding: 0;
    flex-shrink: 0;
  }

  .file-action-btn:hover {
    background-color: var(--bg-secondary);
    color: var(--text-primary);
  }

  .file-action-btn.accept {
    color: var(--success);
  }

  .file-action-btn.accept:hover {
    background-color: rgba(44, 182, 125, 0.12);
  }

  .file-action-btn.danger {
    color: var(--danger);
  }

  .file-action-btn.danger:hover {
    background-color: rgba(242, 95, 76, 0.12);
  }

  .commit-area {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .stage-all-actions {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
  }

  .small-btn {
    padding: 3px 8px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 500;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .small-btn:hover {
    background-color: var(--border);
    color: var(--text-primary);
  }

  .commit-input-row {
    display: flex;
    gap: 6px;
  }

  .commit-input {
    flex: 1;
    padding: 6px 10px;
    font-size: 13px;
  }

  .commit-btn {
    padding: 6px 14px;
    background-color: var(--success);
    color: white;
    font-size: 12px;
    font-weight: 600;
    border-radius: 6px;
    white-space: nowrap;
  }

  .commit-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .commit-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .continue-merge-btn {
    width: 100%;
    margin-top: 8px;
    padding: 8px 14px;
    background-color: rgba(44, 182, 125, 0.12);
    color: var(--success);
    font-size: 13px;
    font-weight: 600;
    border-radius: 6px;
    border: 1px solid var(--success);
  }

  .continue-merge-btn:hover {
    background-color: var(--success);
    color: white;
  }

  .conflict-actions {
    display: flex;
    gap: 4px;
    margin-left: auto;
  }

  .conflict-resolve-btn {
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 600;
    border-radius: 4px;
  }

  .conflict-resolve-btn.ours {
    background-color: rgba(127, 90, 240, 0.15);
    color: var(--accent);
  }

  .conflict-resolve-btn.ours:hover {
    background-color: var(--accent);
    color: white;
  }

  .conflict-resolve-btn.theirs {
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
  }

  .conflict-resolve-btn.theirs:hover {
    background-color: var(--warning);
    color: white;
  }

  .merge-options {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }

  .merge-options p {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 10px;
  }

  .merge-option-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .merge-option-row {
    display: flex;
    gap: 6px;
  }

  .merge-target-input {
    flex: 1;
    padding: 6px 10px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 13px;
  }

  .merge-target-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .merge-option-btn {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: var(--accent);
    color: white;
  }

  .merge-option-btn:hover {
    opacity: 0.9;
  }

  .merge-option-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .merge-option-btn.secondary {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .merge-option-btn.secondary:hover {
    background-color: var(--border);
  }

  .merge-options-title {
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 10px;
  }

  .pull-strategy-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .strategy-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-radius: 8px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    cursor: pointer;
    text-align: left;
  }

  .strategy-btn:hover {
    border-color: var(--accent);
    background-color: var(--bg-secondary);
  }

  .strategy-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .strategy-name {
    font-weight: 600;
    color: var(--text-primary);
    font-size: 14px;
  }

  .strategy-desc {
    color: var(--text-secondary);
    font-size: 12px;
  }

  .diff-modal {
    max-width: 900px;
    width: 95vw;
  }

  .diff-body {
    padding: 0;
    overflow: hidden;
  }

  .diff-body :global(.diff-container) {
    max-height: 60vh;
    overflow: auto;
  }

  .diff-view-controls {
    display: flex;
    gap: 4px;
    margin-right: auto;
    margin-left: 12px;
  }

  .diff-mode-btn {
    padding: 4px 10px;
    font-size: 11px;
    font-weight: 500;
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
  }

  .diff-mode-btn:hover {
    background: var(--border);
  }

  .diff-mode-btn.active {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
</style>
