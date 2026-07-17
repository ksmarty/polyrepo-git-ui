<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { X, FolderSearch, Loader2 } from '@lucide/svelte';

  const dispatch = createEventDispatcher<{
    close: void;
    complete: void;
  }>();

  let step: 'choose' | 'scanning' | 'select' = $state('choose');
  let selectedPath: string = $state('');
  let discoveredRepos: string[] = $state([]);
  let selectedRepos: Set<string> = $state(new Set());
  let loading: boolean = $state(false);
  let error: string | null = $state(null);
  let groupId: string = $state('');
  let groups: { id: string; name: string }[] = $state([]);

  async function loadGroups() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      groups = await invoke('get_groups');
    } catch (e) {
      console.error('Failed to load groups:', e);
    }
  }

  async function browseFolder() {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const result = await open({ directory: true });
      if (result && typeof result === 'string') {
        selectedPath = result;
        await scanDirectory();
      }
    } catch (e) {
      console.error('Failed to open dialog:', e);
    }
  }

  async function scanDirectory() {
    if (!selectedPath) return;
    step = 'scanning';
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      discoveredRepos = await invoke('scan_directory_for_repos', { path: selectedPath });
      if (discoveredRepos.length === 0) {
        error = 'No git repositories found at this path';
        step = 'choose';
        return;
      }
      selectedRepos = new Set(discoveredRepos);
      await loadGroups();
      step = 'select';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to scan directory';
      step = 'choose';
    }
  }

  function toggleRepo(path: string) {
    const next = new Set(selectedRepos);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    selectedRepos = next;
  }

  function selectAll() {
    selectedRepos = new Set(discoveredRepos);
  }

  function selectNone() {
    selectedRepos = new Set();
  }

  async function importSelected() {
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      for (const path of selectedRepos) {
        await invoke('add_repo', { path, groupId: groupId || null });
      }
      dispatch('complete');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to import repos';
      loading = false;
    }
  }

  function getRepoName(path: string): string {
    return path.split('/').pop() || path;
  }

  loadGroups();
</script>

<div class="modal-overlay" role="presentation" onclick={() => dispatch('close')} onkeydown={(e) => e.key === 'Escape' && dispatch('close')}>
  <div class="modal" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h3>Import Repositories</h3>
      <button class="close-btn" onclick={() => dispatch('close')}>
        <X size={18} />
      </button>
    </div>

    <div class="modal-body">
      {#if error}
        <div class="error">{error}</div>
      {/if}

      {#if step === 'choose'}
        <p class="description">
          Select a parent folder containing git repositories,
          or a single git repo. The app will scan for repos automatically.
        </p>

        <div class="path-input">
          <input
            type="text"
            bind:value={selectedPath}
            placeholder="/path/to/repos"
            onkeydown={(e) => e.key === 'Enter' && scanDirectory()}
          />
          <button class="browse-btn" onclick={browseFolder}>
            <FolderSearch size={16} />
            Browse
          </button>
        </div>

        <button
          class="primary-btn"
          onclick={scanDirectory}
          disabled={!selectedPath}
        >
          Scan
        </button>

      {:else if step === 'scanning'}
        <div class="scanning">
          <span class="spin"><Loader2 size={32} /></span>
          <p>Scanning for git repositories...</p>
        </div>

      {:else if step === 'select'}
        <div class="select-header">
          <span>Found {discoveredRepos.length} repositories</span>
          <div class="select-actions">
            <button class="link-btn" onclick={selectAll}>Select all</button>
            <button class="link-btn" onclick={selectNone}>Select none</button>
          </div>
        </div>

        <div class="repo-select-group">
          <label for="group-select">Add to folder (optional)</label>
          <select id="group-select" bind:value={groupId}>
            <option value="">No folder</option>
            {#each groups as group}
              <option value={group.id}>{group.name}</option>
            {/each}
          </select>
        </div>

        <div class="repo-list">
          {#each discoveredRepos as repoPath}
            <label class="repo-checkbox">
              <input
                type="checkbox"
                checked={selectedRepos.has(repoPath)}
                onchange={() => toggleRepo(repoPath)}
              />
              <span class="repo-name">{getRepoName(repoPath)}</span>
              <span class="repo-path">{repoPath}</span>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    {#if step === 'select'}
      <div class="modal-footer">
        <button class="secondary-btn" onclick={() => dispatch('close')}>Cancel</button>
        <button
          class="primary-btn"
          onclick={importSelected}
          disabled={loading || selectedRepos.size === 0}
        >
          {loading ? 'Importing...' : `Import ${selectedRepos.size} repos`}
        </button>
      </div>
    {/if}
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
    max-height: 80vh;
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

  .description {
    color: var(--text-secondary);
    font-size: 14px;
    margin-bottom: 16px;
    line-height: 1.5;
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

  .path-input {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .path-input input {
    flex: 1;
  }

  .browse-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 8px 14px;
    font-weight: 500;
    white-space: nowrap;
  }

  .browse-btn:hover {
    background-color: var(--border);
  }

  .primary-btn {
    background-color: var(--accent);
    color: white;
    padding: 10px 20px;
    font-weight: 600;
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

  .link-btn {
    background: transparent;
    color: var(--accent);
    padding: 0;
    font-size: 13px;
    font-weight: 500;
  }

  .link-btn:hover {
    text-decoration: underline;
  }

  .scanning {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 40px;
    color: var(--text-secondary);
  }

  .spin {
    display: inline-flex;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .select-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    font-size: 14px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .select-actions {
    display: flex;
    gap: 12px;
  }

  .repo-select-group {
    margin-bottom: 16px;
  }

  .repo-select-group label {
    display: block;
    font-size: 13px;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .repo-select-group select {
    width: 200px;
  }

  .repo-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 300px;
    overflow-y: auto;
  }

  .repo-checkbox {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: background-color 0.1s;
  }

  .repo-checkbox:hover {
    background-color: var(--bg-tertiary);
  }

  .repo-checkbox input[type="checkbox"] {
    accent-color: var(--accent);
    width: 16px;
    height: 16px;
  }

  .repo-name {
    font-weight: 500;
    min-width: 120px;
  }

  .repo-path {
    color: var(--text-secondary);
    font-size: 11px;
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid var(--border);
  }
</style>
