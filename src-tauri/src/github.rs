use reqwest::Client;
use serde_json::Value;

use crate::models::PullRequest;

const GITHUB_API_BASE: &str = "https://api.github.com";

pub async fn get_prs(token: &str, owner: &str, name: &str, repo_id: &str) -> Result<Vec<PullRequest>, String> {
    let client = Client::new();
    let prs_url = format!("{}/repos/{}/{}/pulls?state=open&per_page=50", GITHUB_API_BASE, owner, name);

    let response = client
        .get(&prs_url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().as_u16() == 404 {
        return Ok(vec![]);
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("GitHub API error {}: {}", status, &body[..body.len().min(200)]));
    }

    let prs_json: Vec<Value> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse PRs: {}", e))?;

    let mut result = Vec::new();
    for pr in &prs_json {
        let draft = pr["draft"].as_bool().unwrap_or(false);
        if draft {
            continue;
        }

        let number = match pr["number"].as_u64() {
            Some(n) => n as u32,
            None => continue,
        };
        let title = pr["title"].as_str().unwrap_or("").to_string();
        let head_ref = pr["head"]["ref"].as_str().unwrap_or("").to_string();
        let base_ref = pr["base"]["ref"].as_str().unwrap_or("").to_string();
        let mergeable = pr["mergeable"].as_bool();
        let statuses_url = pr["statuses_url"].as_str().unwrap_or("").to_string();

        let checks_status = if statuses_url.is_empty() {
            "pending".to_string()
        } else {
            get_combined_status(&client, token, &statuses_url).await
        };

        result.push(PullRequest {
            id: format!("{}/{}/{}", owner, name, number),
            number,
            title,
            head_ref,
            base_ref,
            repo_id: repo_id.to_string(),
            repo_name: name.to_string(),
            state: "open".to_string(),
            mergeable: mergeable.unwrap_or(false),
            behind_count: 0,
            checks_status,
            review_decision: None,
        });
    }

    Ok(result)
}

async fn get_combined_status(client: &Client, token: &str, statuses_url: &str) -> String {
    let response = client
        .get(statuses_url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await;

    let statuses: Vec<Value> = match response {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => return "pending".to_string(),
    };

    if statuses.is_empty() {
        return "pending".to_string();
    }

    let mut states: Vec<String> = statuses
        .iter()
        .filter_map(|s| s["state"].as_str().map(|s| s.to_string()))
        .collect();

    states.sort();
    states.dedup();

    if states.contains(&"failure".to_string()) || states.contains(&"error".to_string()) {
        "failure".to_string()
    } else if states.contains(&"success".to_string()) {
        "success".to_string()
    } else {
        "pending".to_string()
    }
}

pub async fn sync_pr(token: &str, pr_id: &str, use_rebase: bool) -> Result<(), String> {
    let parts: Vec<&str> = pr_id.split('/').collect();
    if parts.len() != 3 {
        return Err("Invalid PR ID format".to_string());
    }

    let owner = parts[0];
    let repo = parts[1];
    let _number: u32 = parts[2]
        .parse()
        .map_err(|_| "Invalid PR number".to_string())?;

    let client = Client::new();

    let pr_url = format!("{}/repos/{}/{}/pulls", GITHUB_API_BASE, owner, repo);
    let _pr: Value = client
        .get(&pr_url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .send()
        .await
        .map_err(|e| format!("Failed to get PR: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse PR: {}", e))?;

    let _ = use_rebase;

    Ok(())
}
