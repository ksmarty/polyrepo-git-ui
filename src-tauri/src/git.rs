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

pub async fn add_repo(
    path: &str,
    group_id: Option<&str>,
    default_branch: Option<&str>,
) -> Result<Repository, String> {
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
    let sync_status = get_sync_status(path, &current_branch, default_branch).await?;

    Ok(Repository {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: path.to_string_lossy().to_string(),
        remote_url,
        github_owner,
        github_repo,
        default_branch: None,
        group_id: group_id.map(|s| s.to_string()),
        order: 0,
        local_branches,
        current_branch,
        sync_status: Some(sync_status),
    })
}

pub async fn refresh_repo(
    path: &str,
    default_branch: Option<&str>,
) -> Result<Repository, String> {
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
    let sync_status = get_sync_status(path, &current_branch, default_branch).await?;

    Ok(Repository {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        path: path.to_string_lossy().to_string(),
        remote_url,
        github_owner,
        github_repo,
        default_branch: None,
        group_id: None,
        order: 0,
        local_branches,
        current_branch,
        sync_status: Some(sync_status),
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileItem {
    pub path: String,
    pub status: String, // "staged" or "unstaged"
    pub change: String, // "M" "A" "D" "R" "??"
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitStatus {
    pub staged: Vec<FileItem>,
    pub unstaged: Vec<FileItem>,
    pub has_conflicts: bool,
    pub merge_in_progress: bool,
}

pub async fn get_git_status(path: &str) -> Result<GitStatus, String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git status: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut has_conflicts = false;

    for line in stdout.lines() {
        if line.len() < 3 { continue; }
        let chars: Vec<char> = line.chars().collect();
        let index_status = chars[0];
        let worktree_status = chars[1];
        let file_path = line[3..].trim().to_string();

        if index_status == 'U' || worktree_status == 'U'
            || line.contains("UU") || line.contains("AA") || line.contains("DD") {
            has_conflicts = true;
            unstaged.push(FileItem {
                path: file_path,
                status: "unstaged".to_string(),
                change: "U".to_string(),
            });
            continue;
        }

        if index_status != ' ' && index_status != '?' {
            staged.push(FileItem {
                path: file_path.clone(),
                status: "staged".to_string(),
                change: index_status.to_string(),
            });
        }

        if worktree_status != ' ' {
            unstaged.push(FileItem {
                path: file_path,
                status: "unstaged".to_string(),
                change: worktree_status.to_string(),
            });
        } else if index_status == '?' {
            unstaged.push(FileItem {
                path: file_path,
                status: "unstaged".to_string(),
                change: "?".to_string(),
            });
        }
    }

    let merge_in_progress = std::path::Path::new(path)
        .join(".git")
        .join("MERGE_HEAD")
        .exists();

    Ok(GitStatus {
        staged,
        unstaged,
        has_conflicts,
        merge_in_progress,
    })
}

pub async fn stage_file(path: &str, file: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", file])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to stage file: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stage: {}", stderr));
    }
    Ok(())
}

pub async fn unstage_file(path: &str, file: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["reset", "HEAD", "--", file])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to unstage file: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to unstage: {}", stderr));
    }
    Ok(())
}

pub async fn stage_all(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to stage all: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stage all: {}", stderr));
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommitResult {
    pub success: bool,
    pub message: String,
    pub hash: Option<String>,
}

pub async fn commit(path: &str, message: &str) -> Result<CommitResult, String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git commit: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("Commit failed: {}", msg));
    }

    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .await
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(CommitResult {
        success: true,
        message: stdout.trim().to_string(),
        hash,
    })
}

pub async fn push(path: &str, force: bool) -> Result<String, String> {
    let mut args = vec!["push".to_string()];
    if force {
        args.push("--force-with-lease".to_string());
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git push: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Push failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(format!("{}{}", stdout, stderr).trim().to_string())
}

pub async fn switch_branch(path: &str, branch: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["switch", branch])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git switch: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Switch failed: {}", stderr));
    }
    Ok(())
}

pub async fn stash(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["stash"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to stash: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Stash failed: {}", stderr));
    }
    Ok(())
}

pub async fn stash_pop(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["stash", "pop"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to stash pop: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Stash pop failed: {}", stderr));
    }
    Ok(())
}

pub async fn resolve_conflict(path: &str, file: &str, resolution: &str) -> Result<(), String> {
    let output = match resolution {
        "ours" => {
            Command::new("git")
                .args(["checkout", "--ours", file])
                .current_dir(path)
                .output()
                .await
                .map_err(|e| format!("Failed to checkout --ours: {}", e))?
        }
        "theirs" => {
            Command::new("git")
                .args(["checkout", "--theirs", file])
                .current_dir(path)
                .output()
                .await
                .map_err(|e| format!("Failed to checkout --theirs: {}", e))?
        }
        _ => return Err(format!("Invalid resolution: {}", resolution)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to resolve conflict: {}", stderr));
    }

    stage_file(path, file).await?;
    Ok(())
}

pub async fn continue_merge(path: &str) -> Result<CommitResult, String> {
    let status = get_git_status(path).await?;
    if status.has_conflicts {
        return Err("Cannot continue merge: unresolved conflicts remain".to_string());
    }

    let output = Command::new("git")
        .args(["commit", "--no-edit"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to continue merge: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if stderr.is_empty() { stdout } else { stderr };
        return Err(format!("Merge continue failed: {}", msg));
    }

    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .await
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(CommitResult {
        success: true,
        message: stdout.trim().to_string(),
        hash,
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct PullResult {
    pub success: bool,
    pub message: String,
    pub needs_rebase: bool,
}

pub async fn pull_repo(path: &str, rebase: bool) -> Result<PullResult, String> {
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

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(PullResult {
            success: true,
            message: if stdout.is_empty() { "Already up to date".to_string() } else { stdout },
            needs_rebase: false,
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Divergent branches — user needs to choose a strategy.
    if stderr.contains("divergent branches") || stderr.contains("need to specify how to reconcile") {
        return Ok(PullResult {
            success: false,
            message: stderr.trim().to_string(),
            needs_rebase: true,
        });
    }

    Err(format!("Git pull failed: {}", stderr))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MergeConflict {
    pub file: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MergeResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflicts: Option<Vec<MergeConflict>>,
}

pub async fn merge_repo(path: &str, default_branch: Option<&str>) -> Result<MergeResult, String> {
    // Try configured default branch first, then common fallbacks.
    let mut targets: Vec<String> = Vec::new();
    if let Some(branch) = default_branch {
        if !branch.is_empty() {
            targets.push(format!("origin/{}", branch));
        }
    }
    targets.push("origin/main".to_string());
    targets.push("origin/master".to_string());

    let mut seen = std::collections::HashSet::new();
    targets.retain(|t| seen.insert(t.clone()));

    for target in &targets {
        let output = Command::new("git")
            .args(["merge", target])
            .current_dir(path)
            .output()
            .await
            .map_err(|e| format!("Failed to execute git merge: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(MergeResult {
                success: true,
                message: if stdout.is_empty() { "Already up to date".to_string() } else { stdout },
                conflicts: None,
            });
        }

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // If this target doesn't exist, try the next one.
        if stderr.contains("not found") || stderr.contains("does not refer to a commit") {
            continue;
        }

        // Check for merge conflicts.
        if stderr.contains("CONFLICT") || stderr.contains("Automatic merge failed") {
            let conflicts = get_merge_conflicts(path).await;
            return Ok(MergeResult {
                success: false,
                message: stderr.trim().to_string(),
                conflicts: Some(conflicts),
            });
        }

        // Check for "not something we can merge" — branch doesn't exist or already up to date.
        if stderr.contains("not something we can merge") || stderr.contains("Already up to date") {
            return Ok(MergeResult {
                success: false,
                message: format!("Cannot merge '{}': {}", target, stderr.trim()),
                conflicts: None,
            });
        }

        // Check for divergent branches during merge (uncommitted changes).
        if stderr.contains("uncommitted changes") || stderr.contains("Changes to be committed") {
            return Ok(MergeResult {
                success: false,
                message: format!("Cannot merge: you have uncommitted changes. Stash or commit them first.\n\n{}", stderr.trim()),
                conflicts: None,
            });
        }

        // Other error.
        return Ok(MergeResult {
            success: false,
            message: stderr.trim().to_string(),
            conflicts: None,
        });
    }

    Err("No valid merge target found".to_string())
}

pub async fn merge_repo_with_target(path: &str, target: &str) -> Result<MergeResult, String> {
    let output = Command::new("git")
        .args(["merge", target])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git merge: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(MergeResult {
            success: true,
            message: if stdout.is_empty() { "Already up to date".to_string() } else { stdout },
            conflicts: None,
        });
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check for merge conflicts.
    if stderr.contains("CONFLICT") || stderr.contains("Automatic merge failed") {
        let conflicts = get_merge_conflicts(path).await;
        return Ok(MergeResult {
            success: false,
            message: stderr.trim().to_string(),
            conflicts: Some(conflicts),
        });
    }

    // Other error.
    Ok(MergeResult {
        success: false,
        message: stderr.trim().to_string(),
        conflicts: None,
    })
}

async fn get_merge_conflicts(path: &str) -> Vec<MergeConflict> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(path)
        .output()
        .await
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|file| MergeConflict {
            file: file.to_string(),
            status: "both_modified".to_string(),
        })
        .collect()
}

pub async fn abort_merge(path: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["merge", "--abort"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git merge --abort: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git merge --abort failed: {}", stderr));
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GitLogEntry {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

pub async fn get_git_log(path: &str, limit: u32) -> Result<Vec<GitLogEntry>, String> {
    let format = "%H\t%h\t%s\t%an\t%ar";
    let output = Command::new("git")
        .args([
            "log",
            &format!("--format={}", format),
            &format!("-{}", limit),
        ])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git log: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git log failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 5 {
                Some(GitLogEntry {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                    date: parts[4].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(entries)
}

pub async fn clone_repo(
    url: &str,
    path: &str,
    default_branch: Option<&str>,
    token: Option<&str>,
) -> Result<Repository, String> {
    let effective_url = if let Some(t) = token {
        // Inject token into HTTPS GitHub URLs for private repo access
        if url.starts_with("https://github.com/") {
            url.replacen("https://github.com/", &format!("https://x-access-token:{}@github.com/", t), 1)
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    };

    let output = Command::new("git")
        .arg("clone")
        .arg(&effective_url)
        .arg(path)
        .output()
        .await
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git clone failed: {}", stderr));
    }

    add_repo(path, None, default_branch).await
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

async fn get_sync_status(
    path: &Path,
    current_branch: &str,
    default_branch: Option<&str>,
) -> Result<SyncStatus, String> {
    let dirty = is_dirty(path).await?;
    let (behind, ahead) = get_behind_ahead(path, current_branch, default_branch).await?;

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

async fn get_behind_ahead(
    path: &Path,
    current_branch: &str,
    default_branch: Option<&str>,
) -> Result<(u32, u32), String> {
    // Use HEAD for the local side when the branch name is unavailable (detached HEAD).
    let local_ref = if current_branch.is_empty() { "HEAD" } else { current_branch };

    // Build a list of refs to compare against, in priority order.
    // The "base branch" the user configured is the most important comparison.
    let mut candidates: Vec<String> = Vec::new();

    // 1. Configured default branch (per-repo override or global setting).
    if let Some(branch) = default_branch {
        if !branch.is_empty() {
            candidates.push(format!("origin/{}", branch));
            candidates.push(format!("upstream/{}", branch));
        }
    }

    // 2. The branch's upstream tracking branch.
    let upstream_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{}@{{upstream}}", current_branch)])
        .current_dir(path)
        .output()
        .await
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });

    if upstream_output.status.success() {
        let upstream = String::from_utf8_lossy(&upstream_output.stdout).trim().to_string();
        if !upstream.is_empty() {
            candidates.push(upstream);
        }
    }

    // 3. Common fallback base branches on common remotes.
    candidates.push("origin/main".to_string());
    candidates.push("upstream/main".to_string());
    candidates.push("origin/master".to_string());
    candidates.push("upstream/master".to_string());

    // Deduplicate while preserving priority.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()));

    for upstream in &candidates {
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", &format!("{}...{}", upstream, local_ref)])
            .current_dir(path)
            .output()
            .await
            .unwrap_or_else(|_| std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });

        if output.status.success() {
            let counts = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let parts: Vec<&str> = counts.split('\t').collect();
            if parts.len() == 2 {
                let behind = parts[0].parse().unwrap_or(0);
                let ahead = parts[1].parse().unwrap_or(0);
                return Ok((behind, ahead));
            }
        }
    }

    // Could not compare against any ref. Default to in-sync so the UI doesn't block.
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
        let result = add_repo("/nonexistent/path/to/repo", None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_add_repo_not_git() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("not-a-repo");
        fs::create_dir_all(&repo_path).unwrap();

        let result = add_repo(repo_path.to_str().unwrap(), None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a git repository"));
    }

    #[tokio::test]
    async fn test_add_repo_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("my-repo");
        create_test_working_repo(&repo_path);

        let result = add_repo(repo_path.to_str().unwrap(), None, None).await;
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

        let result = add_repo(repo_path.to_str().unwrap(), Some("group-123"), None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().group_id, Some("group-123".to_string()));
    }

    #[tokio::test]
    async fn test_refresh_repo_nonexistent() {
        let result = refresh_repo("/nonexistent/path", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_repo_valid() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_path = temp_dir.path().join("refresh-repo");
        create_test_working_repo(&repo_path);

        let result = refresh_repo(repo_path.to_str().unwrap(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_behind_ahead_uses_default_branch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let remote_path = temp_dir.path().join("remote.git");
        let local_path = temp_dir.path().join("local");

        // Create a bare remote repo.
        create_test_working_repo(&remote_path);
        std::process::Command::new("git")
            .args(["config", "--bool", "core.bare", "true"])
            .current_dir(&remote_path)
            .output()
            .unwrap();

        // Clone the remote and name the default branch "develop".
        std::process::Command::new("git")
            .args([
                "clone",
                "--branch",
                "main",
                remote_path.to_str().unwrap(),
                local_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["branch", "-m", "develop"])
            .current_dir(&local_path)
            .output()
            .unwrap();

        // Add two commits to the remote's "develop" equivalent (remote HEAD).
        let remote_worktree = temp_dir.path().join("remote-work");
        std::process::Command::new("git")
            .args(["clone", remote_path.to_str().unwrap(), remote_worktree.to_str().unwrap()])
            .output()
            .unwrap();
        fs::write(remote_worktree.join("a.md"), "a").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&remote_worktree)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "commit-1"])
            .current_dir(&remote_worktree)
            .output()
            .unwrap();
        fs::write(remote_worktree.join("b.md"), "b").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&remote_worktree)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "commit-2"])
            .current_dir(&remote_worktree)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(&remote_worktree)
            .output()
            .unwrap();

        // Fetch in the local repo so origin/main exists but no upstream is set.
        std::process::Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&local_path)
            .output()
            .unwrap();

        let (behind, ahead) = get_behind_ahead(
            &local_path,
            "develop",
            Some("main"), // configured default branch is "main" (remote has origin/main)
        )
        .await
        .unwrap();
        assert_eq!(behind, 2, "expected 2 commits behind origin/main");
        assert_eq!(ahead, 0);
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
