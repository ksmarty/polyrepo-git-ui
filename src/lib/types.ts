export interface SyncStatus {
  behind: number;
  ahead: number;
  is_dirty: boolean;
  up_to_date: boolean;
}

export interface Repository {
  id: string;
  name: string;
  path: string;
  remote_url: string | null;
  github_owner: string | null;
  github_repo: string | null;
  default_branch: string | null;
  group_id: string | null;
  order: number;
  local_branches: string[];
  remote_branches: string[];
  current_branch: string;
  sync_status: SyncStatus | null;
}

export interface RepoGroup {
  id: string;
  name: string;
  parent_id: string | null;
  order: number;
}

export interface PullRequest {
  id: string;
  number: number;
  title: string;
  head_ref: string;
  base_ref: string;
  repo_id: string;
  repo_name: string;
  state: 'open' | 'closed' | 'merged';
  mergeable: boolean;
  behind_count: number;
  checks_status: 'pending' | 'success' | 'failure' | 'cancelled';
  review_decision: 'approved' | 'changes_requested' | 'review_required' | null;
  html_url: string;
  body: string | null;
  author: string | null;
  requested_reviewers: string[];
}

export interface GitHubAuth {
  method: 'oauth' | 'pat' | null;
  token: string | null;
  user: string | null;
}

export interface AppConfig {
  default_repo_location: string;
  theme: string;
  auto_fetch_on_open: boolean;
  fetch_interval_seconds: number;
  sidebar_width: number;
  pr_density: 'compact' | 'relaxed';
  minimize_on_close: boolean;
}
