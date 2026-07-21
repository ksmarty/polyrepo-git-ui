import type { Repository, RepoGroup, AppConfig, PullRequest } from './types';
import type { GitLogEntry, MergeResult, GitStatus, PullResult } from './tauri';
import * as api from './tauri';

class AppState {
  repos: Repository[] = $state([]);
  groups: RepoGroup[] = $state([]);
  selectedRepo: Repository | null = $state(null);
  config: AppConfig = $state({ default_repo_location: '', theme: 'midnight', auto_fetch_on_open: true, fetch_interval_seconds: 300, sidebar_width: 300, pr_density: 'compact', minimize_on_close: true });
  gitLog: GitLogEntry[] = $state([]);
  gitStatus: GitStatus | null = $state(null);
  loadingGitLog: boolean = $state(false);
  loadingGitStatus: boolean = $state(false);
  refreshingRepo: string | null = $state(null);
  refreshingAll: boolean = $state(false);
  mergingRepo: string | null = $state(null);
  fetchingRepo: string | null = $state(null);
  pullingRepo: string | null = $state(null);
  mergeResult: MergeResult | null = $state(null);
  showMergeConflict: boolean = $state(false);
  pullResult: PullResult | null = $state(null);
  showPullStrategy: boolean = $state(false);
  diffFile: string | null = $state(null);
  diffContent: string = $state('');
  diffStaged: boolean = $state(false);
  loadingDiff: boolean = $state(false);
  errorMsg: string | null = $state(null);
  showError: boolean = $state(false);
  notification: { type: 'success' | 'error'; message: string } | null = $state(null);
  prs: PullRequest[] = $state([]);
  loadingPrs: boolean = $state(false);
  prsError: string | null = $state(null);
  prsLoaded: boolean = $state(false);

  dismissError() {
    this.errorMsg = null;
    this.showError = false;
  }

  showNotification(type: 'success' | 'error', message: string) {
    this.notification = { type, message };
    setTimeout(() => { this.notification = null; }, 4000);
  }

  async loadPRs(force: boolean = false, stateFilter: string = 'open') {
    this.loadingPrs = true;
    this.prsError = null;
    const timeout = setTimeout(() => { this.loadingPrs = false; }, 30000);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const fresh: PullRequest[] = await invoke('get_all_prs', { stateFilter });
      if (!force && this.prsLoaded) {
        const oldIds = this.prs.map(p => p.id).sort().join(',');
        const newIds = fresh.map(p => p.id).sort().join(',');
        if (oldIds === newIds) {
          const changed = this.prs.some((old, i) =>
            old.title !== fresh[i]?.title ||
            old.state !== fresh[i]?.state ||
            old.mergeable !== fresh[i]?.mergeable ||
            old.checks_status !== fresh[i]?.checks_status ||
            old.review_decision !== fresh[i]?.review_decision
          );
          if (!changed) return;
        }
      }
      this.prs = fresh;
      this.prsLoaded = true;
    } catch (e) {
      this.prsError = e instanceof Error ? e.message : 'Failed to load PRs';
    } finally {
      clearTimeout(timeout);
      this.loadingPrs = false;
    }
  }

  async loadAll() {
    try {
      const [reposData, groupsData, configData] = await Promise.all([
        api.getRepos(),
        api.getGroups(),
        api.getConfig(),
      ]);
      this.repos = reposData;
      this.groups = groupsData;
      this.config = configData;
      this.applyTheme(configData.theme);
    } catch (e) {
      console.error('Failed to load app state:', e);
    }
  }

  applyTheme(theme: string) {
    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light');
    } else {
      document.documentElement.setAttribute('data-theme', theme);
    }
  }

  async selectRepo(repo: Repository) {
    if (this.selectedRepo?.id === repo.id) return;
    this.selectedRepo = repo;
    this.gitLog = [];
    this.gitStatus = null;
    this.loadingGitLog = true;
    await this.refreshRepo(repo.id);
    await this.loadGitLog(repo.id);
    await this.loadGitStatus(repo.id);
  }

  async refreshRepo(id: string) {
    this.refreshingRepo = id;
    try {
      const refreshed = await api.refreshRepo(id);
      this.repos = this.repos.map(r => r.id === id ? refreshed : r);
      if (this.selectedRepo?.id === id) {
        this.selectedRepo = refreshed;
      }
    } catch (e) {
      console.error('Failed to refresh repo:', e);
    } finally {
      this.refreshingRepo = null;
    }
  }

  async refreshAllRepos() {
    this.refreshingAll = true;
    try {
      const refreshed = await api.getRepos();
      this.repos = refreshed;
      if (this.selectedRepo) {
        const updated = refreshed.find(r => r.id === this.selectedRepo!.id);
        if (updated) this.selectedRepo = updated;
      }
    } catch (e) {
      console.error('Failed to refresh all repos:', e);
    } finally {
      this.refreshingAll = false;
    }
  }

  async fetchRepo(id: string) {
    this.fetchingRepo = id;
    try {
      console.log('[polyrepo] fetchRepo start', id);
      await api.fetchRepo(id);
      console.log('[polyrepo] fetchRepo done, refreshing');
      await this.refreshRepo(id);
      console.log('[polyrepo] refresh done');
    } catch (e) {
      console.error('[polyrepo] fetchRepo error', e);
      this.errorMsg = `Fetch failed: ${e}`;
      this.showError = true;
    } finally {
      this.fetchingRepo = null;
    }
  }

  async pullRepo(id: string, rebase: boolean = false) {
    this.pullingRepo = id;
    try {
      const result = await api.pullRepo(id, rebase);
      if (result.success) {
        this.showNotification('success', 'Pull successful');
        await this.refreshRepo(id);
        await this.loadGitLog(id);
      } else if (result.needs_rebase) {
        // Divergent branches — show strategy modal.
        this.pullResult = result;
        this.showPullStrategy = true;
      } else {
        this.errorMsg = `Pull failed: ${result.message}`;
        this.showError = true;
      }
    } catch (e) {
      this.errorMsg = `Pull failed: ${e}`;
      this.showError = true;
    } finally {
      this.pullingRepo = null;
    }
  }

  async mergeRepo(id: string) {
    this.mergingRepo = id;
    this.mergeResult = null;
    try {
      const result = await api.mergeRepo(id);
      this.mergeResult = result;
      if (result.conflicts && result.conflicts.length > 0) {
        this.showMergeConflict = true;
      } else if (!result.success) {
        this.showMergeConflict = true;
      } else {
        this.showNotification('success', result.message);
        await this.refreshRepo(id);
        await this.loadGitLog(id);
      }
    } catch (e) {
      this.mergeResult = { success: false, message: String(e) };
      this.showMergeConflict = true;
    } finally {
      this.mergingRepo = null;
    }
  }

  async abortMerge(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('abort_merge', { id });
      this.showMergeConflict = false;
      this.mergeResult = null;
      await this.refreshRepo(id);
    } catch (e) {
      console.error('Failed to abort merge:', e);
    }
  }

  async loadGitLog(id: string) {
    this.loadingGitLog = true;
    try {
      this.gitLog = await api.getGitLog(id, 20);
    } catch (e) {
      console.error('Failed to load git log:', e);
      this.gitLog = [];
    } finally {
      this.loadingGitLog = false;
    }
  }

  async updateConfig(newConfig: AppConfig) {
    try {
      await api.updateConfig(newConfig);
      this.config = newConfig;
      this.applyTheme(newConfig.theme);
    } catch (e) {
      console.error('Failed to update config:', e);
    }
  }

  async saveDefaultBranch(repoId: string, branch: string | null) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_repo_default_branch', { id: repoId, defaultBranch: branch });
      this.repos = this.repos.map(r => r.id === repoId ? { ...r, default_branch: branch } : r);
      if (this.selectedRepo?.id === repoId) {
        this.selectedRepo = { ...this.selectedRepo, default_branch: branch };
      }
    } catch (e) {
      this.showNotification('error', `Failed to save branch: ${e}`);
    }
  }

  async loadGitStatus(id: string) {
    this.loadingGitStatus = true;
    try {
      this.gitStatus = await api.getGitStatus(id);
    } catch (e) {
      console.error('Failed to load git status:', e);
      this.gitStatus = null;
    } finally {
      this.loadingGitStatus = false;
    }
  }

  async loadDiff(id: string, file: string, staged: boolean) {
    this.loadingDiff = true;
    this.diffFile = file;
    this.diffStaged = staged;
    try {
      this.diffContent = await api.getFileDiff(id, file, staged);
    } catch (e) {
      this.diffContent = `Failed to load diff: ${e}`;
    } finally {
      this.loadingDiff = false;
    }
  }

  async stageFile(id: string, file: string) {
    try {
      await api.stageFile(id, file);
      await this.loadGitStatus(id);
    } catch (e) {
      this.showNotification('error', `Stage failed: ${e}`);
    }
  }

  async unstageFile(id: string, file: string) {
    try {
      await api.unstageFile(id, file);
      await this.loadGitStatus(id);
    } catch (e) {
      this.showNotification('error', `Unstage failed: ${e}`);
    }
  }

  async stageAll(id: string) {
    try {
      await api.stageAll(id);
      await this.loadGitStatus(id);
    } catch (e) {
      this.showNotification('error', `Stage all failed: ${e}`);
    }
  }

  async commit(id: string, message: string) {
    try {
      const result = await api.commit(id, message);
      await this.refreshRepo(id);
      await this.loadGitLog(id);
      await this.loadGitStatus(id);
      this.showNotification('success', `Committed ${result.hash || ''}`);
    } catch (e) {
      this.showNotification('error', `Commit failed: ${e}`);
      throw e;
    }
  }

  async pushRepo(id: string) {
    try {
      const result = await api.push(id);
      await this.refreshRepo(id);
      this.showNotification('success', result || 'Pushed successfully');
    } catch (e) {
      this.showNotification('error', `Push failed: ${e}`);
    }
  }

  async switchBranch(id: string, branch: string) {
    try {
      await api.switchBranch(id, branch);
      await this.refreshRepo(id);
      await this.loadGitLog(id);
      await this.loadGitStatus(id);
      this.showNotification('success', `Switched to ${branch}`);
    } catch (e) {
      this.showNotification('error', `Switch failed: ${e}`);
    }
  }

  async discardFile(id: string, filePath: string) {
    try {
      await api.discardFile(id, filePath);
      await this.loadGitStatus(id);
      this.showNotification('success', `Discarded changes to ${filePath}`);
    } catch (e) {
      this.showNotification('error', `Discard failed: ${e}`);
    }
  }

  async stashRepo(id: string) {
    try {
      await api.stash(id);
      await this.refreshRepo(id);
      await this.loadGitStatus(id);
      this.showNotification('success', 'Changes stashed');
    } catch (e) {
      this.showNotification('error', `Stash failed: ${e}`);
    }
  }

  async stashPopRepo(id: string) {
    try {
      await api.stashPop(id);
      await this.refreshRepo(id);
      await this.loadGitStatus(id);
      this.showNotification('success', 'Stash popped');
    } catch (e) {
      this.showNotification('error', `Stash pop failed: ${e}`);
    }
  }

  async resolveFileConflict(id: string, file: string, resolution: string) {
    try {
      await api.resolveConflict(id, file, resolution);
      this.showNotification('success', `Resolved ${file} using ${resolution}`);

      // Reload status so the file changes panel + merge state updates
      const status = await api.getGitStatus(id);
      this.gitStatus = status;

      // Update the merge conflicts list in the modal
      if (this.mergeResult?.conflicts) {
        this.mergeResult = {
          ...this.mergeResult,
          conflicts: this.mergeResult.conflicts.filter(c => c.file !== file),
        };
      }
    } catch (e) {
      this.showNotification('error', `Resolution failed: ${e}`);
    }
  }

  async continueMerge(id: string) {
    try {
      const result = await api.continueMerge(id);
      this.showMergeConflict = false;
      this.mergeResult = null;
      await this.refreshRepo(id);
      await this.loadGitLog(id);
      await this.loadGitStatus(id);
      this.showNotification('success', `Merge complete ${result.hash || ''}`);
    } catch (e) {
      this.showNotification('error', `Continue merge failed: ${e}`);
    }
  }
}

export const app = new AppState();
