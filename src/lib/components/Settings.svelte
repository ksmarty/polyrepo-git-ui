<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';

  import type { AppConfig, Repository, RepoGroup } from '../types';

  import AuthSetup from './AuthSetup.svelte';
  import ImportModal from './ImportModal.svelte';
  import CloneModal from './CloneModal.svelte';

  import { Settings as SettingsIcon, FolderGit2, GitBranch, Trash2, Plus, FolderOpen, Palette, Download } from '@lucide/svelte';
  import { app } from '../stores.svelte';

  const dispatch = createEventDispatcher<{
    themeChange: string;
    dataChange: void;
    authChange: void;
  }>();

  let activeSection: 'general' | 'themes' | 'repos' | 'github' = $state('general');

  let config: AppConfig = $state({
    default_repo_location: '',
    theme: 'midnight',
    auto_fetch_on_open: true,
    fetch_interval_seconds: 300,
    sidebar_width: 300,
    pr_density: 'compact',
    minimize_on_close: true,
  });

  let repos: Repository[] = $state([]);
  let groups: RepoGroup[] = $state([]);
  let error: string | null = $state(null);
  let showImportModal: boolean = $state(false);
  let showCloneModal: boolean = $state(false);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let configPath: string = $state('');
  let previousTheme: string = $state('');

  const themes = [
    { value: 'catppuccin', label: 'Catppuccin', colors: { bg: '#1e1e2e', bgSecondary: '#252536', accent: '#f38ba8', text: '#cdd6f4' } },
    { value: 'forest', label: 'Corona Forest', colors: { bg: '#f2f7f5', bgSecondary: '#e6f0ec', accent: '#00473e', text: '#00473e' } },
    { value: 'ocean', label: 'Fizzy Whirlpool', colors: { bg: '#004643', bgSecondary: '#005c58', accent: '#f9bc60', text: '#e8e4e6' } },
    { value: 'dark', label: 'Flattered Sugar', colors: { bg: '#0f0e17', bgSecondary: '#1a1929', accent: '#ff8906', text: '#fffffe' } },
    { value: 'solarized', label: 'Midnight Evening', colors: { bg: '#232946', bgSecondary: '#2a3050', accent: '#eebbc3', text: '#fffffe' } },
    { value: 'midnight', label: 'Candy Grape', colors: { bg: '#16161a', bgSecondary: '#1e1e24', accent: '#7f5af0', text: '#fffffe' } },
    { value: 'light', label: 'White Piglet', colors: { bg: '#fffffe', bgSecondary: '#f2f0ed', accent: '#e53170', text: '#2b2c34' } },
  ];

  onMount(async () => {
    await loadConfig();
    await loadRepos();
    await loadGroups();
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

  async function loadGroups() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      groups = await invoke('get_groups');
    } catch (e) {
      console.error('Failed to load groups:', e);
    }
  }

  async function moveRepoToGroup(repoId: string, groupId: string | null) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('move_repo_to_group', { repoId, groupId: groupId || null });
      await loadRepos();
      dispatch('dataChange');
    } catch (e) {
      console.error('Failed to move repo:', e);
    }
  }

  async function removeRepo(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_repo', { id });
      if (app.selectedRepo?.id === id) {
        app.selectedRepo = null;
      }
      await loadRepos();
      dispatch('dataChange');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to remove repo';
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

  function selectTheme(themeValue: string) {
    config.theme = themeValue;
    scheduleSave();
  }

  function handleImportComplete() {
    showImportModal = false;
    loadRepos();
  }

  function handleCloneComplete() {
    showCloneModal = false;
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
      class:active={activeSection === 'themes'}
      onclick={() => activeSection = 'themes'}
    >
      <Palette size={16} />
      Themes
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
          <label class="checkbox-label">
            <input
              type="checkbox"
              bind:checked={config.minimize_on_close}
              onchange={scheduleSave}
            />
            Minimize on close
          </label>
          <p class="hint">When closing the window, minimize instead of quitting</p>
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

    {:else if activeSection === 'themes'}
      <div class="section">
        <h3>Themes</h3>
        <div class="theme-grid">
          {#each themes as theme}
            <button
              class="theme-card"
              class:active={config.theme === theme.value}
              onclick={() => selectTheme(theme.value)}
            >
              <div class="theme-preview" style="background-color: {theme.colors.bg};">
                <div class="preview-sidebar" style="background-color: {theme.colors.bgSecondary};">
                  <div class="preview-item" style="background-color: {theme.colors.accent};"></div>
                  <div class="preview-item" style="background-color: {theme.colors.text}; opacity: 0.3;"></div>
                  <div class="preview-item" style="background-color: {theme.colors.text}; opacity: 0.3;"></div>
                </div>
                <div class="preview-content">
                  <div class="preview-header" style="background-color: {theme.colors.bgSecondary};">
                    <div class="preview-dot" style="background-color: {theme.colors.accent};"></div>
                    <div class="preview-line" style="background-color: {theme.colors.text}; opacity: 0.2;"></div>
                  </div>
                  <div class="preview-body">
                    <div class="preview-line long" style="background-color: {theme.colors.text}; opacity: 0.15;"></div>
                    <div class="preview-line medium" style="background-color: {theme.colors.text}; opacity: 0.1;"></div>
                    <div class="preview-card" style="background-color: {theme.colors.bgSecondary}; border-color: {theme.colors.text};">
                      <div class="preview-line short" style="background-color: {theme.colors.accent};"></div>
                    </div>
                  </div>
                </div>
              </div>
              <span class="theme-label">{theme.label}</span>
              {#if config.theme === theme.value}
                <span class="theme-active-badge">Active</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>

    {:else if activeSection === 'repos'}
      <div class="section">
        <div class="section-header">
          <h3>Repositories</h3>
          <div class="section-actions">
            <button class="action-btn" onclick={() => showCloneModal = true}>
              <Download size={14} />
              Clone
            </button>
            <button class="action-btn" onclick={() => showImportModal = true}>
              <Plus size={14} />
              Import
            </button>
          </div>
        </div>

        <div class="item-list">
          {#each repos as repo (repo.id)}
            <div class="item-row">
              <div class="item-info">
                <span class="item-name">{repo.name}</span>
                <span class="item-detail">{repo.path}</span>
              </div>
              <div class="item-actions">
                <select
                  class="folder-select"
                  value={repo.group_id || ''}
                  onchange={(e) => moveRepoToGroup(repo.id, (e.target as HTMLSelectElement).value || null)}
                >
                  <option value="">Ungrouped</option>
                  {#each groups as group}
                    <option value={group.id}>{group.name}</option>
                  {/each}
                </select>
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

{#if showCloneModal}
  <CloneModal
    on:close={() => showCloneModal = false}
    on:complete={handleCloneComplete}
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

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }

  .section-header h3 {
    font-size: 16px;
    font-weight: 700;
    margin: 0;
    padding: 0;
    border: none;
    letter-spacing: -0.02em;
  }

  .section-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
    border-radius: 6px;
    border: 1px solid var(--border);
  }

  .action-btn:hover {
    background-color: var(--border);
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

  .secondary-btn {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 8px 16px;
    font-weight: 500;
  }

  .secondary-btn:hover {
    background-color: var(--border);
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

  .folder-select {
    padding: 4px 24px 4px 8px;
    font-size: 12px;
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 6px center;
  }

  .folder-select:hover {
    border-color: var(--text-secondary);
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

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 12px;
    background: transparent;
    border: 2px solid var(--border);
    border-radius: 12px;
    cursor: pointer;
    transition: border-color 0.2s, transform 0.15s;
  }

  .theme-card:hover {
    border-color: var(--text-secondary);
    transform: translateY(-2px);
  }

  .theme-card.active {
    border-color: var(--accent);
    background-color: rgba(127, 90, 240, 0.05);
  }

  .theme-preview {
    width: 100%;
    height: 100px;
    border-radius: 8px;
    display: flex;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .preview-sidebar {
    width: 35%;
    padding: 8px 6px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-item {
    height: 6px;
    border-radius: 3px;
    width: 80%;
  }

  .preview-item:first-child {
    width: 60%;
  }

  .preview-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 6px;
    gap: 6px;
  }

  .preview-header {
    height: 12px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 6px;
  }

  .preview-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .preview-line {
    height: 4px;
    border-radius: 2px;
  }

  .preview-line.long { width: 90%; }
  .preview-line.medium { width: 60%; }
  .preview-line.short { width: 40%; height: 6px; }

  .preview-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .preview-card {
    border-radius: 4px;
    padding: 6px;
    border: 1px solid;
  }

  .theme-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .theme-active-badge {
    font-size: 10px;
    font-weight: 600;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
</style>
