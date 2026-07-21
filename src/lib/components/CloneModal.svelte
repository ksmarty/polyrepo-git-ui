<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { X, Loader2, Lock, Globe, Search, ChevronDown } from '@lucide/svelte';
  import { app } from '../stores.svelte';

  const dispatch = createEventDispatcher<{
    close: void;
    complete: void;
  }>();

  let url: string = $state('');
  let path: string = $state('');
  let cloning: boolean = $state(false);
  let error: string | null = $state(null);
  let githubConnected: boolean = $state(false);
  let repos: { name: string; full_name: string; clone_url: string; html_url: string; private: boolean }[] = $state([]);
  let filteredRepos: typeof repos = $state([]);
  let searchQuery: string = $state('');
  let showDropdown: boolean = $state(false);
  let loadingRepos: boolean = $state(false);
  let selectedRepo: typeof repos[0] | null = $state(null);
  let dropdownRef: HTMLDivElement | null = $state(null);

  onMount(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const auth = await invoke('get_github_auth') as { token: string | null };
      githubConnected = !!auth.token;
      if (githubConnected && auth.token) {
        await fetchUserRepos(auth.token);
      }
    } catch {
      githubConnected = false;
    }
  });

  async function fetchUserRepos(token: string) {
    loadingRepos = true;
    try {
      const headers = { Authorization: `Bearer ${token}`, Accept: 'application/vnd.github.v3+json' };
      let allRepos: typeof repos = [];
      let page = 1;
      const perPage = 100;

      while (true) {
        const res = await fetch(`https://api.github.com/user/repos?per_page=${perPage}&page=${page}&sort=updated&direction=desc`, { headers });
        if (!res.ok) break;
        const data = await res.json();
        if (!data.length) break;
        allRepos = allRepos.concat(
          data.map((r: any) => ({
            name: r.name,
            full_name: r.full_name,
            clone_url: r.clone_url,
            html_url: r.html_url,
            private: r.private,
          }))
        );
        if (data.length < perPage) break;
        page++;
      }

      repos = allRepos;
      filteredRepos = allRepos;
    } catch {
      repos = [];
      filteredRepos = [];
    }
    loadingRepos = false;
  }

  function filterRepos() {
    const q = searchQuery.toLowerCase().trim();
    if (!q) {
      filteredRepos = repos;
    } else {
      filteredRepos = repos.filter(r =>
        r.full_name.toLowerCase().includes(q) || r.name.toLowerCase().includes(q)
      );
    }
  }

  function selectRepo(repo: typeof repos[0]) {
    selectedRepo = repo;
    url = repo.clone_url;
    searchQuery = repo.full_name;
    showDropdown = false;
    updatePathFromUrl();
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      showDropdown = false;
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      showDropdown = true;
    }
  }

  function extractRepoName(repoUrl: string): string {
    const cleaned = repoUrl.replace(/\.git$/, '').replace(/\/$/, '');
    const parts = cleaned.split('/');
    return parts[parts.length - 1] || 'repo';
  }

  function updatePathFromUrl() {
    if (!url) {
      path = '';
      return;
    }
    const repoName = extractRepoName(url);
    const baseDir = app.config.default_repo_location || '~/Projects';
    path = `${baseDir}/${repoName}`;
  }

  function clearSearch() {
    searchQuery = '';
    selectedRepo = null;
    filteredRepos = repos;
    url = '';
    path = '';
  }

  async function handleClone() {
    if (!url.trim() || !path.trim()) return;
    cloning = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clone_repo', { url: url.trim(), path: path.trim() });
      dispatch('complete');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to clone repository';
      cloning = false;
    }
  }

  function handleModalClick(e: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
      showDropdown = false;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="modal-overlay" role="presentation" onclick={() => dispatch('close')} onkeydown={(e) => e.key === 'Escape' && dispatch('close')}>
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => { e.stopPropagation(); handleModalClick(e); }} onkeydown={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h3>Clone Repository</h3>
      <button class="close-btn" onclick={() => dispatch('close')}>
        <X size={18} />
      </button>
    </div>

    <div class="modal-body">
      {#if error}
        <div class="error">{error}</div>
      {/if}

      {#if githubConnected}
        <div class="auth-badge">
          <Lock size={14} />
          <span>GitHub connected — private repos are accessible</span>
        </div>
      {:else}
        <div class="auth-badge public">
          <Globe size={14} />
          <span>Not connected — only public repos can be cloned</span>
        </div>
      {/if}

      {#if githubConnected}
        <div class="form-group">
          <label for="repo-search">Search your repositories</label>
          <div class="search-wrapper" bind:this={dropdownRef}>
            <div class="search-input-wrapper">
              <Search size={14} class="search-icon" />
              <input
                id="repo-search"
                type="text"
                bind:value={searchQuery}
                oninput={() => { filterRepos(); showDropdown = true; selectedRepo = null; }}
                onfocus={() => showDropdown = true}
                onkeydown={handleSearchKeydown}
                placeholder="Search repos by name..."
                class="search-input"
                autocomplete="off"
              />
              {#if searchQuery}
                <button class="clear-btn" onclick={clearSearch}>
                  <X size={12} />
                </button>
              {/if}
              <ChevronDown size={14} class="chevron" />
            </div>

            {#if showDropdown && filteredRepos.length > 0}
              <div class="dropdown">
                {#each filteredRepos.slice(0, 50) as repo}
                  <button
                    class="dropdown-item"
                    class:selected={selectedRepo?.full_name === repo.full_name}
                    onclick={() => selectRepo(repo)}
                  >
                    <span class="repo-name">{repo.name}</span>
                    <span class="repo-org">{repo.full_name.split('/')[0]}</span>
                    {#if repo.private}
                      <span class="private-badge">Private</span>
                    {/if}
                  </button>
                {/each}
                {#if filteredRepos.length > 50}
                  <div class="dropdown-footer">Showing 50 of {filteredRepos.length} repos</div>
                {/if}
              </div>
            {/if}

            {#if showDropdown && searchQuery && filteredRepos.length === 0 && !loadingRepos}
              <div class="dropdown">
                <div class="dropdown-empty">No repos found</div>
              </div>
            {/if}

            {#if loadingRepos}
              <div class="loading-repos">
                <Loader2 size={14} class="spin" />
                Loading repos...
              </div>
            {/if}
          </div>
        </div>

        <div class="divider">
          <span>or enter a URL manually</span>
        </div>
      {/if}

      <div class="form-group">
        <label for="clone-url">Repository URL</label>
        <input
          id="clone-url"
          type="text"
          bind:value={url}
          oninput={() => { selectedRepo = null; updatePathFromUrl(); }}
          placeholder="https://github.com/user/repo.git"
        />
        <p class="hint">HTTPS or SSH URL</p>
      </div>

      <div class="form-group">
        <label for="clone-path">Clone to</label>
        <input
          id="clone-path"
          type="text"
          bind:value={path}
          placeholder={app.config.default_repo_location ? `${app.config.default_repo_location}/repo-name` : '~/Projects/repo-name'}
        />
        <p class="hint">Local path where the repo will be cloned</p>
      </div>
    </div>

    <div class="modal-footer">
      <button class="secondary-btn" onclick={() => dispatch('close')}>Cancel</button>
      <button
        class="primary-btn"
        onclick={handleClone}
        disabled={cloning || !url.trim() || !path.trim()}
      >
        {#if cloning}
          <span class="spin"><Loader2 size={14} /></span>
          Cloning...
        {:else}
          Clone
        {/if}
      </button>
    </div>
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .modal {
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 14px;
    width: 540px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h3 {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    color: var(--text-secondary);
    padding: 4px;
    border-radius: 6px;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background-color: var(--bg-tertiary);
  }

  .modal-body {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
  }

  .error {
    background-color: rgba(242, 95, 76, 0.1);
    border: 1px solid var(--danger);
    color: var(--danger);
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 13px;
    margin-bottom: 16px;
  }

  .auth-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 13px;
    margin-bottom: 16px;
    background-color: rgba(44, 182, 125, 0.1);
    border: 1px solid var(--success);
    color: var(--success);
  }

  .auth-badge.public {
    background-color: rgba(249, 188, 96, 0.1);
    border: 1px solid var(--warning);
    color: var(--warning);
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 6px;
    color: var(--text-secondary);
  }

  .form-group input {
    width: 100%;
    padding: 8px 12px;
    font-size: 13px;
    font-family: monospace;
  }

  .hint {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 4px;
    opacity: 0.7;
  }

  .search-wrapper {
    position: relative;
  }

  .search-input-wrapper {
    display: flex;
    align-items: center;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0 10px;
    transition: border-color 0.15s;
  }

  .search-input-wrapper:focus-within {
    border-color: var(--accent);
  }

  .search-icon {
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    padding: 8px 8px;
    font-size: 13px;
    font-family: monospace;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    width: 100%;
  }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    color: var(--text-secondary);
    padding: 2px;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .clear-btn:hover {
    color: var(--text-primary);
    background-color: var(--bg-tertiary);
  }

  .chevron {
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: transform 0.15s;
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    margin-top: 4px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
    max-height: 200px;
    overflow-y: auto;
    z-index: 10;
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 12px;
    font-size: 13px;
    text-align: left;
    background: transparent;
    color: var(--text-primary);
    border: none;
    cursor: pointer;
    transition: background-color 0.1s;
  }

  .dropdown-item:hover {
    background-color: var(--bg-tertiary);
  }

  .dropdown-item.selected {
    background-color: rgba(127, 90, 240, 0.1);
  }

  .repo-name {
    font-family: monospace;
    font-size: 13px;
    font-weight: 500;
  }

  .repo-org {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .private-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 10px;
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
    margin-left: auto;
  }

  .dropdown-footer {
    padding: 8px 12px;
    font-size: 11px;
    color: var(--text-secondary);
    text-align: center;
    border-top: 1px solid var(--border);
  }

  .dropdown-empty {
    padding: 16px 12px;
    font-size: 13px;
    color: var(--text-secondary);
    text-align: center;
  }

  .loading-repos {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 12px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .divider {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }

  .divider::before,
  .divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background-color: var(--border);
  }

  .divider span {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid var(--border);
  }

  .primary-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: var(--accent);
    color: white;
    padding: 10px 20px;
    font-weight: 600;
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .secondary-btn {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 10px 20px;
    font-weight: 500;
  }

  .secondary-btn:hover {
    background-color: var(--border);
  }

  .spin {
    display: inline-flex;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
