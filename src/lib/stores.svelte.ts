import type { Repository, RepoGroup, AppConfig } from './types';
import type { GitLogEntry, MergeResult } from './tauri';
import * as api from './tauri';

class AppState {
  repos: Repository[] = $state([]);
  groups: RepoGroup[] = $state([]);
  selectedRepo: Repository | null = $state(null);
  config: AppConfig = $state({ default_branch: 'main', default_repo_location: '', theme: 'midnight', auto_fetch_on_open: true, fetch_interval_seconds: 300, sidebar_width: 300 });
  gitLog: GitLogEntry[] = $state([]);
  loadingGitLog: boolean = $state(false);
  refreshingRepo: string | null = $state(null);
  refreshingAll: boolean = $state(false);
  mergingRepo: string | null = $state(null);
  mergeResult: MergeResult | null = $state(null);
  showMergeConflict: boolean = $state(false);
  errorMsg: string | null = $state(null);
  showError: boolean = $state(false);

  dismissError() {
    this.errorMsg = null;
    this.showError = false;
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
    this.selectedRepo = repo;
    await this.refreshRepo(repo.id);
    await this.loadGitLog(repo.id);
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
    try {
      await api.fetchRepo(id);
      await this.refreshRepo(id);
    } catch (e) {
      this.errorMsg = `Fetch failed: ${e}`;
      this.showError = true;
      throw e;
    }
  }

  async pullRepo(id: string, rebase: boolean = false) {
    try {
      await api.pullRepo(id, rebase);
      await this.refreshRepo(id);
      await this.loadGitLog(id);
    } catch (e) {
      this.errorMsg = `Pull failed: ${e}`;
      this.showError = true;
      throw e;
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
      } else {
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
      console.error('Failed to save default branch:', e);
    }
  }
}

export const app = new AppState();
