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

  let dragOverTarget: string | null = $state(null);
  let dragOverType: 'group' | 'repo' | 'ungrouped' | null = $state(null);
  let dragOverSide: 'above' | 'below' | 'inside' = $state('inside');

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

  function handleRepoClick(repo: Repository) {
    dispatch('select', repo);
  }

  function handleImportComplete() {
    showImportModal = false;
    dispatch('dataChange');
  }

  function onDragStart(e: DragEvent, type: 'repo' | 'group', id: string) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('application/json', JSON.stringify({ type, id }));
    (e.target as HTMLElement).classList.add('dragging');
  }

  function onDragEnd(e: DragEvent) {
    (e.target as HTMLElement).classList.remove('dragging');
    dragOverTarget = null;
    dragOverType = null;
    dragOverSide = 'inside';
  }

  function getDropInfo(e: DragEvent): { target: string; type: 'group' | 'repo' | 'ungrouped'; side: 'above' | 'below' | 'inside'; groupId: string | null } | null {
    const el = e.target as HTMLElement;
    const rect = el.getBoundingClientRect();
    const y = e.clientY - rect.top;
    const ratio = y / rect.height;

    // Check for repo-item
    const repoItem = el.closest('[data-repo-id]');
    if (repoItem) {
      const repoId = repoItem.getAttribute('data-repo-id')!;
      const groupId = repoItem.getAttribute('data-group-id');
      const side: 'above' | 'below' = ratio < 0.5 ? 'above' : 'below';
      return { target: repoId, type: 'repo', side, groupId };
    }

    // Check for group-header (but not inside a group-content)
    const groupHeader = el.closest('[data-group-id]');
    if (groupHeader) {
      const groupId = groupHeader.getAttribute('data-group-id')!;
      return { target: groupId, type: 'group', side: 'inside', groupId };
    }

    // Check for ungrouped zone
    const ungrouped = el.closest('[data-ungrouped]');
    if (ungrouped) {
      return { target: '__ungrouped', type: 'ungrouped', side: 'inside', groupId: null };
    }

    return null;
  }

  function onTreeDragOver(e: DragEvent) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    e.dataTransfer.dropEffect = 'move';

    const info = getDropInfo(e);
    if (info) {
      dragOverTarget = info.target;
      dragOverType = info.type;
      dragOverSide = info.side;
    }
  }

  function onTreeDragLeave(e: DragEvent) {
    const related = e.relatedTarget as HTMLElement | null;
    const tree = (e.currentTarget as HTMLElement);
    if (!related || !tree.contains(related)) {
      dragOverTarget = null;
      dragOverType = null;
      dragOverSide = 'inside';
    }
  }

  async function onTreeDrop(e: DragEvent) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    const raw = e.dataTransfer.getData('application/json');
    if (!raw) return;

    let data: { type: string; id: string };
    try {
      data = JSON.parse(raw);
    } catch {
      return;
    }

    const info = getDropInfo(e);
    if (!info) {
      dragOverTarget = null;
      dragOverType = null;
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      if (data.type === 'repo') {
        if (info.type === 'group') {
          // Drop repo into folder
          await invoke('move_repo_to_group', { repoId: data.id, groupId: info.target });
          dispatch('dataChange');
        } else if (info.type === 'ungrouped') {
          // Drop repo out of folder
          await invoke('move_repo_to_group', { repoId: data.id, groupId: null });
          dispatch('dataChange');
        } else if (info.type === 'repo' && info.target !== data.id) {
          // Reorder repo: get list for the target group, move source to target position
          const targetGroupId = info.groupId;
          const repoIds = repos
            .filter(r => r.group_id === targetGroupId)
            .sort((a, b) => a.order - b.order)
            .map(r => r.id);
          const fromIdx = repoIds.indexOf(data.id);
          let toIdx = repoIds.indexOf(info.target);

          if (fromIdx === -1) {
            // Moving from a different group — first move, then reorder
            await invoke('move_repo_to_group', { repoId: data.id, groupId: targetGroupId });
            const newRepoIds = repos
              .filter(r => r.group_id === targetGroupId)
              .sort((a, b) => a.order - b.order)
              .map(r => r.id);
            toIdx = newRepoIds.indexOf(info.target);
            if (info.side === 'below') toIdx++;
            newRepoIds.splice(newRepoIds.indexOf(data.id), 1);
            if (toIdx > newRepoIds.indexOf(data.id)) toIdx--;
            newRepoIds.splice(Math.max(0, toIdx), 0, data.id);
            await invoke('reorder_repos', { repoIds: newRepoIds });
          } else {
            repoIds.splice(fromIdx, 1);
            toIdx = repoIds.indexOf(info.target);
            if (info.side === 'below') toIdx++;
            repoIds.splice(Math.max(0, toIdx), 0, data.id);
            await invoke('reorder_repos', { repoIds });
          }
          dispatch('dataChange');
        }
      } else if (data.type === 'group') {
        if (info.type === 'group' && info.target !== data.id) {
          // Reorder groups
          const allGroupIds = groups
            .filter(g => g.parent_id === null)
            .sort((a, b) => a.order - b.order)
            .map(g => g.id);
          const fromIdx = allGroupIds.indexOf(data.id);
          let toIdx = allGroupIds.indexOf(info.target);
          if (info.side === 'below') toIdx++;
          allGroupIds.splice(fromIdx, 1);
          toIdx = allGroupIds.indexOf(info.target);
          if (info.side === 'below') toIdx++;
          allGroupIds.splice(Math.max(0, toIdx), 0, data.id);
          await invoke('reorder_groups', { groupIds: allGroupIds });
          dispatch('dataChange');
        }
      }
    } catch (err) {
      console.error('Drop failed:', err);
    }

    dragOverTarget = null;
    dragOverType = null;
    dragOverSide = 'inside';
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

  <div
    class="repo-tree"
    role="tree"
    tabindex="-1"
    ondragover={onTreeDragOver}
    ondragleave={onTreeDragLeave}
    ondrop={onTreeDrop}
  >
    {#each getRootGroups() as group (group.id)}
      <div class="group" role="treeitem" aria-expanded={expandedGroups.has(group.id)} aria-selected="false">
        <div
          class="group-header"
          class:drag-over={dragOverTarget === group.id && dragOverType === 'group'}
          data-group-id={group.id}
          role="treeitem"
          tabindex="0"
          aria-selected="false"
          draggable="true"
          ondragstart={(e) => onDragStart(e, 'group', group.id)}
          ondragend={onDragEnd}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggleGroup(group.id); } }}
        >
          <button class="toggle-btn" onclick={() => toggleGroup(group.id)}>
            {#if expandedGroups.has(group.id)}
              <ChevronDown size={14} />
            {:else}
              <ChevronRight size={14} />
            {/if}
          </button>
          <span class="group-name">{group.name}</span>
          <span class="repo-count">{getReposInGroup(group.id).length}</span>
        </div>

        {#if expandedGroups.has(group.id)}
          <div class="group-content">
            {#each getSubGroups(group.id) as subGroup (subGroup.id)}
              <div class="subgroup">
                <div
                  class="group-header sub"
                  class:drag-over={dragOverTarget === subGroup.id && dragOverType === 'group'}
                  data-group-id={subGroup.id}
                  role="treeitem"
                  draggable="true"
                  tabindex="0"
                  aria-selected="false"
                  ondragstart={(e) => onDragStart(e, 'group', subGroup.id)}
                  ondragend={onDragEnd}
                  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggleGroup(subGroup.id); } }}
                >
                  <button class="toggle-btn" onclick={() => toggleGroup(subGroup.id)}>
                    {#if expandedGroups.has(subGroup.id)}
                      <ChevronDown size={14} />
                    {:else}
                      <ChevronRight size={14} />
                    {/if}
                  </button>
                  <span class="group-name">{subGroup.name}</span>
                </div>

                {#if expandedGroups.has(subGroup.id)}
                  <div class="group-content">
                    {#each filteredRepos(getReposInGroup(subGroup.id)) as repo (repo.id)}
                      <div
                        class="repo-item"
                        class:selected={selectedRepo?.id === repo.id}
                        class:drag-over-above={dragOverTarget === repo.id && dragOverType === 'repo' && dragOverSide === 'above'}
                        class:drag-over-below={dragOverTarget === repo.id && dragOverType === 'repo' && dragOverSide === 'below'}
                        data-repo-id={repo.id}
                        data-group-id={subGroup.id}
                        role="treeitem"
                        tabindex="0"
                        aria-selected={selectedRepo?.id === repo.id}
                        draggable="true"
                        ondragstart={(e) => onDragStart(e, 'repo', repo.id)}
                        ondragend={onDragEnd}
                        onclick={() => handleRepoClick(repo)}
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
                      >
                        <span class="sync-dot {getSyncClass(repo)}"></span>
                        <span class="repo-name">{repo.name}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}

            {#each filteredRepos(getReposInGroup(group.id)) as repo (repo.id)}
              <div
                class="repo-item"
                class:selected={selectedRepo?.id === repo.id}
                class:drag-over-above={dragOverTarget === repo.id && dragOverType === 'repo' && dragOverSide === 'above'}
                class:drag-over-below={dragOverTarget === repo.id && dragOverType === 'repo' && dragOverSide === 'below'}
                data-repo-id={repo.id}
                data-group-id={group.id}
                role="treeitem"
                tabindex="0"
                aria-selected={selectedRepo?.id === repo.id}
                draggable="true"
                ondragstart={(e) => onDragStart(e, 'repo', repo.id)}
                ondragend={onDragEnd}
                onclick={() => handleRepoClick(repo)}
                onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
              >
                <span class="sync-dot {getSyncClass(repo)}"></span>
                <span class="repo-name">{repo.name}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}

    <div
      class="ungrouped"
      class:drag-over={dragOverTarget === '__ungrouped' && dragOverType === 'ungrouped'}
      data-ungrouped="true"
    >
      <span class="ungrouped-label">Ungrouped</span>
      {#each filteredRepos(getReposInGroup(null)) as repo (repo.id)}
        <div
          class="repo-item"
          class:selected={selectedRepo?.id === repo.id}
          class:drag-over-above={dragOverTarget === repo.id && dragOverType === 'repo' && dragOverSide === 'above'}
          class:drag-over-below={dragOverTarget === repo.id && dragOverType === 'repo' && dragOverSide === 'below'}
          data-repo-id={repo.id}
          data-group-id={null}
          role="treeitem"
          tabindex="0"
          aria-selected={selectedRepo?.id === repo.id}
          draggable="true"
          ondragstart={(e) => onDragStart(e, 'repo', repo.id)}
          ondragend={onDragEnd}
          onclick={() => handleRepoClick(repo)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
        >
          <span class="sync-dot {getSyncClass(repo)}"></span>
          <span class="repo-name">{repo.name}</span>
        </div>
      {/each}
    </div>

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
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 4px 4px;
    border-radius: 6px;
    font-weight: 500;
    font-size: 13px;
    color: var(--text-secondary);
    cursor: grab;
    transition: background-color 0.15s;
  }

  .group-header:active {
    cursor: grabbing;
  }

  .group-header.drag-over {
    background-color: rgba(127, 90, 240, 0.15);
    outline: 2px dashed var(--info);
    outline-offset: -2px;
  }

  .toggle-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: transparent;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .toggle-btn:hover {
    color: var(--text-primary);
  }

  .group-header.sub {
    padding-left: 20px;
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
    padding: 5px 8px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    border-radius: 6px;
    font-size: 13px;
    cursor: grab;
    transition: background-color 0.1s;
    border: 2px solid transparent;
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

  .repo-item.dragging {
    opacity: 0.4;
  }

  .repo-item.drag-over-above {
    border-top-color: var(--info);
    margin-top: -2px;
    padding-top: 7px;
  }

  .repo-item.drag-over-below {
    border-bottom-color: var(--info);
    margin-bottom: -2px;
    padding-bottom: 7px;
  }

  .ungrouped {
    padding: 6px 4px;
    border-top: 1px dashed var(--border);
    margin-top: 8px;
    border-radius: 6px;
    min-height: 40px;
    transition: background-color 0.15s;
  }

  .ungrouped.drag-over {
    background-color: rgba(127, 90, 240, 0.1);
    outline: 2px dashed var(--info);
    outline-offset: -2px;
  }

  .ungrouped-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.6;
    display: block;
    padding: 2px 4px 4px;
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
