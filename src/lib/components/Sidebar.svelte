<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Repository, RepoGroup } from '../types';
  import ImportModal from './ImportModal.svelte';
  import { Search, Plus, FolderPlus, ChevronRight, ChevronDown, GitBranch } from '@lucide/svelte';

  interface Props {
    repos: Repository[];
    groups: RepoGroup[];
    selectedRepo: Repository | null;
  }

  let { repos, groups, selectedRepo }: Props = $props();

  const dispatch = createEventDispatcher<{
    select: Repository;
    dataChange: void;
  }>();

  let expandedGroups: Set<string> = $state(new Set());
  let searchQuery: string = $state('');
  let showImportModal: boolean = $state(false);
  let showNewGroupInput: boolean = $state(false);
  let newGroupName: string = $state('');

  let dropTarget: string | null = $state(null);
  let dropPosition: 'before' | 'inside' | 'after' = $state('after');

  function toggleGroup(groupId: string) {
    const next = new Set(expandedGroups);
    if (next.has(groupId)) {
      next.delete(groupId);
    } else {
      next.add(groupId);
    }
    expandedGroups = next;
  }

  function getReposInGroup(groupId: string | null): Repository[] {
    return repos
      .filter(r => r.group_id === groupId)
      .sort((a, b) => a.order - b.order);
  }

  function getSubGroups(parentId: string | null): RepoGroup[] {
    return groups
      .filter(g => g.parent_id === parentId)
      .sort((a, b) => a.order - b.order);
  }

  function getRootGroups(): RepoGroup[] {
    return groups
      .filter(g => g.parent_id === null)
      .sort((a, b) => a.order - b.order);
  }

  function filteredRepos(repos: Repository[]): Repository[] {
    if (!searchQuery) return repos;
    return repos.filter(r =>
      r.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      r.path.toLowerCase().includes(searchQuery.toLowerCase())
    );
  }

  function getSyncClass(repo: Repository): string {
    if (!repo.sync_status) return 'unknown';
    if (repo.sync_status.is_dirty) return 'dirty';
    if (repo.sync_status.behind > 0) return 'behind';
    if (repo.sync_status.ahead > 0) return 'ahead';
    return 'up-to-date';
  }

  async function createGroup() {
    if (!newGroupName.trim()) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('create_group', { name: newGroupName.trim() });
      newGroupName = '';
      showNewGroupInput = false;
      dispatch('dataChange');
    } catch (e) {
      console.error('Failed to create group:', e);
    }
  }

  function handleImportComplete() {
    showImportModal = false;
    dispatch('dataChange');
  }

  function onRepoDragStart(e: DragEvent, repoId: string) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', JSON.stringify({ type: 'repo', id: repoId }));
  }

  function onGroupDragStart(e: DragEvent, groupId: string) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', JSON.stringify({ type: 'group', id: groupId }));
  }

  function onDragEnd() {
    dropTarget = null;
    dropPosition = 'after';
  }

  function onGroupDragOver(e: DragEvent, groupId: string) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    e.dataTransfer.dropEffect = 'move';
    dropTarget = groupId;
    dropPosition = 'inside';
  }

  function onGroupDragLeave() {
    if (dropTarget) {
      dropTarget = null;
      dropPosition = 'after';
    }
  }

  async function onGroupDrop(e: DragEvent, targetGroupId: string) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    const raw = e.dataTransfer.getData('text/plain');
    if (!raw) return;
    try {
      const data = JSON.parse(raw);
      const { invoke } = await import('@tauri-apps/api/core');
      if (data.type === 'repo') {
        await invoke('move_repo_to_group', { repoId: data.id, groupId: targetGroupId });
        dispatch('dataChange');
      } else if (data.type === 'group' && data.id !== targetGroupId) {
        const allGroupIds = groups
          .filter(g => g.parent_id === null)
          .sort((a, b) => a.order - b.order)
          .map(g => g.id);
        const fromIdx = allGroupIds.indexOf(data.id);
        const toIdx = allGroupIds.indexOf(targetGroupId);
        if (fromIdx !== -1 && toIdx !== -1) {
          allGroupIds.splice(fromIdx, 1);
          allGroupIds.splice(toIdx, 0, data.id);
          await invoke('reorder_groups', { groupIds: allGroupIds });
          dispatch('dataChange');
        }
      }
    } catch (err) {
      console.error('Drop failed:', err);
    }
    onDragEnd();
  }

  function onUngroupedDragOver(e: DragEvent) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    e.dataTransfer.dropEffect = 'move';
    dropTarget = '__ungrouped';
    dropPosition = 'inside';
  }

  function onUngroupedDragLeave() {
    if (dropTarget === '__ungrouped') {
      dropTarget = null;
      dropPosition = 'after';
    }
  }

  async function onUngroupedDrop(e: DragEvent) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    const raw = e.dataTransfer.getData('text/plain');
    if (!raw) return;
    try {
      const data = JSON.parse(raw);
      if (data.type === 'repo') {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('move_repo_to_group', { repoId: data.id, groupId: null });
        dispatch('dataChange');
      }
    } catch (err) {
      console.error('Drop failed:', err);
    }
    onDragEnd();
  }

  function onRepoDragOver(e: DragEvent, repoId: string, _repoGroupId: string | null) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    e.dataTransfer.dropEffect = 'move';
    dropTarget = repoId;
    dropPosition = 'after';
  }

  async function onRepoDrop(e: DragEvent, targetRepoId: string, targetGroupId: string | null) {
    e.preventDefault();
    e.stopPropagation();
    if (!e.dataTransfer) return;
    const raw = e.dataTransfer.getData('text/plain');
    if (!raw) return;
    try {
      const data = JSON.parse(raw);
      if (data.type === 'repo' && data.id !== targetRepoId) {
        const { invoke } = await import('@tauri-apps/api/core');
        const repoIds = repos
          .filter(r => r.group_id === targetGroupId)
          .sort((a, b) => a.order - b.order)
          .map(r => r.id);
        const fromIdx = repoIds.indexOf(data.id);
        let toIdx = repoIds.indexOf(targetRepoId);
        if (fromIdx !== -1 && toIdx !== -1) {
          repoIds.splice(fromIdx, 1);
          if (fromIdx < toIdx) toIdx--;
          repoIds.splice(toIdx, 0, data.id);
          await invoke('reorder_repos', { repoIds });
          dispatch('dataChange');
        } else if (fromIdx === -1) {
          await invoke('move_repo_to_group', { repoId: data.id, groupId: targetGroupId });
          const newRepoIds = repos
            .filter(r => r.group_id === targetGroupId)
            .sort((a, b) => a.order - b.order)
            .map(r => r.id);
          toIdx = newRepoIds.indexOf(targetRepoId);
          if (toIdx !== -1) {
            newRepoIds.splice(toIdx, 0, data.id);
            await invoke('reorder_repos', { repoIds: newRepoIds });
          }
          dispatch('dataChange');
        }
      }
    } catch (err) {
      console.error('Drop failed:', err);
    }
    onDragEnd();
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="search-wrapper">
      <Search size={14} />
      <input
        type="text"
        placeholder="Search repos..."
        bind:value={searchQuery}
        class="search-input"
      />
    </div>
    <div class="header-actions">
      <button class="action-btn" onclick={() => showImportModal = true} title="Import repos">
        <Plus size={16} />
        <span>Import</span>
      </button>
      <button class="action-btn" onclick={() => showNewGroupInput = true} title="New folder">
        <FolderPlus size={16} />
        <span>Folder</span>
      </button>
    </div>
  </div>

  {#if showNewGroupInput}
    <div class="new-group-input">
      <input
        type="text"
        placeholder="Folder name"
        bind:value={newGroupName}
        onkeydown={(e) => {
          if (e.key === 'Enter') createGroup();
          if (e.key === 'Escape') { showNewGroupInput = false; newGroupName = ''; }
        }}
      />
      <button class="confirm-btn" onclick={createGroup}>Create</button>
      <button class="cancel-btn" onclick={() => { showNewGroupInput = false; newGroupName = ''; }}>Cancel</button>
    </div>
  {/if}

  <div class="repo-tree">
    {#each getRootGroups() as group (group.id)}
      <div
        class="group"
        role="listitem"
        class:drop-target={dropTarget === group.id && dropPosition === 'inside'}
        ondragover={(e) => onGroupDragOver(e, group.id)}
        ondragleave={onGroupDragLeave}
        ondrop={(e) => onGroupDrop(e, group.id)}
      >
        <button
          class="group-header"
          draggable="true"
          onclick={() => toggleGroup(group.id)}
          ondragstart={(e) => onGroupDragStart(e, group.id)}
          ondragend={onDragEnd}
        >
          {#if expandedGroups.has(group.id)}
            <ChevronDown size={14} />
          {:else}
            <ChevronRight size={14} />
          {/if}
          <span class="group-name">{group.name}</span>
          <span class="repo-count">{getReposInGroup(group.id).length}</span>
        </button>

        {#if expandedGroups.has(group.id)}
          <div class="group-content">
            {#each getSubGroups(group.id) as subGroup (subGroup.id)}
              <div class="subgroup">
                <button
                  class="group-header sub"
                  draggable="true"
                  onclick={() => toggleGroup(subGroup.id)}
                  ondragstart={(e) => onGroupDragStart(e, subGroup.id)}
                  ondragend={onDragEnd}
                >
                  {#if expandedGroups.has(subGroup.id)}
                    <ChevronDown size={14} />
                  {:else}
                    <ChevronRight size={14} />
                  {/if}
                  <span class="group-name">{subGroup.name}</span>
                </button>

                {#if expandedGroups.has(subGroup.id)}
                  <div class="group-content">
                    {#each filteredRepos(getReposInGroup(subGroup.id)) as repo (repo.id)}
                      <button
                        class="repo-item"
                        class:selected={selectedRepo?.id === repo.id}
                        class:drop-above={dropTarget === repo.id && dropPosition === 'before'}
                        class:drop-below={dropTarget === repo.id && dropPosition === 'after'}
                        draggable="true"
                        onclick={() => dispatch('select', repo)}
                        ondragstart={(e) => onRepoDragStart(e, repo.id)}
                        ondragend={onDragEnd}
                        ondragover={(e) => onRepoDragOver(e, repo.id, subGroup.id)}
                        ondrop={(e) => onRepoDrop(e, repo.id, subGroup.id)}
                      >
                        <span class="sync-dot {getSyncClass(repo)}"></span>
                        <span class="repo-name">{repo.name}</span>
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}

            {#each filteredRepos(getReposInGroup(group.id)) as repo (repo.id)}
              <button
                class="repo-item"
                class:selected={selectedRepo?.id === repo.id}
                class:drop-above={dropTarget === repo.id && dropPosition === 'before'}
                class:drop-below={dropTarget === repo.id && dropPosition === 'after'}
                draggable="true"
                onclick={() => dispatch('select', repo)}
                ondragstart={(e) => onRepoDragStart(e, repo.id)}
                ondragend={onDragEnd}
                ondragover={(e) => onRepoDragOver(e, repo.id, group.id)}
                ondrop={(e) => onRepoDrop(e, repo.id, group.id)}
              >
                <span class="sync-dot {getSyncClass(repo)}"></span>
                <span class="repo-name">{repo.name}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}

    {#if filteredRepos(getReposInGroup(null)).length > 0}
      <div
        class="ungrouped"
        role="listitem"
        class:drop-target={dropTarget === '__ungrouped'}
        ondragover={onUngroupedDragOver}
        ondragleave={onUngroupedDragLeave}
        ondrop={onUngroupedDrop}
      >
        {#each filteredRepos(getReposInGroup(null)) as repo (repo.id)}
          <button
            class="repo-item"
            class:selected={selectedRepo?.id === repo.id}
            class:drop-above={dropTarget === repo.id && dropPosition === 'before'}
            class:drop-below={dropTarget === repo.id && dropPosition === 'after'}
            draggable="true"
            onclick={() => dispatch('select', repo)}
            ondragstart={(e) => onRepoDragStart(e, repo.id)}
            ondragend={onDragEnd}
            ondragover={(e) => onRepoDragOver(e, repo.id, null)}
            ondrop={(e) => onRepoDrop(e, repo.id, null)}
          >
            <span class="sync-dot {getSyncClass(repo)}"></span>
            <span class="repo-name">{repo.name}</span>
          </button>
        {/each}
      </div>
    {/if}

    {#if repos.length === 0}
      <div class="empty-state">
        <GitBranch size={32} />
        <p>No repos added yet</p>
        <p class="hint">Click Import to get started</p>
      </div>
    {/if}
  </div>
</aside>

{#if showImportModal}
  <ImportModal
    on:close={() => showImportModal = false}
    on:complete={handleImportComplete}
  />
{/if}

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .search-wrapper {
    display: flex;
    align-items: center;
    gap: 8px;
    background-color: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0 10px;
    color: var(--text-secondary);
  }

  .search-wrapper:focus-within {
    border-color: var(--accent);
  }

  .search-input {
    flex: 1;
    border: none;
    background: transparent;
    padding: 8px 0;
    font-size: 13px;
  }

  .search-input:focus {
    border: none;
    outline: none;
  }

  .header-actions {
    display: flex;
    gap: 6px;
  }

  .action-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 6px 10px;
    background-color: var(--bg-tertiary);
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    border-radius: 6px;
  }

  .action-btn:hover {
    background-color: var(--accent);
    color: white;
  }

  .new-group-input {
    display: flex;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    background-color: var(--bg-tertiary);
    min-width: 0;
  }

  .new-group-input input {
    flex: 1;
    min-width: 0;
    padding: 4px 8px;
    font-size: 13px;
  }

  .confirm-btn {
    padding: 4px 10px;
    background-color: var(--accent);
    color: white;
    font-size: 12px;
    border-radius: 6px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .cancel-btn {
    padding: 4px 10px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 12px;
    border-radius: 6px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .cancel-btn:hover {
    background-color: var(--bg-secondary);
  }

  .repo-tree {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .group {
    margin-bottom: 2px;
    border-radius: 6px;
    transition: background-color 0.15s;
  }

  .group.drop-target {
    background-color: rgba(127, 90, 240, 0.1);
    outline: 2px dashed var(--info);
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    border-radius: 6px;
    font-weight: 500;
    font-size: 13px;
    cursor: grab;
  }

  .group-header:active {
    cursor: grabbing;
  }

  .group-header:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .group-header.sub {
    padding-left: 24px;
  }

  .group-name {
    flex: 1;
  }

  .repo-count {
    font-size: 11px;
    color: var(--text-secondary);
    background-color: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 10px;
    font-weight: 500;
  }

  .group-content {
    padding-left: 12px;
  }

  .repo-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    border-radius: 6px;
    font-size: 13px;
    cursor: grab;
    transition: background-color 0.1s;
  }

  .repo-item:active {
    cursor: grabbing;
  }

  .repo-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .repo-item.selected {
    background-color: var(--accent);
    color: white;
  }

  .repo-item.drop-above {
    box-shadow: inset 0 2px 0 0 var(--info);
  }

  .repo-item.drop-below {
    box-shadow: inset 0 -2px 0 0 var(--info);
  }

  .ungrouped {
    padding: 4px 0;
    border-top: 1px dashed var(--border);
    margin-top: 4px;
    padding-top: 8px;
    border-radius: 6px;
    transition: background-color 0.15s;
  }

  .ungrouped.drop-target {
    background-color: rgba(127, 90, 240, 0.1);
    outline: 2px dashed var(--info);
  }

  .sync-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .sync-dot.up-to-date { background-color: var(--success); }
  .sync-dot.behind { background-color: var(--warning); }
  .sync-dot.ahead { background-color: var(--info); }
  .sync-dot.dirty { background-color: var(--danger); }
  .sync-dot.unknown { background-color: var(--text-secondary); }

  .repo-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 40px 20px;
    color: var(--text-secondary);
    text-align: center;
  }

  .hint {
    font-size: 12px;
    opacity: 0.6;
  }
</style>
