use std::path::Path;

use tokio::process::Command;

use crate::models::{Repository, SyncStatus};

pub async fn check_git_installed() -> Result<bool, String> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    Ok(output.status.success())
}

pub async fn add_repo(path: &str, group_id: Option<&str>) -> Result<Repository, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !path.join(".git").exists() {
        return Err("Not a git repository".to_string());
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let remote_url = get_remote_url(path).await?;
    let (github_owner, github_repo) = remote_url
        .as_ref()
        .and_then(|url| Repository::parse_github_url(url))
        .unzip();

    let current_branch = get_current_branch(path).await?;
    let local_branches = get_local_branches(path).await?;
    let sync_status = get_sync_status(path, &current_branch).await?;

    Ok(Repository {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: path.to_string_lossy().to_string(),
        remote_url,
        github_owner,
        github_repo,
        default_branch: None,
        group_id: group_id.map(|s| s.to_string()),
        local_branches,
        current_branch,
        sync_status: Some(sync_status),
    })
}

pub async fn refresh_repo(path: &str) -> Result<Repository, String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let remote_url = get_remote_url(path).await?;
    let (github_owner, github_repo) = remote_url
        .as_ref()
        .and_then(|url| Repository::parse_github_url(url))
        .unzip();

    let current_branch = get_current_branch(path).await?;
    let local_branches = get_local_branches(path).await?;
    let sync_status = get_sync_status(path, &current_branch).await?;

    Ok(Repository {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: path.to_string_lossy().to_string(),
        remote_url,
        github_owner,
        github_repo,
        default_branch: None,
        group_id: None,
        local_branches,
        current_branch,
        sync_status: Some(sync_status),
    })
}

pub async fn fetch_repo(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .arg("fetch")
        .arg("--all")
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git fetch: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git fetch failed: {}", stderr));
    }

    Ok(())
}

pub async fn pull_repo(path: &str, rebase: bool) -> Result<(), String> {
    let mut args = vec!["pull".to_string()];
    if rebase {
        args.push("--rebase".to_string());
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git pull: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git pull failed: {}", stderr));
    }

    Ok(())
}

pub async fn clone_repo(url: &str, path: &str) -> Result<Repository, String> {
    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }

    add_repo(path, None).await
}

async fn get_remote_url(path: &Path) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git config: {}", e))?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(url))
    } else {
        Ok(None)
    }
}

async fn get_current_branch(path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git branch: {}", e))?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch)
    } else {
        Ok("main".to_string())
    }
}

async fn get_local_branches(path: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git branch: {}", e))?;

    if output.status.success() {
        let branches = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty())
            .collect();
        Ok(branches)
    } else {
        Ok(Vec::new())
    }
}

async fn get_sync_status(path: &Path, current_branch: &str) -> Result<SyncStatus, String> {
    let dirty = is_dirty(path).await?;
    let (behind, ahead) = get_behind_ahead(path, current_branch).await?;

    Ok(SyncStatus {
        behind,
        ahead,
        is_dirty: dirty,
        up_to_date: behind == 0 && ahead == 0,
    })
}

async fn is_dirty(path: &Path) -> Result<bool, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git status: {}", e))?;

    Ok(!output.stdout.is_empty())
}

async fn get_behind_ahead(path: &Path, current_branch: &str) -> Result<(u32, u32), String> {
    let upstream = format!("origin/{}", current_branch);

    let output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", &format!("{}...{}", upstream, current_branch)])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git rev-list: {}", e))?;

    if output.status.success() {
        let counts = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = counts.split('\t').collect();
        if parts.len() == 2 {
            let behind = parts[0].parse().unwrap_or(0);
            let ahead = parts[1].parse().unwrap_or(0);
            return Ok((behind, ahead));
        }
    }

    Ok((0, 0))
}

pub async fn scan_directory_for_repos(path: &str) -> Result<Vec<String>, String> {
    let dir = std::path::Path::new(path);
    if !dir.exists() {
        return Err("Path does not exist".to_string());
    }

    // If the path itself is a git repo, return it directly
    if dir.is_dir() && dir.join(".git").exists() {
        return Ok(vec![dir.to_string_lossy().to_string()]);
    }

    if !dir.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let mut repos = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        if entry_path.is_dir() && entry_path.join(".git").exists() {
            repos.push(entry_path.to_string_lossy().to_string());
        }
    }
    repos.sort();
    Ok(repos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_working_repo(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("failed to init repo");

        // Create an initial commit so HEAD exists
        fs::write(dir.join("README.md"), "test").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(dir)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "init", "--allow-empty"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[tokio::test]
    async fn test_add_repo_nonexistent_path() {
        let result = add_repo("/nonexistent/path/to/repo", None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_add_repo_not_git() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("not-a-repo");
        fs::create_dir_all(&repo_path).unwrap();

        let result = add_repo(repo_path.to_str().unwrap(), None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_add_repo_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("my-repo");
        create_test_working_repo(&repo_path);

        let result = add_repo(repo_path.to_str().unwrap(), None).await;
        assert!(result.is_ok());

        let repo = result.unwrap();
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.path, repo_path.to_str().unwrap());
        assert!(repo.id.len() > 0);
    }

    #[tokio::test]
    async fn test_add_repo_with_group() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("grouped-repo");
        create_test_working_repo(&repo_path);

        let result = add_repo(repo_path.to_str().unwrap(), Some("group-123")).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().group_id, Some("group-123".to_string()));
    }

    #[tokio::test]
    async fn test_refresh_repo_nonexistent() {
        let result = refresh_repo("/nonexistent/path").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_repo_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("refresh-repo");
        create_test_working_repo(&repo_path);

        let result = refresh_repo(repo_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scan_directory_for_repos_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = scan_directory_for_repos(temp_dir.path().to_str().unwrap()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_scan_directory_for_repos_finds_git_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base = temp_dir.path();

        // Create two git repos and one non-git dir
        create_test_working_repo(&base.join("repo-a"));
        create_test_working_repo(&base.join("repo-b"));
        fs::create_dir_all(base.join("not-a-repo")).unwrap();

        let result = scan_directory_for_repos(base.to_str().unwrap()).await;
        assert!(result.is_ok());

        let repos = result.unwrap();
        assert_eq!(repos.len(), 2);
        assert!(repos.iter().any(|r| r.contains("repo-a")));
        assert!(repos.iter().any(|r| r.contains("repo-b")));
    }

    #[tokio::test]
    async fn test_scan_directory_for_repos_nonexistent() {
        let result = scan_directory_for_repos("/nonexistent/dir").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scan_directory_for_repos_file_not_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file = temp_dir.path().join("not-a-dir");
        fs::write(&file, "content").unwrap();

        let result = scan_directory_for_repos(file.to_str().unwrap()).await;
        assert!(result.is_err());
    }
}
