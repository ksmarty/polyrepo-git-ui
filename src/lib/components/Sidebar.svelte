<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Repository, RepoGroup } from '../types';
  import ImportModal from './ImportModal.svelte';
  import Sortable from 'sortablejs';
  import { Search, Plus, FolderPlus, ChevronRight, ChevronDown, GitBranch, GripVertical } from '@lucide/svelte';

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

  function getRepoIdsInGroup(groupId: string | null): string[] {
    return repos
      .filter(r => r.group_id === groupId)
      .sort((a, b) => a.order - b.order)
      .map(r => r.id);
  }

  function getGroupSortIds(): string[] {
    return getRootGroups().map(g => g.id);
  }

  async function moveRepoInSortable(
    repoId: string,
    fromGroupId: string | null,
    toGroupId: string | null,
    newIndex: number
  ) {
    if (fromGroupId === toGroupId) {
      // Reorder within the same group.
      const ids = getRepoIdsInGroup(toGroupId);
      const oldIndex = ids.indexOf(repoId);
      if (oldIndex === -1) return;
      ids.splice(oldIndex, 1);
      ids.splice(newIndex, 0, repoId);
      if (
        ids.every(
          (id, idx) =>
            id ===
            getRepoIdsInGroup(toGroupId)[idx]
        )
      ) {
        return;
      }
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('reorder_repos', { repoIds: ids });
      dispatch('dataChange');
    } else {
      // Move to a different group then reorder.
      const targetIds = getRepoIdsInGroup(toGroupId);
      targetIds.splice(newIndex, 0, repoId);
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('move_repo_to_group', { repoId, groupId: toGroupId });
      await invoke('reorder_repos', { repoIds: targetIds });
      dispatch('dataChange');
    }
  }

  function sortableGroup(node: HTMLElement) {
    const sortable = Sortable.create(node, {
      group: { name: 'groups', pull: false, put: false },
      animation: 150,
      handle: '.group-handle',
      draggable: '.group-wrapper',
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      forceFallback: true,
      fallbackClass: 'sortable-drag',
      onEnd: async (evt) => {
        if (evt.oldIndex == null || evt.newIndex == null) return;
        const ids = getGroupSortIds();
        const [moved] = ids.splice(evt.oldIndex, 1);
        ids.splice(evt.newIndex, 0, moved);
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('reorder_groups', { groupIds: ids });
          dispatch('dataChange');
        } catch (e) {
          console.error('Failed to reorder groups:', e);
        }
      },
    });
    return {
      destroy() {
        sortable.destroy();
      },
    };
  }

  function containerToGroupId(el: HTMLElement): string | null {
    if (el.hasAttribute('data-ungrouped')) return null;
    const groupId = el.getAttribute('data-group-id');
    return groupId ?? null;
  }

  function sortableRepos(node: HTMLElement, _groupId: string | null) {
    const sortable = Sortable.create(node, {
      group: { name: 'repos', pull: true, put: true },
      animation: 150,
      draggable: '.repo-item',
      handle: '.repo-handle',
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      forceFallback: true,
      fallbackClass: 'sortable-drag',
      delay: 0,
      delayOnTouchOnly: true,
      onEnd: async (evt) => {
        // Only handle the event on the sortable instance where the item was dropped.
        if (evt.to !== node) return;

        const repoId = evt.item.getAttribute('data-repo-id');
        if (!repoId || evt.oldIndex == null || evt.newIndex == null) return;

        const fromGroupId = containerToGroupId(evt.from);
        const toGroupId = containerToGroupId(evt.to);
        // newIndex after Sortable already inserted the item into the target container.
        const newIndex = evt.newIndex;

        try {
          await moveRepoInSortable(repoId, fromGroupId, toGroupId, newIndex);
        } catch (e) {
          console.error('Failed to move repo:', e);
        }
      },
    });
    return {
      destroy() {
        sortable.destroy();
      },
    };
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
    use:sortableGroup
  >
    {#each getRootGroups() as group (group.id)}
      <div class="group-wrapper" data-group-id={group.id}>
        <div
          class="group-header"
          data-group-id={group.id}
          role="treeitem"
          tabindex="0"
          aria-expanded={expandedGroups.has(group.id)}
          aria-selected="false"
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggleGroup(group.id); } }}
        >
          <button class="toggle-btn" onclick={() => toggleGroup(group.id)}>
            {#if expandedGroups.has(group.id)}
              <ChevronDown size={14} />
            {:else}
              <ChevronRight size={14} />
            {/if}
          </button>
          <span class="group-handle" aria-label="Drag to reorder folder"><GripVertical size={10} /></span>
          <span class="group-name">{group.name}</span>
          <span class="repo-count">{getReposInGroup(group.id).length}</span>
        </div>

        {#if expandedGroups.has(group.id)}
          <div class="group-content">
            {#each getSubGroups(group.id) as subGroup (subGroup.id)}
              <div class="subgroup">
                <div
                  class="group-header sub"
                  data-group-id={subGroup.id}
                  role="treeitem"
                  tabindex="0"
                  aria-expanded={expandedGroups.has(subGroup.id)}
                  aria-selected="false"
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
                  <div
                    class="group-content repo-list"
                    data-group-id={subGroup.id}
                    use:sortableRepos={subGroup.id}
                  >
                    {#each filteredRepos(getReposInGroup(subGroup.id)) as repo (repo.id)}
                      <div
                        class="repo-item"
                        class:selected={selectedRepo?.id === repo.id}
                        data-repo-id={repo.id}
                        data-group-id={subGroup.id}
                        role="treeitem"
                        tabindex="0"
                        aria-selected={selectedRepo?.id === repo.id}
                        onclick={() => handleRepoClick(repo)}
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
                      >
                        <span class="repo-handle"><GripVertical size={10} /></span>
                        <span class="sync-dot {getSyncClass(repo)}"></span>
                        <span class="repo-name">{repo.name}</span>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}

            <div
              class="group-content repo-list"
              data-group-id={group.id}
              use:sortableRepos={group.id}
            >
              {#each filteredRepos(getReposInGroup(group.id)) as repo (repo.id)}
                <div
                  class="repo-item"
                  class:selected={selectedRepo?.id === repo.id}
                  data-repo-id={repo.id}
                  data-group-id={group.id}
                  role="treeitem"
                  tabindex="0"
                  aria-selected={selectedRepo?.id === repo.id}
                  onclick={() => handleRepoClick(repo)}
                  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
                >
                  <span class="repo-handle"><GripVertical size={10} /></span>
                  <span class="sync-dot {getSyncClass(repo)}"></span>
                  <span class="repo-name">{repo.name}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}

    <div
      class="ungrouped repo-list"
      data-ungrouped="true"
      use:sortableRepos={null}
    >
      <span class="ungrouped-label">Ungrouped</span>
      {#each filteredRepos(getReposInGroup(null)) as repo (repo.id)}
        <div
          class="repo-item"
          class:selected={selectedRepo?.id === repo.id}
          data-repo-id={repo.id}
          data-group-id={null}
          role="treeitem"
          tabindex="0"
          aria-selected={selectedRepo?.id === repo.id}
          onclick={() => handleRepoClick(repo)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
        >
          <span class="repo-handle"><GripVertical size={10} /></span>
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

  :global(.sortable-ghost) {
    opacity: 0.5;
    background-color: rgba(127, 90, 240, 0.12);
    border: 2px dashed var(--info);
    border-radius: 6px;
  }

  :global(.sortable-chosen) {
    background-color: rgba(127, 90, 240, 0.08);
  }

  :global(.sortable-drag) {
    opacity: 0.95;
    background-color: var(--bg-tertiary);
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.2);
    border-radius: 6px;
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

  .group-wrapper {
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
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .group-header:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .group-header.sub {
    padding-left: 20px;
  }

  .group-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    opacity: 0.5;
    cursor: grab;
    padding: 0 2px;
    user-select: none;
  }

  .group-handle:hover {
    opacity: 0.9;
    color: var(--text-primary);
  }

  .group-handle:active {
    cursor: grabbing;
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

  .repo-list {
    min-height: 8px;
  }

  .repo-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 8px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
    transition: background-color 0.1s;
    border: 2px solid transparent;
  }

  .repo-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .repo-item.selected {
    background-color: var(--accent);
    color: white;
  }

  .repo-handle {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    opacity: 0.4;
    cursor: grab;
    padding: 0 2px;
    user-select: none;
  }

  .repo-item:hover .repo-handle {
    opacity: 0.8;
  }

  .repo-handle:active {
    cursor: grabbing;
  }

  .ungrouped {
    padding: 6px 4px;
    border-top: 1px dashed var(--border);
    margin-top: 8px;
    border-radius: 6px;
    min-height: 40px;
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
