use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub behind: u32,
    pub ahead: u32,
    pub is_dirty: bool,
    pub up_to_date: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub default_branch: Option<String>,
    pub group_id: Option<String>,
    pub order: u32,
    pub local_branches: Vec<String>,
    pub remote_branches: Vec<String>,
    pub current_branch: String,
    pub sync_status: Option<SyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGroup {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: String,
    pub number: u32,
    pub title: String,
    pub head_ref: String,
    pub base_ref: String,
    pub repo_id: String,
    pub repo_name: String,
    pub state: String,
    pub mergeable: bool,
    pub behind_count: u32,
    pub checks_status: String,
    pub review_decision: Option<String>,
    pub html_url: String,
    pub body: Option<String>,
    pub author: Option<String>,
    pub requested_reviewers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAuth {
    pub method: Option<String>,
    pub token: Option<String>,
    pub user: Option<String>,
}

impl Default for GitHubAuth {
    fn default() -> Self {
        Self {
            method: None,
            token: None,
            user: None,
        }
    }
}

impl RepoGroup {
    pub fn new(name: &str, parent_id: Option<&str>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            order: 0,
        }
    }
}

impl Repository {
    pub fn parse_github_url(url: &str) -> Option<(String, String)> {
        // Handle SSH URLs: git@github.com:owner/repo.git
        if let Some(rest) = url.strip_prefix("git@github.com:") {
            let path = rest.trim_end_matches(".git");
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() == 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        }

        // Handle HTTPS URLs: https://github.com/owner/repo.git
        if let Some(rest) = url.strip_prefix("https://github.com/") {
            let path = rest.trim_end_matches(".git");
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() == 2 {
                return Some((parts[0].to_string(), parts[1].to_string()));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_ssh() {
        let result = Repository::parse_github_url("git@github.com:owner/repo.git");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_ssh_no_dotgit() {
        let result = Repository::parse_github_url("git@github.com:owner/repo");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_https() {
        let result = Repository::parse_github_url("https://github.com/owner/repo.git");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_https_no_dotgit() {
        let result = Repository::parse_github_url("https://github.com/owner/repo");
        assert_eq!(result, Some(("owner".to_string(), "repo".to_string())));
    }

    #[test]
    fn test_parse_github_url_invalid() {
        assert_eq!(Repository::parse_github_url("https://gitlab.com/owner/repo"), None);
        assert_eq!(Repository::parse_github_url("not-a-url"), None);
        assert_eq!(Repository::parse_github_url(""), None);
    }

    #[test]
    fn test_parse_github_url_no_trailing_slash() {
        let result = Repository::parse_github_url("https://github.com/org/project");
        assert_eq!(result, Some(("org".to_string(), "project".to_string())));
    }

    #[test]
    fn test_github_auth_default() {
        let auth = GitHubAuth::default();
        assert!(auth.method.is_none());
        assert!(auth.token.is_none());
        assert!(auth.user.is_none());
    }

    #[test]
    fn test_repo_group_new() {
        let group = RepoGroup::new("frontend", None);
        assert_eq!(group.name, "frontend");
        assert!(group.parent_id.is_none());
        assert_eq!(group.order, 0);
        assert!(!group.id.is_empty());
    }

    #[test]
    fn test_repo_group_new_with_parent() {
        let group = RepoGroup::new("apps", Some("root-id"));
        assert_eq!(group.name, "apps");
        assert_eq!(group.parent_id, Some("root-id".to_string()));
    }

    #[test]
    fn test_sync_status_up_to_date() {
        let status = SyncStatus {
            behind: 0,
            ahead: 0,
            is_dirty: false,
            up_to_date: true,
        };
        assert!(status.up_to_date);
        assert!(!status.is_dirty);
    }
}
