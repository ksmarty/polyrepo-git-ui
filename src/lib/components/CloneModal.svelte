<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { X, Loader2, Lock, Globe } from '@lucide/svelte';
  import { app } from '../stores.svelte';

  const dispatch = createEventDispatcher<{
    close: void;
    complete: void;
  }>();

  let url: string = $state('');
  let path: string = $state('');
  let loading: boolean = $state(false);
  let error: string | null = $state(null);
  let githubConnected: boolean = $state(false);

  onMount(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const auth = await invoke('get_github_auth') as { token: string | null };
      githubConnected = !!auth.token;
    } catch {
      githubConnected = false;
    }
  });

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

  async function handleClone() {
    if (!url.trim() || !path.trim()) return;
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clone_repo', { url: url.trim(), path: path.trim() });
      dispatch('complete');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to clone repository';
      loading = false;
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
<div class="modal-overlay" role="presentation" onclick={() => dispatch('close')} onkeydown={(e) => e.key === 'Escape' && dispatch('close')}>
  <!-- svelte-ignore a11y_no_static_element_interactions a11y_click_events_have_key_events -->
  <div class="modal" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
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

      <div class="form-group">
        <label for="clone-url">Repository URL</label>
        <input
          id="clone-url"
          type="text"
          bind:value={url}
          oninput={updatePathFromUrl}
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
        disabled={loading || !url.trim() || !path.trim()}
      >
        {#if loading}
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
    width: 500px;
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
