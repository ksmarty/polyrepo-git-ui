<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { GitBranch, Unplug, KeyRound, ExternalLink } from '@lucide/svelte';

  const dispatch = createEventDispatcher<{ authChange: void }>();

  let authMethod: 'oauth' | 'pat' | null = $state(null);
  let patToken: string = $state('');
  let patUser: string = $state('');
  let loading: boolean = $state(false);
  let error: string | null = $state(null);
  let success: string | null = $state(null);
  let isAuthenticated: boolean = $state(false);
  let scopes: string[] = $state([]);

  async function loadAuth() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const auth = await invoke('get_github_auth') as { token: string | null; method: string | null; user: string | null };
      isAuthenticated = auth.token !== null;
      authMethod = auth.method as 'oauth' | 'pat' | null;
      patUser = auth.user ?? '';

      if (auth.token) {
        try {
          scopes = await invoke('get_github_token_scopes') as string[];
        } catch {
          scopes = [];
        }
      }
    } catch (e) {
      console.error('Failed to load auth:', e);
    }
  }

  async function startOAuth() {
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('start_github_oauth');
      success = 'OAuth flow completed successfully';
      await loadAuth();
      dispatch('authChange');
    } catch (e) {
      error = e instanceof Error ? e.message : 'OAuth failed';
    } finally {
      loading = false;
    }
  }

  async function savePAT() {
    if (!patToken) return;
    loading = true;
    error = null;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('set_github_pat', { token: patToken, user: null });
      success = 'PAT saved successfully';
      patToken = '';
      await loadAuth();
      dispatch('authChange');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to save PAT';
    } finally {
      loading = false;
    }
  }

  async function disconnect() {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('disconnect_github');
      isAuthenticated = false;
      authMethod = null;
      patUser = '';
      scopes = [];
      success = 'Disconnected from GitHub';
      dispatch('authChange');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to disconnect';
    }
  }

  loadAuth();
</script>

<div class="auth-setup">
  <h3>GitHub Integration</h3>

  {#if error}
    <div class="alert error">
      <span>{error}</span>
      <button onclick={() => error = null}>&times;</button>
    </div>
  {/if}

  {#if success}
    <div class="alert success">
      <span>{success}</span>
    </div>
  {/if}

  {#if isAuthenticated}
    <div class="connected">
      <div class="status">
        <span class="status-dot"></span>
        <span>Connected to GitHub</span>
      </div>
      <p class="auth-info">
        Authenticated via {authMethod === 'oauth' ? 'OAuth' : 'Personal Access Token'}
        {#if patUser}
          as {patUser}
        {/if}
      </p>
      {#if scopes.length > 0}
        <p class="scopes-line">Scopes: {scopes.join(', ')}</p>
      {/if}
      <button class="disconnect-button" onclick={disconnect}>
        <Unplug size={16} />
        Disconnect
      </button>
    </div>
  {:else}
    <div class="auth-split">
      <div class="auth-methods">
        <div class="method">
          <div class="method-header">
            <GitBranch size={20} />
            <h4>OAuth (Recommended)</h4>
          </div>
          <p>Use GitHub's OAuth flow for the best security. This opens your browser for authentication.</p>
          <button class="oauth-button" onclick={startOAuth} disabled={loading}>
            {loading ? 'Connecting...' : 'Connect with GitHub'}
          </button>
        </div>

        <div class="divider">
          <span>or</span>
        </div>

        <div class="method">
          <div class="method-header">
            <KeyRound size={20} />
            <h4>Personal Access Token</h4>
          </div>
          <p>Use a classic or fine-grained PAT for enterprise environments or when OAuth isn't available.</p>
          <div class="pat-form">
            <input type="password" bind:value={patToken} placeholder="ghp_xxxxxxxxxxxx" />
            <button class="save-button" onclick={savePAT} disabled={loading || !patToken}>Save Token</button>
          </div>
        </div>
      </div>

      <div class="permissions-info">
        <h4>Token Types</h4>
        <div class="token-type">
          <h5>Fine-grained (recommended)</h5>
          <p>Scoped to specific repos. No SSO authorization needed.</p>
          <table>
            <thead><tr><th>Permission</th><th>Access Level</th></tr></thead>
            <tbody>
              <tr><td>Contents</td><td>Read and Write</td></tr>
              <tr><td>Pull requests</td><td>Read and Write</td></tr>
              <tr><td>Metadata</td><td>Read-only</td></tr>
              <tr><td>Commit statuses</td><td>Read-only</td></tr>
            </tbody>
          </table>
          <p class="create-link"><a href="https://github.com/settings/tokens?type=beta" target="_blank">Create fine-grained token <ExternalLink size={12} /></a></p>
        </div>
        <div class="token-type">
          <h5>Classic</h5>
          <p>Broader access. If your org uses SAML SSO, you must authorize the token for SSO after creating it.</p>
          <p class="sso-steps">Settings > Tokens > <em>Configure SSO</em> > Authorize</p>
          <p class="create-link"><a href="https://github.com/settings/tokens" target="_blank">Create classic token <ExternalLink size={12} /></a></p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .auth-setup { max-width: 500px; }

  @media (min-width: 900px) {
    .auth-setup { max-width: 100%; }
  }

  h3 {
    font-size: 16px;
    font-weight: 700;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
    letter-spacing: -0.02em;
  }

  h4 {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 0;
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

  .alert.success {
    background-color: rgba(44, 182, 125, 0.1);
    border: 1px solid var(--success);
    color: var(--success);
  }

  .alert button {
    background: transparent;
    color: inherit;
    padding: 0;
    font-size: 16px;
    line-height: 1;
  }

  .connected {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--success);
  }

  .auth-info {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 8px;
  }

  .scopes-line {
    color: var(--text-secondary);
    font-size: 12px;
    font-family: monospace;
    margin-bottom: 14px;
  }

  .disconnect-button {
    display: flex;
    align-items: center;
    gap: 6px;
    background-color: var(--danger);
    color: white;
    font-weight: 500;
  }

  .auth-methods {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .auth-split {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  @media (min-width: 900px) {
    .auth-split {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 20px;
      align-items: stretch;
    }
  }

  .method {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px;
  }

  .method-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .method-header :global(svg) {
    color: var(--accent);
  }

  .method p {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 12px;
    line-height: 1.5;
  }

  .oauth-button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    background-color: var(--accent);
    color: white;
    width: 100%;
    font-weight: 600;
  }

  .divider {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .divider::before,
  .divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background-color: var(--border);
  }

  .pat-form {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .pat-form input {
    width: 100%;
  }

  .save-button {
    background-color: var(--accent);
    color: white;
    font-weight: 600;
  }

  .permissions-info {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 20px;
  }

  .permissions-info h4 {
    margin-bottom: 16px;
  }

  .permissions-info p {
    color: var(--text-secondary);
    font-size: 13px;
    margin-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .permissions-info a {
    color: var(--accent);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    text-decoration: none;
  }

  .permissions-info a:hover {
    text-decoration: underline;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th, td {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }

  th {
    color: var(--text-secondary);
    font-weight: 500;
  }

  .token-type {
    margin-bottom: 16px;
  }

  .token-type:last-child {
    margin-bottom: 0;
  }

  .token-type h5 {
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .token-type p {
    margin-bottom: 8px;
  }

  .token-type p:last-child {
    margin-bottom: 0;
  }

  .create-link {
    margin-top: 12px;
  }

  .sso-steps {
    font-size: 12px;
    font-family: monospace;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    padding: 6px 10px;
    border-radius: 6px;
    margin-top: 4px;
  }
</style>
