<script lang="ts">
  import type { PullRequest } from '../types';
  import { X, CheckCircle, XCircle, Clock, Users } from '@lucide/svelte';
  import { marked } from 'marked';

  interface Props {
    pr: PullRequest;
    repoName: string;
    onClose: () => void;
  }

  let { pr, repoName, onClose }: Props = $props();

  let renderedBody = $derived(pr.body ? marked.parse(pr.body) as string : '');

  async function openInGitHub() {
    if (!pr.html_url) return;
    try {
      const { open } = await import('@tauri-apps/plugin-shell');
      await open(pr.html_url);
    } catch (e) {
      console.error('Failed to open URL:', e);
    }
  }

  function getChecksIcon() {
    switch (pr.checks_status) {
      case 'success': return CheckCircle;
      case 'failure': return XCircle;
      default: return Clock;
    }
  }

  function getChecksClass(): string {
    switch (pr.checks_status) {
      case 'success': return 'success';
      case 'failure': return 'failure';
      case 'pending': return 'pending';
      case 'cancelled': return 'cancelled';
      default: return '';
    }
  }

  function getReviewClass(): string {
    switch (pr.review_decision) {
      case 'approved': return 'approved';
      case 'changes_requested': return 'changes-requested';
      default: return '';
    }
  }

  function getStateClass(): string {
    switch (pr.state) {
      case 'open': return 'state-open';
      case 'closed': return 'state-closed';
      case 'merged': return 'state-merged';
      default: return '';
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onClose();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-overlay" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-content" onclick={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <div class="header-title">
        <span class="pr-number">#{pr.number}</span>
        <h3>{pr.title}</h3>
      </div>
      <button class="modal-close" onclick={onClose}>
        <X size={18} />
      </button>
    </div>

    <div class="modal-body">
      <div class="info-section">
        <div class="pr-info-row">
          <span class="info-label">Repository</span>
          <span class="info-value">{repoName}</span>
        </div>

        <div class="pr-info-row">
          <span class="info-label">Author</span>
          <span class="info-value">{pr.author ?? 'Unknown'}</span>
        </div>

        <div class="pr-info-row">
          <span class="info-label">State</span>
          <span class="status-chip {getStateClass()}">{pr.state}</span>
        </div>

        <div class="pr-info-row">
          <span class="info-label">Branch</span>
          <span class="info-value branch-info">
            <span class="head-branch">{pr.head_ref}</span>
            <span class="arrow">&rarr;</span>
            <span class="base-branch">{pr.base_ref}</span>
          </span>
        </div>

        <div class="pr-info-row">
          <span class="info-label">Checks</span>
          {#if pr.checks_status}
            {@const ChecksIcon = getChecksIcon()}
            <span class="status-chip checks {getChecksClass()}">
              <ChecksIcon size={12} />
              {pr.checks_status}
            </span>
          {/if}
        </div>

        {#if pr.review_decision}
          <div class="pr-info-row">
            <span class="info-label">Review</span>
            <span class="status-chip {getReviewClass()}">
              {pr.review_decision.replace('_', ' ')}
            </span>
          </div>
        {/if}

        {#if pr.requested_reviewers.length > 0}
          <div class="pr-info-row">
            <span class="info-label">
              <Users size={14} />
              Reviewers
            </span>
            <span class="info-value">
              {#each pr.requested_reviewers as reviewer, i}{reviewer}{#if i < pr.requested_reviewers.length - 1}, {/if}{/each}
            </span>
          </div>
        {/if}

        <div class="pr-info-row">
          <span class="info-label">Sync</span>
          <span class="info-value">
            {#if !pr.mergeable}
              Has conflicts
            {:else if pr.behind_count > 0}
              {pr.behind_count} commits behind
            {:else}
              Up to date
            {/if}
          </span>
        </div>
      </div>

      {#if pr.body}
        <div class="pr-description">
          <span class="info-label">Description</span>
          <div class="description-content markdown-body">{@html renderedBody}</div>
        </div>
      {/if}
    </div>

    <div class="modal-actions">
      <button class="close-btn" onclick={onClose}>Close</button>
      {#if pr.html_url}
        <button class="github-link-btn" onclick={openInGitHub}>
          <svg viewBox="0 0 24 24" width="14" height="14"><path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" fill="currentColor" /></svg>
          Open in GitHub
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
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
    max-width: 600px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .header-title {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .pr-number {
    color: var(--text-secondary);
    font-size: 14px;
    font-weight: 600;
    flex-shrink: 0;
  }

  .modal-header h3 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    word-break: break-word;
  }

  .modal-close {
    background: transparent;
    color: var(--text-secondary);
    padding: 4px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .modal-close:hover {
    color: var(--text-primary);
  }

  .modal-body {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .info-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex-shrink: 0;
  }

  .pr-info-row {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
  }

  .info-label {
    color: var(--text-secondary);
    min-width: 90px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    font-weight: 500;
  }

  .info-value {
    color: var(--text-primary);
    word-break: break-all;
  }

  .branch-info {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: monospace;
    font-size: 12px;
  }

  .head-branch { color: var(--info); }
  .arrow { opacity: 0.4; }
  .base-branch { color: var(--text-secondary); }

  .status-chip {
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 600;
  }

  .status-chip.state-open {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .status-chip.state-closed {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .status-chip.state-merged {
    background-color: rgba(127, 90, 240, 0.15);
    color: var(--accent);
  }

  .status-chip.success {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .status-chip.failure {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .status-chip.pending {
    background-color: rgba(249, 188, 96, 0.15);
    color: var(--warning);
  }

  .status-chip.cancelled {
    background-color: rgba(167, 169, 190, 0.15);
    color: var(--text-secondary);
  }

  .status-chip.approved {
    background-color: rgba(44, 182, 125, 0.15);
    color: var(--success);
  }

  .status-chip.changes-requested {
    background-color: rgba(242, 95, 76, 0.15);
    color: var(--danger);
  }

  .status-chip.checks {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .pr-description {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    min-height: 0;
    margin-top: 12px;
  }

  .description-content {
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    font-size: 13px;
    color: var(--text-primary);
    word-break: break-word;
    line-height: 1.5;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .description-content.markdown-body :global(h1),
  .description-content.markdown-body :global(h2),
  .description-content.markdown-body :global(h3),
  .description-content.markdown-body :global(h4) {
    margin: 12px 0 6px;
    font-weight: 600;
  }

  .description-content.markdown-body :global(h1) { font-size: 16px; }
  .description-content.markdown-body :global(h2) { font-size: 15px; }
  .description-content.markdown-body :global(h3) { font-size: 14px; }

  .description-content.markdown-body :global(p) {
    margin: 0 0 8px;
  }

  .description-content.markdown-body :global(p:last-child) {
    margin-bottom: 0;
  }

  .description-content.markdown-body :global(code) {
    background-color: rgba(127, 90, 240, 0.12);
    color: var(--accent);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 12px;
    font-family: monospace;
  }

  .description-content.markdown-body :global(pre) {
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px;
    overflow-x: auto;
    margin: 8px 0;
  }

  .description-content.markdown-body :global(pre code) {
    background: none;
    padding: 0;
    color: var(--text-primary);
  }

  .description-content.markdown-body :global(ul),
  .description-content.markdown-body :global(ol) {
    margin: 6px 0;
    padding-left: 20px;
  }

  .description-content.markdown-body :global(li) {
    margin: 2px 0;
  }

  .description-content.markdown-body :global(blockquote) {
    border-left: 3px solid var(--accent);
    margin: 8px 0;
    padding: 4px 12px;
    color: var(--text-secondary);
  }

  .description-content.markdown-body :global(a) {
    color: var(--accent);
    text-decoration: none;
  }

  .description-content.markdown-body :global(a:hover) {
    text-decoration: underline;
  }

  .description-content.markdown-body :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 12px 0;
  }

  .description-content.markdown-body :global(table) {
    border-collapse: collapse;
    margin: 8px 0;
    width: 100%;
  }

  .description-content.markdown-body :global(th),
  .description-content.markdown-body :global(td) {
    border: 1px solid var(--border);
    padding: 4px 8px;
    font-size: 12px;
    text-align: left;
  }

  .description-content.markdown-body :global(th) {
    background-color: var(--bg-secondary);
    font-weight: 600;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
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

  .github-link-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    background-color: var(--accent);
    color: white;
    font-size: 12px;
    font-weight: 600;
    border-radius: 6px;
  }

  .github-link-btn:hover {
    opacity: 0.9;
  }
</style>
