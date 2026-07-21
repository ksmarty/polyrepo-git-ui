mod config;
mod git;
mod github;
mod models;

use config::AppConfig;
use models::{PullRequest, RepoGroup, Repository};
use std::net::TcpListener;
use tauri::Manager;

#[tauri::command]
async fn get_repos(state: tauri::State<'_, AppState>) -> Result<Vec<Repository>, String> {
    let config = state.config.lock().await;
    Ok(config.repos.clone())
}

#[tauri::command]
async fn add_repo(
    state: tauri::State<'_, AppState>,
    path: String,
    group_id: Option<String>,
) -> Result<Repository, String> {
    let default_branch = {
        let config = state.config.lock().await;
        config.app_config.default_branch.clone()
    };
    let repo = git::add_repo(&path, group_id.as_deref(), Some(&default_branch)).await?;
    let mut config = state.config.lock().await;
    config.repos.push(repo.clone());
    config.save().map_err(|e| e.to_string())?;
    Ok(repo)
}

#[tauri::command]
async fn remove_repo(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.repos.retain(|r| r.id != id);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn refresh_repo(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Repository, String> {
    let (repo_path, repo_default_branch, global_default_branch) = {
        let config = state.config.lock().await;
        let repo = config
            .repos
            .iter()
            .find(|r| r.id == id)
            .ok_or("Repo not found")?;
        (
            repo.path.clone(),
            repo.default_branch.clone(),
            config.app_config.default_branch.clone(),
        )
    };
    let effective_default_branch = repo_default_branch.as_deref().or(Some(global_default_branch.as_str()));
    let refreshed = git::refresh_repo(&repo_path, effective_default_branch).await?;
    let mut config = state.config.lock().await;
    if let Some(existing) = config.repos.iter_mut().find(|r| r.id == id) {
        existing.name = refreshed.name;
        existing.remote_url = refreshed.remote_url;
        existing.github_owner = refreshed.github_owner;
        existing.github_repo = refreshed.github_repo;
        existing.local_branches = refreshed.local_branches;
        existing.remote_branches = refreshed.remote_branches;
        existing.current_branch = refreshed.current_branch;
        existing.sync_status = refreshed.sync_status;

        // Auto-detect default branch if not set
        if existing.default_branch.is_none() {
            existing.default_branch = git::detect_default_branch(std::path::Path::new(&repo_path)).await;
        }
    }
    config.save().map_err(|e| e.to_string())?;
    Ok(config.repos.iter().find(|r| r.id == id).cloned().ok_or("Repo not found".to_string())?)
}

#[tauri::command]
async fn update_repo_default_branch(
    state: tauri::State<'_, AppState>,
    id: String,
    default_branch: Option<String>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    if let Some(repo) = config.repos.iter_mut().find(|r| r.id == id) {
        repo.default_branch = default_branch;
    }
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn refresh_all_repos(state: tauri::State<'_, AppState>) -> Result<Vec<Repository>, String> {
    let repo_ids: Vec<String> = {
        let config = state.config.lock().await;
        config.repos.iter().map(|r| r.id.clone()).collect()
    };
    let mut results = Vec::new();
    for id in repo_ids {
        match refresh_repo(state.clone(), id).await {
            Ok(repo) => results.push(repo),
            Err(e) => eprintln!("Failed to refresh repo: {}", e),
        }
    }
    Ok(results)
}

#[tauri::command]
async fn fetch_repo(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config
            .repos
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.path.clone())
            .ok_or("Repo not found")?
    };
    git::fetch_repo(&repo_path).await
}

#[tauri::command]
async fn pull_repo(
    state: tauri::State<'_, AppState>,
    id: String,
    rebase: bool,
) -> Result<git::PullResult, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config
            .repos
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.path.clone())
            .ok_or("Repo not found")?
    };
    git::pull_repo(&repo_path, rebase).await
}

#[tauri::command]
async fn merge_repo(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<git::MergeResult, String> {
    let (repo_path, repo_default_branch, global_default_branch) = {
        let config = state.config.lock().await;
        let repo = config
            .repos
            .iter()
            .find(|r| r.id == id)
            .ok_or("Repo not found")?;
        (
            repo.path.clone(),
            repo.default_branch.clone(),
            config.app_config.default_branch.clone(),
        )
    };
    let effective_default_branch = repo_default_branch.as_deref().or(Some(global_default_branch.as_str()));
    git::merge_repo(&repo_path, effective_default_branch).await
}

#[tauri::command]
async fn merge_repo_with_target(
    state: tauri::State<'_, AppState>,
    id: String,
    target: String,
) -> Result<git::MergeResult, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config
            .repos
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.path.clone())
            .ok_or("Repo not found")?
    };
    git::merge_repo_with_target(&repo_path, &target).await
}

#[tauri::command]
async fn abort_merge(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config
            .repos
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.path.clone())
            .ok_or("Repo not found")?
    };
    git::abort_merge(&repo_path).await
}

#[tauri::command]
async fn get_git_log(
    state: tauri::State<'_, AppState>,
    id: String,
    limit: Option<u32>,
) -> Result<Vec<git::GitLogEntry>, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config
            .repos
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.path.clone())
            .ok_or("Repo not found")?
    };
    git::get_git_log(&repo_path, limit.unwrap_or(20)).await
}

#[tauri::command]
async fn get_groups(state: tauri::State<'_, AppState>) -> Result<Vec<RepoGroup>, String> {
    let config = state.config.lock().await;
    Ok(config.groups.clone())
}

#[tauri::command]
async fn create_group(
    state: tauri::State<'_, AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<RepoGroup, String> {
    let group = RepoGroup::new(&name, parent_id.as_deref());
    let mut config = state.config.lock().await;
    config.groups.push(group.clone());
    config.save().map_err(|e| e.to_string())?;
    Ok(group)
}

#[tauri::command]
async fn update_group(
    state: tauri::State<'_, AppState>,
    id: String,
    name: String,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    if let Some(group) = config.groups.iter_mut().find(|g| g.id == id) {
        group.name = name;
    }
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn delete_group(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.groups.retain(|g| g.id != id);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn move_repo_to_group(
    state: tauri::State<'_, AppState>,
    repo_id: String,
    group_id: Option<String>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    if let Some(repo) = config.repos.iter_mut().find(|r| r.id == repo_id) {
        repo.group_id = group_id;
    }
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn reorder_groups(
    state: tauri::State<'_, AppState>,
    group_ids: Vec<String>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    for (i, id) in group_ids.iter().enumerate() {
        if let Some(group) = config.groups.iter_mut().find(|g| g.id == *id) {
            group.order = i as u32;
        }
    }
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn reorder_repos(
    state: tauri::State<'_, AppState>,
    repo_ids: Vec<String>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    for (i, id) in repo_ids.iter().enumerate() {
        if let Some(repo) = config.repos.iter_mut().find(|r| r.id == *id) {
            repo.order = i as u32;
        }
    }
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_all_prs(state: tauri::State<'_, AppState>) -> Result<Vec<PullRequest>, String> {
    let (repos_data, token) = {
        let config = state.config.lock().await;
        let repos_data: Vec<(String, String, String)> = config
            .repos
            .iter()
            .filter(|r| r.github_owner.is_some() && r.github_repo.is_some())
            .map(|r| {
                (
                    r.id.clone(),
                    r.github_owner.clone().unwrap_or_default(),
                    r.github_repo.clone().unwrap_or_default(),
                )
            })
            .collect();
        let token = config.github_auth.token.clone();
        (repos_data, token)
    };
    let token = match token {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };
    let mut all_prs = Vec::new();
    for (repo_id, owner, name) in repos_data {
        match github::get_prs(&token, &owner, &name, &repo_id).await {
            Ok(prs) => all_prs.extend(prs),
            Err(e) => {
                eprintln!("[polyrepo] PR fetch {}/{}: {}", owner, name, e);
            }
        }
    }
    Ok(all_prs)
}

#[tauri::command]
async fn sync_pr(
    state: tauri::State<'_, AppState>,
    pr_id: String,
    use_rebase: bool,
) -> Result<(), String> {
    let token = {
        let config = state.config.lock().await;
        config.github_auth.token.clone()
    };
    let token = token.as_ref().ok_or("GitHub authentication required")?;
    github::sync_pr(token, &pr_id, use_rebase).await
}

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.app_config.clone())
}

#[tauri::command]
async fn get_config_path() -> Result<String, String> {
    config::Config::config_path_string().ok_or("Could not determine config path".to_string())
}

#[tauri::command]
async fn open_config_folder() -> Result<(), String> {
    let dir = config::Config::config_dir().ok_or("Could not determine config directory")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    open::that(&dir).map_err(|e| format!("Failed to open config folder: {}", e))
}

#[tauri::command]
async fn update_config(
    state: tauri::State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut state_config = state.config.lock().await;
    state_config.app_config = config;
    state_config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn clone_repo(
    state: tauri::State<'_, AppState>,
    url: String,
    path: String,
) -> Result<Repository, String> {
    let (default_branch, token) = {
        let config = state.config.lock().await;
        (
            config.app_config.default_branch.clone(),
            config.github_auth.token.clone(),
        )
    };
    let mut repo = git::clone_repo(&url, &path, Some(&default_branch), token.as_deref()).await?;

    // Auto-detect default branch from the cloned repo
    let detected_branch = git::detect_default_branch(std::path::Path::new(&path)).await;
    if repo.default_branch.is_none() {
        repo.default_branch = detected_branch;
    }

    let mut config = state.config.lock().await;
    config.repos.push(repo.clone());
    config.save().map_err(|e| e.to_string())?;
    Ok(repo)
}

#[tauri::command]
async fn check_git_installed() -> Result<bool, String> {
    git::check_git_installed().await
}

#[tauri::command]
async fn get_github_auth(state: tauri::State<'_, AppState>) -> Result<models::GitHubAuth, String> {
    let config = state.config.lock().await;
    Ok(config.github_auth.clone())
}

#[tauri::command]
async fn get_github_token_scopes(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.lock().await;
    let token = match &config.github_auth.token {
        Some(t) => t.clone(),
        None => return Ok(vec![]),
    };

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .bearer_auth(&token)
        .header("User-Agent", "polyrepo-git-ui")
        .send()
        .await
        .map_err(|e| format!("Failed to check scopes: {}", e))?;

    let parts: Vec<_> = response
        .headers()
        .get_all("x-oauth-scopes")
        .iter()
        .flat_map(|h| h.to_str().unwrap_or("").split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(parts)
}

#[tauri::command]
async fn set_github_pat(
    state: tauri::State<'_, AppState>,
    token: String,
    user: Option<String>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.github_auth = models::GitHubAuth {
        method: Some("pat".to_string()),
        token: Some(token),
        user,
    };
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

fn load_oauth_credentials() -> Result<(String, String), String> {
    // 1. Environment variables (highest priority, for dev override)
    let env_id = std::env::var("OAUTH_CLIENT_ID")
        .or_else(|_| std::env::var("GITHUB_OAUTH_CLIENT_ID"));
    let env_secret = std::env::var("OAUTH_CLIENT_SECRET")
        .or_else(|_| std::env::var("GITHUB_OAUTH_CLIENT_SECRET"));
    if let (Ok(id), Ok(secret)) = (env_id, env_secret) {
        if !id.is_empty() && !secret.is_empty() {
            return Ok((id, secret));
        }
    }

    // 2. .env next to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let path = exe_dir.join(".env");
            if let Ok((id, secret)) = read_env_file(&path) {
                return Ok((id, secret));
            }
        }
    }

    // 3. .env in project root (dev mode: CARGO_MANIFEST_DIR is src-tauri/)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(".env");
        if let Ok((id, secret)) = read_env_file(&path) {
            return Ok((id, secret));
        }
    }

    // 4. Credentials embedded at compile time (set via env vars during CI build)
    let id = option_env!("OAUTH_CLIENT_ID")
        .or_else(|| option_env!("GITHUB_OAUTH_CLIENT_ID"))
        .unwrap_or("");
    let secret = option_env!("OAUTH_CLIENT_SECRET")
        .or_else(|| option_env!("GITHUB_OAUTH_CLIENT_SECRET"))
        .unwrap_or("");
    if !id.is_empty() && !secret.is_empty() {
        return Ok((id.to_string(), secret.to_string()));
    }

    Err(
        "OAuth not configured. Create a .env file in the project root with:\n\
         OAUTH_CLIENT_ID=your_id\n\
         OAUTH_CLIENT_SECRET=your_secret\n\
         Or set those as environment variables."
            .to_string(),
    )
}

fn read_env_file(path: &std::path::Path) -> Result<(String, String), String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut id = String::new();
    let mut secret = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            match key {
                "OAUTH_CLIENT_ID" | "GITHUB_OAUTH_CLIENT_ID" => id = value.to_string(),
                "OAUTH_CLIENT_SECRET" | "GITHUB_OAUTH_CLIENT_SECRET" => secret = value.to_string(),
                _ => {}
            }
        }
    }
    if id.is_empty() || secret.is_empty() {
        return Err(format!("Missing OAUTH_CLIENT_ID or OAUTH_CLIENT_SECRET in {}", path.display()));
    }
    Ok((id, secret))
}

#[tauri::command]
async fn start_github_oauth(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Load credentials: env vars > oauth.toml next to executable > oauth.toml in project root
    let (client_id, client_secret) = load_oauth_credentials()?;

    let listener = TcpListener::bind("127.0.0.1:14201")
        .map_err(|e| format!("Failed to bind local server: {}", e))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;

    let state_code = uuid::Uuid::new_v4().to_string();
    let redirect_uri = "http://127.0.0.1:14201/callback".to_string();
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&scope=repo,read:org&state={}&redirect_uri={}",
        client_id, state_code, redirect_uri
    );
    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    let code = tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > std::time::Duration::from_secs(120) {
                return Err("OAuth timed out after 2 minutes".to_string());
            }
            if let Ok((stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let mut reader = std::io::BufReader::new(&stream);
                use std::io::Read;
                let n = reader.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                if let Some(query_start) = request.find("GET /callback?") {
                    let query_part = &request[query_start + 14..];
                    if let Some(query_end) = query_part.find(' ') {
                        let query = &query_part[..query_end];
                        let mut found_code = None;
                        let mut found_state = None;
                        for param in query.split('&') {
                            let mut parts = param.splitn(2, '=');
                            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                                if key == "code" {
                                    found_code = Some(value.to_string());
                                }
                                if key == "state" {
                                    found_state = Some(value.to_string());
                                }
                            }
                        }
                        if let (Some(code), Some(state_val)) = (found_code, found_state) {
                            if state_val == state_code {
                                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authenticated!</h1><p>You can close this window.</p></body></html>";
                                let _ = std::io::Write::write_all(&mut &stream, response.as_bytes());
                                return Ok(code);
                            }
                        }
                    }
                    let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Invalid state parameter</h1></body></html>";
                    let _ = std::io::Write::write_all(&mut &stream, response.as_bytes());
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })
    .await
    .map_err(|e| format!("OAuth server error: {}", e))??;

    let client = reqwest::Client::new();
    let token_response: serde_json::Value = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_response["access_token"]
        .as_str()
        .ok_or_else(|| format!("No access_token in response: {:?}", token_response))?
        .to_string();

    let user_response: serde_json::Value = client
        .get("https://api.github.com/user")
        .bearer_auth(&access_token)
        .header("User-Agent", "polyrepo-git-ui")
        .send()
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    let username = user_response["login"].as_str().unwrap_or("unknown").to_string();

    let mut config = state.config.lock().await;
    config.github_auth = models::GitHubAuth {
        method: Some("oauth".to_string()),
        token: Some(access_token),
        user: Some(username),
    };
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn disconnect_github(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.github_auth = models::GitHubAuth::default();
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn scan_directory_for_repos(path: String) -> Result<Vec<String>, String> {
    git::scan_directory_for_repos(&path).await
}

struct AppState {
    config: tokio::sync::Mutex<config::Config>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config = config::Config::load().unwrap_or_default();
            app.manage(AppState {
                config: tokio::sync::Mutex::new(config),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_repos,
            add_repo,
            remove_repo,
            refresh_repo,
            refresh_all_repos,
            fetch_repo,
            pull_repo,
            merge_repo,
            merge_repo_with_target,
            abort_merge,
            get_git_log,
            get_groups,
            create_group,
            update_group,
            delete_group,
            move_repo_to_group,
            reorder_groups,
            reorder_repos,
            update_repo_default_branch,
            get_all_prs,
            sync_pr,
            get_config,
            update_config,
            get_config_path,
            open_config_folder,
            clone_repo,
            check_git_installed,
            get_github_auth,
            get_github_token_scopes,
            set_github_pat,
            start_github_oauth,
            disconnect_github,
            scan_directory_for_repos,
            get_git_status,
            get_file_diff,
            stage_file,
            unstage_file,
            stage_all,
            commit,
            push,
            switch_branch,
            discard_file,
            stash,
            stash_pop,
            resolve_conflict,
            continue_merge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn get_git_status(state: tauri::State<'_, AppState>, id: String) -> Result<git::GitStatus, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::get_git_status(&repo_path).await
}

#[tauri::command]
async fn get_file_diff(state: tauri::State<'_, AppState>, id: String, file: String, staged: bool) -> Result<String, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::get_file_diff(&repo_path, &file, staged).await
}

#[tauri::command]
async fn stage_file(state: tauri::State<'_, AppState>, id: String, file: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::stage_file(&repo_path, &file).await
}

#[tauri::command]
async fn unstage_file(state: tauri::State<'_, AppState>, id: String, file: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::unstage_file(&repo_path, &file).await
}

#[tauri::command]
async fn stage_all(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::stage_all(&repo_path).await
}

#[tauri::command]
async fn commit(state: tauri::State<'_, AppState>, id: String, message: String) -> Result<git::CommitResult, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::commit(&repo_path, &message).await
}

#[tauri::command]
async fn push(state: tauri::State<'_, AppState>, id: String, force: bool) -> Result<String, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::push(&repo_path, force).await
}

#[tauri::command]
async fn switch_branch(state: tauri::State<'_, AppState>, id: String, branch: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::switch_branch(&repo_path, &branch).await
}

#[tauri::command]
async fn discard_file(state: tauri::State<'_, AppState>, id: String, file_path: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::discard_file(&repo_path, &file_path).await
}

#[tauri::command]
async fn stash(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::stash(&repo_path).await
}

#[tauri::command]
async fn stash_pop(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::stash_pop(&repo_path).await
}

#[tauri::command]
async fn resolve_conflict(state: tauri::State<'_, AppState>, id: String, file: String, resolution: String) -> Result<(), String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::resolve_conflict(&repo_path, &file, &resolution).await
}

#[tauri::command]
async fn continue_merge(state: tauri::State<'_, AppState>, id: String) -> Result<git::CommitResult, String> {
    let repo_path = {
        let config = state.config.lock().await;
        config.repos.iter().find(|r| r.id == id).map(|r| r.path.clone()).ok_or("Repo not found")?
    };
    git::continue_merge(&repo_path).await
}
