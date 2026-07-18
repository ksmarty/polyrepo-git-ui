import { invoke } from '@tauri-apps/api/core';
import type { Repository, RepoGroup, PullRequest, AppConfig } from './types';

export interface GitLogEntry {
  hash: string;
  short_hash: string;
  message: string;
  author: string;
  date: string;
}

export interface MergeResult {
  success: boolean;
  message: string;
  conflicts?: MergeConflict[];
}

export interface MergeConflict {
  file: string;
  status: 'both_modified' | 'both_added' | 'deleted_by_them' | 'deleted_by_us' | 'added_by_them' | 'added_by_us';
}

export interface FileItem {
  path: string;
  status: string;
  change: string;
}

export interface GitStatus {
  staged: FileItem[];
  unstaged: FileItem[];
  has_conflicts: boolean;
  merge_in_progress: boolean;
}

export interface CommitResult {
  success: boolean;
  message: string;
  hash: string | null;
}

export interface PullResult {
  success: boolean;
  message: string;
  needs_rebase: boolean;
}

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

export async function pullRepo(id: string, rebase: boolean): Promise<PullResult> {
  return invoke('pull_repo', { id, rebase });
}

export async function mergeRepo(id: string): Promise<MergeResult> {
  return invoke('merge_repo', { id });
}

export async function getGitLog(id: string, limit?: number): Promise<GitLogEntry[]> {
  return invoke('get_git_log', { id, limit });
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

export async function reorderRepos(repoIds: string[]): Promise<void> {
  return invoke('reorder_repos', { repoIds });
}

export async function reorderGroups(groupIds: string[]): Promise<void> {
  return invoke('reorder_groups', { groupIds });
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

export async function getGitStatus(id: string): Promise<GitStatus> {
  return invoke('get_git_status', { id });
}

export async function getFileDiff(id: string, file: string, staged: boolean): Promise<string> {
  return invoke('get_file_diff', { id, file, staged });
}

export async function stageFile(id: string, file: string): Promise<void> {
  return invoke('stage_file', { id, file });
}

export async function unstageFile(id: string, file: string): Promise<void> {
  return invoke('unstage_file', { id, file });
}

export async function stageAll(id: string): Promise<void> {
  return invoke('stage_all', { id });
}

export async function commit(id: string, message: string): Promise<CommitResult> {
  return invoke('commit', { id, message });
}

export async function push(id: string, force: boolean = false): Promise<string> {
  return invoke('push', { id, force });
}

export async function switchBranch(id: string, branch: string): Promise<void> {
  return invoke('switch_branch', { id, branch });
}

export async function stash(id: string): Promise<void> {
  return invoke('stash', { id });
}

export async function stashPop(id: string): Promise<void> {
  return invoke('stash_pop', { id });
}

export async function resolveConflict(id: string, file: string, resolution: string): Promise<void> {
  return invoke('resolve_conflict', { id, file, resolution });
}

export async function continueMerge(id: string): Promise<CommitResult> {
  return invoke('continue_merge', { id });
}
