<script lang="ts">
  import ImportModal from './ImportModal.svelte';
  import CloneModal from './CloneModal.svelte';
  import Sortable from 'sortablejs';
  import { Search, Plus, FolderPlus, ChevronRight, ChevronDown, GitBranch, Filter, Download, Trash2 } from '@lucide/svelte';
  import { app } from '../stores.svelte';
  import type { Repository, RepoGroup } from '../types';

  let expandedGroups: Set<string> = $state(new Set());
  let searchQuery: string = $state('');
  let showImportModal: boolean = $state(false);
  let showCloneModal: boolean = $state(false);
  let showNewGroupInput: boolean = $state(false);
  let newGroupName: string = $state('');
  let isDraggingRepo: boolean = $state(false);
  let dragOverGroupId: string | null = $state(null);
  let filterStale: boolean = $state(false);
  let contextMenu: { x: number; y: number; repoId: string } | null = $state(null);

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
    let list = app.repos
      .filter(r => r.group_id === groupId)
      .sort((a, b) => a.order - b.order);
    if (filterStale) {
      list = list.filter(r => r.sync_status && (r.sync_status.behind > 0 || r.sync_status.is_dirty));
    }
    return list;
  }

  function isGroupEmpty(groupId: string): boolean {
    return getReposInGroup(groupId).length === 0 && getSubGroups(groupId).length === 0;
  }

  function getSubGroups(parentId: string | null): RepoGroup[] {
    return app.groups
      .filter(g => g.parent_id === parentId)
      .sort((a, b) => a.order - b.order);
  }

  function getRootGroups(): RepoGroup[] {
    return app.groups
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
    if (repo.sync_status.behind > 0) return 'behind';
    if (repo.sync_status.is_dirty) return 'dirty';
    return 'up-to-date';
  }

  async function createGroup() {
    if (!newGroupName.trim()) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('create_group', { name: newGroupName.trim() });
      newGroupName = '';
      showNewGroupInput = false;
      await app.loadAll();
    } catch (e) {
      console.error('Failed to create group:', e);
    }
  }

  function handleRepoClick(repo: Repository) {
    app.selectRepo(repo);
  }

  function handleContextMenu(e: MouseEvent, repo: Repository) {
    e.preventDefault();
    contextMenu = { x: e.clientX, y: e.clientY, repoId: repo.id };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  async function removeRepoFromSidebar() {
    if (!contextMenu) return;
    const id = contextMenu.repoId;
    closeContextMenu();
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('remove_repo', { id });
      await app.loadAll();
    } catch (e) {
      console.error('Failed to remove repo:', e);
    }
  }

  async function fetchRepoFromSidebar(repoId: string) {
    await app.fetchRepo(repoId);
  }

  function handleImportComplete() {
    showImportModal = false;
    app.loadAll();
  }

  function handleCloneComplete() {
    showCloneModal = false;
    app.loadAll();
  }

  function getRepoIdsInGroup(groupId: string | null): string[] {
    return app.repos
      .filter(r => r.group_id === groupId)
      .sort((a, b) => a.order - b.order)
      .map(r => r.id);
  }

  function getGroupSortIds(): string[] {
    return getRootGroups().map(g => g.id);
  }

  function containerToGroupId(el: HTMLElement): string | null {
    if (el.hasAttribute('data-ungrouped')) return null;
    const groupId = el.getAttribute('data-group-id');
    return groupId ?? null;
  }

  function sortableGroup(node: HTMLElement) {
    const opts: any = {
      group: { name: 'groups', pull: false, put: false },
      animation: 150,
      draggable: '.group-wrapper',
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      forceFallback: true,
      fallbackClass: 'sortable-drag',
      distance: 15,
      delay: 150,
      delayOnTouchOnly: true,
      onEnd: async (evt: any) => {
        if (evt.oldIndex == null || evt.newIndex == null) return;
        const ids = getGroupSortIds();
        const [moved] = ids.splice(evt.oldIndex, 1);
        ids.splice(evt.newIndex, 0, moved);
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('reorder_groups', { groupIds: ids });
          await app.loadAll();
        } catch (e) {
          console.error('Failed to reorder groups:', e);
        }
      },
    };
    const sortable = Sortable.create(node, opts);
    return {
      destroy() {
        sortable.destroy();
      },
    };
  }

  function sortableRepos(node: HTMLElement, _groupId: string | null) {
    const opts: any = {
      group: { name: 'repos', pull: true, put: true },
      animation: 150,
      draggable: '.repo-item',
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      forceFallback: true,
      fallbackClass: 'sortable-drag',
      distance: 12,
      delay: 150,
      delayOnTouchOnly: true,
      onStart() {
        isDraggingRepo = true;
      },
      onEnd: async (evt: any) => {
        const repoId = evt.item.getAttribute('data-repo-id');
        if (!repoId || evt.oldIndex == null || evt.newIndex == null) return;
        const fromGroupId = containerToGroupId(evt.from);
        const toGroupId = containerToGroupId(evt.to);

        // Cross-container move: revert SortableJS's DOM change so Svelte handles rendering.
        if (evt.from !== evt.to) {
          evt.from.appendChild(evt.item);
        }

        // Optimistically update state so Svelte re-renders immediately.
        if (fromGroupId !== toGroupId) {
          app.repos = app.repos.map(r =>
            r.id === repoId ? { ...r, group_id: toGroupId } : r
          );
        }

        try {
          const { invoke } = await import('@tauri-apps/api/core');
          if (fromGroupId === toGroupId) {
            const ids = getRepoIdsInGroup(toGroupId);
            const oldIndex = ids.indexOf(repoId);
            if (oldIndex === -1) return;
            ids.splice(oldIndex, 1);
            ids.splice(evt.newIndex, 0, repoId);
            if (ids.every((id, idx) => id === getRepoIdsInGroup(toGroupId)[idx])) return;
            await invoke('reorder_repos', { repoIds: ids });
          } else {
            await invoke('move_repo_to_group', { repoId, groupId: toGroupId });
            const targetIds = getRepoIdsInGroup(toGroupId);
            targetIds.splice(evt.newIndex, 0, repoId);
            await invoke('reorder_repos', { repoIds: targetIds });
          }
          await app.loadAll();
        } catch (e) {
          console.error('Failed to move repo:', e);
          await app.loadAll();
        }
      },
      onMove(evt: any) {
        dragOverGroupId = containerToGroupId(evt.to);
      },
    };
    const sortable = Sortable.create(node, opts);
    return {
      destroy() {
        sortable.destroy();
        isDraggingRepo = false;
        dragOverGroupId = null;
      },
    };
  }

  function collapsedFolderSortable(node: HTMLElement, groupId: string) {
    const opts: any = {
      group: { name: 'repos', pull: false, put: true },
      animation: 150,
      draggable: '.repo-item',
      ghostClass: 'sortable-ghost',
      chosenClass: 'sortable-chosen',
      dragClass: 'sortable-drag',
      forceFallback: true,
      fallbackClass: 'sortable-drag',
      onAdd: async (evt: any) => {
        const repoId = evt.item.getAttribute('data-repo-id');
        if (!repoId) return;
        // Revert DOM change; let Svelte handle rendering.
        evt.from.appendChild(evt.item);
        app.repos = app.repos.map(r =>
          r.id === repoId ? { ...r, group_id: groupId } : r
        );
        try {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('move_repo_to_group', { repoId, groupId });
          await app.loadAll();
        } catch (e) {
          console.error('Failed to move repo to folder:', e);
          await app.loadAll();
        }
      },
    };
    const sortable = Sortable.create(node, opts);
    return {
      destroy() {
        sortable.destroy();
      },
    };
  }

  function handleGroupDragOver(e: DragEvent, groupId: string) {
    if (!isDraggingRepo) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    dragOverGroupId = groupId;
  }

  function handleGroupDragLeave(e: DragEvent, groupId: string) {
    const related = e.relatedTarget as HTMLElement | null;
    const target = e.currentTarget as HTMLElement;
    if (!related || !target.contains(related)) {
      if (dragOverGroupId === groupId) dragOverGroupId = null;
    }
  }

  async function handleGroupDrop(e: DragEvent, groupId: string) {
    e.preventDefault();
    if (!e.dataTransfer) return;
    dragOverGroupId = null;
    const raw = e.dataTransfer.getData('text/plain');
    if (!raw) return;
    let data: { type: string; id: string };
    try {
      data = JSON.parse(raw);
    } catch {
      return;
    }
    if (data.type !== 'repo') return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('move_repo_to_group', { repoId: data.id, groupId });
      await app.loadAll();
    } catch (err) {
      console.error('Drop failed:', err);
    }
  }

  function onRepoDragStart(e: DragEvent, repoId: string) {
    if (!e.dataTransfer) return;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', JSON.stringify({ type: 'repo', id: repoId }));
  }

  $effect(() => {
    if (!contextMenu) return;
    function handleClick() { contextMenu = null; }
    function handleKeydown(e: KeyboardEvent) { if (e.key === 'Escape') contextMenu = null; }
    window.addEventListener('click', handleClick);
    window.addEventListener('keydown', handleKeydown);
    return () => {
      window.removeEventListener('click', handleClick);
      window.removeEventListener('keydown', handleKeydown);
    };
  });
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<aside class="sidebar" onselectstart={(e) => e.preventDefault()}>
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
      <button
        class="action-btn"
        class:active={filterStale}
        onclick={() => filterStale = !filterStale}
        title="Show only out-of-date repos"
      >
        <Filter size={14} />
      </button>
      <button class="action-btn" onclick={() => showCloneModal = true} title="Clone a repo">
        <Download size={14} />
        <span>Clone</span>
      </button>
      <button class="action-btn" onclick={() => showImportModal = true} title="Import repos">
        <Plus size={16} />
        <span>Import</span>
      </button>
      <button class="action-btn" onclick={() => showNewGroupInput = true} title="New folder">
        <FolderPlus size={16} />
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
      {@const empty = isGroupEmpty(group.id)}
      <div class="group-wrapper" data-group-id={group.id}>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="group-header"
          class:empty-folder={empty}
          class:drop-target={dragOverGroupId === group.id && !expandedGroups.has(group.id)}
          data-group-id={group.id}
          role="treeitem"
          tabindex="0"
          aria-expanded={expandedGroups.has(group.id)}
          aria-selected="false"
          onclick={() => { if (!empty) toggleGroup(group.id); }}
          onkeydown={(e) => { if (!empty && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); toggleGroup(group.id); } }}
          ondragover={(e) => handleGroupDragOver(e, group.id)}
          ondragleave={(e) => handleGroupDragLeave(e, group.id)}
          ondrop={(e) => handleGroupDrop(e, group.id)}
        >
          {#if !empty}
            <span class="toggle-icon">
              {#if expandedGroups.has(group.id)}
                <ChevronDown size={14} />
              {:else}
                <ChevronRight size={14} />
              {/if}
            </span>
          {/if}
          <span class="group-name">{group.name}</span>
          <span class="repo-count">{getReposInGroup(group.id).length}</span>
        </div>

        {#if !expandedGroups.has(group.id)}
          <div
            class="collapsed-drop-target"
            data-group-id={group.id}
            use:collapsedFolderSortable={group.id}
          ></div>
        {/if}

        {#if expandedGroups.has(group.id)}
          <div class="group-content">
            {#each getSubGroups(group.id) as subGroup (subGroup.id)}
              {@const subEmpty = isGroupEmpty(subGroup.id)}
              <div class="subgroup">
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="group-header sub"
                  class:empty-folder={subEmpty}
                  data-group-id={subGroup.id}
                  role="treeitem"
                  tabindex="0"
                  aria-expanded={expandedGroups.has(subGroup.id)}
                  aria-selected="false"
                  onclick={() => { if (!subEmpty) toggleGroup(subGroup.id); }}
                  onkeydown={(e) => { if (!subEmpty && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); toggleGroup(subGroup.id); } }}
                  ondragover={(e) => handleGroupDragOver(e, subGroup.id)}
                  ondragleave={(e) => handleGroupDragLeave(e, subGroup.id)}
                  ondrop={(e) => handleGroupDrop(e, subGroup.id)}
                >
                  {#if !subEmpty}
                    <span class="toggle-icon">
                      {#if expandedGroups.has(subGroup.id)}
                        <ChevronDown size={14} />
                      {:else}
                        <ChevronRight size={14} />
                      {/if}
                    </span>
                  {/if}
                  <span class="group-name">{subGroup.name}</span>
                </div>

                {#if !expandedGroups.has(subGroup.id)}
                  <div
                    class="collapsed-drop-target sub"
                    data-group-id={subGroup.id}
                    use:collapsedFolderSortable={subGroup.id}
                  ></div>
                {/if}

                {#if expandedGroups.has(subGroup.id)}
                  <div
                    class="group-content repo-list"
                    data-group-id={subGroup.id}
                    use:sortableRepos={subGroup.id}
                  >
                    {#each filteredRepos(getReposInGroup(subGroup.id)) as repo (repo.id)}
                      <div
                        class="repo-item"
                        class:selected={app.selectedRepo?.id === repo.id}
                        data-repo-id={repo.id}
                        data-group-id={subGroup.id}
                        role="treeitem"
                        tabindex="0"
                        aria-selected={app.selectedRepo?.id === repo.id}
                        draggable="true"
                        ondragstart={(e) => onRepoDragStart(e, repo.id)}
                        onclick={() => handleRepoClick(repo)}
                        oncontextmenu={(e) => handleContextMenu(e, repo)}
                        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
                      >
                        <span class="sync-dot {getSyncClass(repo)}"></span>
                        <span class="repo-name">{repo.name}</span>
                        {#if repo.sync_status && repo.sync_status.behind > 0}
                          <button
                            class="repo-action-btn"
                            onclick={(e) => { e.stopPropagation(); fetchRepoFromSidebar(repo.id); }}
                            title={`Fetch — ${repo.sync_status.behind} commit${repo.sync_status.behind !== 1 ? 's' : ''} behind`}
                          >
                            <Download size={12} />
                          </button>
                        {/if}
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
                  class:selected={app.selectedRepo?.id === repo.id}
                  data-repo-id={repo.id}
                  data-group-id={group.id}
                  role="treeitem"
                  tabindex="0"
                  aria-selected={app.selectedRepo?.id === repo.id}
                  draggable="true"
                  ondragstart={(e) => onRepoDragStart(e, repo.id)}
                  onclick={() => handleRepoClick(repo)}
                  oncontextmenu={(e) => handleContextMenu(e, repo)}
                  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
                >
                  <span class="sync-dot {getSyncClass(repo)}"></span>
                  <span class="repo-name">{repo.name}</span>
                  {#if repo.sync_status && repo.sync_status.behind > 0}
                    <button
                      class="repo-action-btn"
                      onclick={(e) => { e.stopPropagation(); fetchRepoFromSidebar(repo.id); }}
                      title={`Fetch — ${repo.sync_status.behind} commit${repo.sync_status.behind !== 1 ? 's' : ''} behind`}
                    >
                      <Download size={12} />
                    </button>
                  {/if}
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
      {#each filteredRepos(getReposInGroup(null)) as repo (repo.id)}
        <div
          class="repo-item"
          class:selected={app.selectedRepo?.id === repo.id}
          data-repo-id={repo.id}
          data-group-id={null}
          role="treeitem"
          tabindex="0"
          aria-selected={app.selectedRepo?.id === repo.id}
          draggable="true"
          ondragstart={(e) => onRepoDragStart(e, repo.id)}
          onclick={() => handleRepoClick(repo)}
          oncontextmenu={(e) => handleContextMenu(e, repo)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleRepoClick(repo); } }}
        >
          <span class="sync-dot {getSyncClass(repo)}"></span>
          <span class="repo-name">{repo.name}</span>
          {#if repo.sync_status && repo.sync_status.behind > 0}
            <button
              class="repo-action-btn"
              onclick={(e) => { e.stopPropagation(); fetchRepoFromSidebar(repo.id); }}
              title={`Fetch — ${repo.sync_status.behind} commit${repo.sync_status.behind !== 1 ? 's' : ''} behind`}
            >
              <Download size={12} />
            </button>
          {/if}
        </div>
      {/each}
    </div>

    {#if app.repos.length === 0}
      <div class="empty-state">
        <GitBranch size={32} />
        <p>No repos added yet</p>
        <p class="hint">Click Import to get started</p>
      </div>
    {/if}
  </div>
</aside>

{#if contextMenu}
  <div
    class="context-menu"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    role="menu"
    onclick={(e) => e.stopPropagation()}
  >
    <button class="context-menu-item danger" role="menuitem" onclick={removeRepoFromSidebar}>
      <Trash2 size={14} />
      Remove
    </button>
  </div>
{/if}

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
  .sidebar {
    width: 280px;
    min-width: 280px;
    background-color: var(--bg-secondary);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    user-select: none;
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

  .action-btn.active {
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
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    border-radius: 6px;
    font-weight: 500;
    font-size: 13px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background-color 0.15s;
    user-select: none;
  }

  .group-header:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .group-header.drop-target {
    background-color: rgba(127, 90, 240, 0.15);
    outline: 2px dashed var(--info);
    outline-offset: -2px;
  }

  .group-header.empty-folder {
    cursor: default;
    opacity: 0.5;
  }

  .group-header.sub {
    padding-left: 24px;
  }

  .toggle-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    color: var(--text-secondary);
    flex-shrink: 0;
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

  .collapsed-drop-target {
    min-height: 4px;
    margin: 0 4px;
    border-radius: 4px;
    transition: background-color 0.15s, min-height 0.15s;
  }

  .collapsed-drop-target.sub {
    margin-left: 24px;
  }

  :global(.collapsed-drop-target.sortable-ghost) {
    min-height: 28px;
    background-color: rgba(127, 90, 240, 0.1);
    border: 2px dashed var(--info);
  }

  .repo-list {
    min-height: 8px;
  }

  .repo-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    color: var(--text-secondary);
    text-align: left;
    border-radius: 6px;
    font-size: 13px;
    cursor: pointer;
    transition: background-color 0.1s;
    border: 2px solid transparent;
    min-height: 36px;
  }

  .repo-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .repo-item.selected {
    background-color: var(--accent);
    color: white;
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
  .sync-dot.dirty { background-color: var(--warning); }
  .sync-dot.behind { background-color: var(--danger); }
  .sync-dot.ahead { background-color: var(--info); }
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

  .repo-action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.15s, background-color 0.15s;
  }

  .repo-item:hover .repo-action-btn {
    opacity: 1;
  }

  .repo-action-btn:hover {
    background-color: var(--accent);
    color: white;
  }

  .context-menu {
    position: fixed;
    z-index: 100;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px;
    min-width: 140px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
    border-radius: 6px;
    text-align: left;
  }

  .context-menu-item:hover {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .context-menu-item.danger:hover {
    background-color: rgba(242, 95, 76, 0.1);
    color: var(--danger);
  }
</style>
