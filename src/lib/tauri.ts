import { invoke } from '@tauri-apps/api/core';
import type { Repository, RepoGroup, PullRequest, AppConfig } from './types';

export async function getRepos(): Promise<Repository[]> {
  return invoke('get_repos');
}

export async function addRepo(path: string, groupId?: string): Promise<Repository> {
  return invoke('add_repo', { path, groupId });
}

export async function removeRepo(id: string): Promise<void> {
  return invoke('remove_repo', { id });
}

export async function refreshRepo(id: string): Promise<Repository> {
  return invoke('refresh_repo', { id });
}

export async function fetchRepo(id: string): Promise<void> {
  return invoke('fetch_repo', { id });
}

export async function pullRepo(id: string, rebase: boolean): Promise<void> {
  return invoke('pull_repo', { id, rebase });
}

export async function getGroups(): Promise<RepoGroup[]> {
  return invoke('get_groups');
}

export async function createGroup(name: string, parentId?: string): Promise<RepoGroup> {
  return invoke('create_group', { name, parentId });
}

export async function updateGroup(id: string, name: string): Promise<void> {
  return invoke('update_group', { id, name });
}

export async function deleteGroup(id: string): Promise<void> {
  return invoke('delete_group', { id });
}

export async function moveRepoToGroup(repoId: string, groupId: string | null): Promise<void> {
  return invoke('move_repo_to_group', { repoId, groupId });
}

export async function getAllPRs(): Promise<PullRequest[]> {
  return invoke('get_all_prs');
}

export async function syncPR(prId: string, useRebase: boolean): Promise<void> {
  return invoke('sync_pr', { prId, useRebase });
}

export async function getConfig(): Promise<AppConfig> {
  return invoke('get_config');
}

export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke('update_config', { config });
}

export async function cloneRepo(url: string, path: string): Promise<Repository> {
  return invoke('clone_repo', { url, path });
}

export async function checkGitInstalled(): Promise<boolean> {
  return invoke('check_git_installed');
}

export async function scanDirectoryForRepos(path: string): Promise<string[]> {
  return invoke('scan_directory_for_repos', { path });
}
