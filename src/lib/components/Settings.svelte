<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  import type { AppConfig, Repository } from '../types';

  import AuthSetup from './AuthSetup.svelte';
  import ImportModal from './ImportModal.svelte';

  import { Settings as SettingsIcon, FolderGit2, GitBranch, Trash2, Plus, FolderOpen } from '@lucide/svelte';
  import { app } from '../stores.svelte';

  const dispatch = createEventDispatcher<{
    themeChange: string;
    dataChange: void;
    authChange: void;
  }>();

  let activeSection: 'general' | 'repos' | 'github' = $state('general');

  let config: AppConfig = $state({
    default_branch: 'main',
    default_repo_location: '',
    theme: 'midnight',
    auto_fetch_on_open: true,
    fetch_interval_seconds: 300,
    sidebar_width: 300,
    pr_density: 'compact',
  });

  let repos: Repository[] = $state([]);
  let newRepoPath: string = $state('');
  let loading: boolean = $state(false);
  let error: string | null = $state(null);
  let showImportModal: boolean = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let configPath: string = $state('');
  let previousTheme: string = $state('');

  const themes = [
    { value: 'system', label: 'System' },
    { value: 'catppuccin', label: 'Catppuccin' },
    { value: 'forest', label: 'Corona Forest' },
    { value: 'ocean', label: 'Fizzy Whirlpool' },
    { value: 'dark', label: 'Flattered Sugar' },
    { value: 'solarized', label: 'Midnight Evening' },
    { value: 'midnight', label: 'Candy Grape' },
    { value: 'light', label: 'White Piglet' },
  ];

  onMount(async () => {
    await loadConfig();
    await loadRepos();
    await loadConfigPath();
  });

  async function loadConfig() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      config = await invoke('get_config');
      previousTheme = config.theme;
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  }

  async function loadConfigPath() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      configPath = await invoke('get_config_path');
    } catch (e) {
      console.error('Failed to load config path:', e);
    }
  }

  async function openConfigFolder() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('open_config_folder');
    } catch (e) {
      console.error('Failed to open config folder:', e);
    }
  }

  function scheduleSave() {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('update_config', { config });
        app.config = { ...config };
        const themeChanged = config.theme !== previousTheme;
        previousTheme = config.theme;
        if (themeChanged) {
          dispatch('themeChange', config.theme);
        }
        app.showNotification('success', 'Settings saved');
      } catch (e) {
        const msg = e instanceof Error ? e.message : 'Failed to save settings';
        error = msg;
        app.showNotification('error', msg);
      }
    }, 400);
  }

  async function loadRepos() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      repos = await invoke('get_repos');
    } catch (e) {
      console.error('Failed to load repos:', e);
    }
  }

  async function addRepo() {
    if (!newRepoPath) return;
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('add_repo', { path: newRepoPath });
      newRepoPath = '';
      await loadRepos();
      dispatch('dataChange');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to add repo';
    } finally {
      loading = false;
    }
  }

  async function removeRepo(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_repo', { id });
      await loadRepos();
      dispatch('dataChange');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to remove repo';
    }
  }

  async function browseFolder() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true });
      if (selected && typeof selected === 'string') {
        newRepoPath = selected;
      }
    } catch (e) {
      console.error('Failed to open folder dialog:', e);
    }
  }

  async function browseRepoLocation() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({ directory: true });
      if (selected && typeof selected === 'string') {
        config.default_repo_location = selected;
        scheduleSave();
      }
    } catch (e) {
      console.error('Failed to open folder dialog:', e);
    }
  }

  function handleImportComplete() {
    showImportModal = false;
    loadRepos();
  }
</script>

<div class="settings-layout">
  <nav class="settings-nav">
    <button
      class="nav-item"
      class:active={activeSection === 'general'}
      onclick={() => activeSection = 'general'}
    >
      <SettingsIcon size={16} />
      General
    </button>
    <button
      class="nav-item"
      class:active={activeSection === 'repos'}
      onclick={() => activeSection = 'repos'}
    >
      <FolderGit2 size={16} />
      Repositories
    </button>
    <button
      class="nav-item"
      class:active={activeSection === 'github'}
      onclick={() => activeSection = 'github'}
    >
      <GitBranch size={16} />
      GitHub
    </button>
  </nav>

  <div class="settings-content">
    {#if error}
      <div class="alert error">
        <span>{error}</span>
        <button onclick={() => error = null}>&times;</button>
      </div>
    {/if}

    {#if activeSection === 'general'}
      <div class="section">
        <h3>General</h3>

        <div class="form-group">
          <label for="default-branch">Default Branch</label>
          <input
            id="default-branch"
            type="text"
            bind:value={config.default_branch}
            oninput={scheduleSave}
            placeholder="main"
          />
          <p class="hint">Fallback branch when no per-repo override is set</p>
        </div>

        <div class="form-group">
          <label for="default-repo-location">Default Repo Location</label>
          <div class="location-row">
            <input
              id="default-repo-location"
              type="text"
              bind:value={config.default_repo_location}
              oninput={scheduleSave}
              placeholder="~/Projects"
            />
            <button class="secondary-btn" onclick={browseRepoLocation}>Browse</button>
          </div>
          <p class="hint">Where new repos are cloned to by default</p>
        </div>

        <div class="form-group">
          <label for="theme">Theme</label>
          <select id="theme" bind:value={config.theme} onchange={scheduleSave}>
            {#each themes as t}
              <option value={t.value}>{t.label}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={config.auto_fetch_on_open}
              onchange={scheduleSave}
            />
            Auto-fetch on open
          </label>
        </div>

        <div class="form-group">
          <label for="fetch-interval">Fetch Interval (seconds)</label>
          <input
            id="fetch-interval"
            type="number"
            bind:value={config.fetch_interval_seconds}
            oninput={scheduleSave}
            min="60"
            max="3600"
          />
        </div>

        <div class="form-group">
          <label for="pr-density">PR Density</label>
          <select id="pr-density" bind:value={config.pr_density} onchange={scheduleSave}>
            <option value="compact">Compact</option>
            <option value="relaxed">Relaxed</option>
          </select>
          <p class="hint">Controls spacing in the pull request list</p>
        </div>

        <div class="form-group">
          <label>Config File</label>
          <div class="location-row">
            <code class="config-path">{configPath || '...'}</code>
            <button class="secondary-btn" onclick={openConfigFolder} title="Open config folder">
              <FolderOpen size={14} />
              Open
            </button>
          </div>
          <p class="hint">Configuration is stored as TOML</p>
        </div>
      </div>

    {:else if activeSection === 'repos'}
      <div class="section">
        <h3>Repositories</h3>

        <div class="add-row">
          <input
            type="text"
            bind:value={newRepoPath}
            placeholder="/path/to/repo"
            onkeydown={(e) => e.key === 'Enter' && addRepo()}
          />
          <button class="secondary-btn" onclick={browseFolder}>Browse</button>
          <button class="accent-btn" onclick={addRepo} disabled={loading || !newRepoPath}>
            Add
          </button>
          <button class="import-btn" onclick={() => showImportModal = true}>
            <Plus size={14} />
            Import Folder
          </button>
        </div>

        <div class="item-list">
          {#each repos as repo (repo.id)}
            <div class="item-row">
              <div class="item-info">
                <span class="item-name">{repo.name}</span>
                <span class="item-detail">{repo.path}</span>
              </div>
              <div class="item-actions">
                <button class="danger-btn-sm" onclick={() => removeRepo(repo.id)}>
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          {/each}

          {#if repos.length === 0}
            <p class="empty-text">No repos added yet</p>
          {/if}
        </div>
      </div>

    {:else if activeSection === 'github'}
      <div class="section">
        <AuthSetup on:authChange={() => dispatch('authChange')} />
      </div>
    {/if}
  </div>
</div>

{#if showImportModal}
  <ImportModal
    on:close={() => showImportModal = false}
    on:complete={handleImportComplete}
  />
{/if}

<style>
  .settings-layout {
    display: flex;
    gap: 24px;
    height: 100%;
  }

  .settings-nav {
    width: 180px;
    min-width: 180px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    font-size: 13px;
    font-weight: 500;
    border-radius: 8px;
  }

  .nav-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .nav-item.active {
    background-color: var(--accent);
    color: white;
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
  }

  .section {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
  }

  .section h3 {
    font-size: 16px;
    font-weight: 700;
    margin-bottom: 20px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
    letter-spacing: -0.02em;
  }

  .form-group {
    margin-bottom: 18px;
  }

  .form-group label {
    display: block;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 6px;
    color: var(--text-secondary);
  }

  .checkbox-label {
    display: flex !important;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    color: var(--text-primary) !important;
  }

  .form-group input[type="text"],
  .form-group input[type="number"],
  .form-group select {
    width: 100%;
    max-width: 300px;
  }

  .location-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .location-row input {
    flex: 1;
    max-width: none;
  }

  .config-path {
    flex: 1;
    font-size: 12px;
    padding: 8px 12px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: monospace;
  }

  .hint {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 4px;
    opacity: 0.7;
  }

  .add-row {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .add-row input {
    flex: 1;
  }

  .accent-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background-color: var(--accent);
    color: white;
    padding: 8px 16px;
    font-weight: 600;
    white-space: nowrap;
  }

  .secondary-btn {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 8px 16px;
    font-weight: 500;
  }

  .secondary-btn:hover {
    background-color: var(--border);
  }

  .import-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 8px 12px;
    font-weight: 500;
    white-space: nowrap;
    border: 1px dashed var(--border);
  }
  .import-btn:hover {
    background-color: var(--border);
    border-color: var(--text-secondary);
  }

  .item-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .item-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .item-name {
    font-weight: 500;
    font-size: 14px;
  }

  .item-detail {
    font-size: 12px;
    color: var(--text-secondary);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .danger-btn-sm {
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: transparent;
    color: var(--text-secondary);
    padding: 6px;
    font-size: 13px;
    border-radius: 6px;
  }

  .danger-btn-sm:hover {
    background-color: rgba(242, 95, 76, 0.1);
    color: var(--danger);
  }

  .empty-text {
    text-align: center;
    color: var(--text-secondary);
    padding: 24px;
    font-size: 14px;
  }

  .alert {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 14px;
    border-radius: 8px;
    margin-bottom: 16px;
    font-size: 13px;
    font-weight: 500;
  }

  .alert.error {
    background-color: rgba(242, 95, 76, 0.1);
    border: 1px solid var(--danger);
    color: var(--danger);
  }

  .alert button {
    background: transparent;
    color: inherit;
    padding: 0;
    font-size: 16px;
    line-height: 1;
  }
</style>
