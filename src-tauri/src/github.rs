use reqwest::Client;
use serde_json::Value;

use crate::models::PullRequest;

const GITHUB_API_BASE: &str = "https://api.github.com";

pub async fn get_prs(token: &str, owner: &str, name: &str, repo_id: &str, state: &str) -> Result<Vec<PullRequest>, String> {
    let client = Client::new();
    let prs_url = format!("{}/repos/{}/{}/pulls?state={}&per_page=50", GITHUB_API_BASE, owner, name, state);

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

    let required_reviews = get_required_review_count(&client, token, owner, name).await;

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
        let html_url = pr["html_url"].as_str().unwrap_or("").to_string();
        let pr_state = pr["state"].as_str().unwrap_or("open").to_string();

        let checks_status = if statuses_url.is_empty() {
            "pending".to_string()
        } else {
            get_combined_status(&client, token, &statuses_url).await
        };

        let body = pr["body"].as_str().map(|s| s.to_string());
        let author = pr["user"]["login"].as_str().map(|s| s.to_string());
        let requested_reviewers: Vec<String> = pr["requested_reviewers"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r["login"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let approved_reviews = get_approved_review_count(&client, token, owner, name, number).await;

        result.push(PullRequest {
            id: format!("{}/{}/{}", owner, name, number),
            number,
            title,
            head_ref,
            base_ref,
            repo_id: repo_id.to_string(),
            repo_name: name.to_string(),
            state: pr_state,
            mergeable: mergeable.unwrap_or(true),
            behind_count: 0,
            checks_status,
            review_decision: None,
            html_url,
            body,
            author,
            requested_reviewers,
            approved_reviews,
            required_reviews,
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

async fn get_required_review_count(client: &Client, token: &str, owner: &str, name: &str) -> u32 {
    // First, try the rulesets API (used by repos with GitHub Rulesets)
    if let Some(count) = get_required_review_count_from_rulesets(client, token, owner, name).await {
        return count;
    }

    // Fallback: get default branch from repo metadata, then try old branch protection API
    let default_branch = get_default_branch(client, token, owner, name).await;
    let branches_to_try: Vec<&str> = if let Some(ref branch) = default_branch {
        vec![branch.as_str(), "main", "master"]
    } else {
        vec!["main", "master"]
    };

    for branch in branches_to_try {
        let url = format!("{}/repos/{}/{}/branches/{}/protection/required_pull_request_reviews", GITHUB_API_BASE, owner, name, branch);
        let response = client
            .get(&url)
            .bearer_auth(token)
            .header("User-Agent", "polyrepo-git-ui")
            .header("Accept", "application/vnd.github+json")
            .send()
            .await;

        if let Ok(r) = response {
            if r.status().is_success() {
                let data: Value = r.json().await.unwrap_or_default();
                if let Some(count) = data["required_approving_review_count"].as_u64() {
                    return count as u32;
                }
            }
        }
    }

    0
}

async fn get_default_branch(client: &Client, token: &str, owner: &str, name: &str) -> Option<String> {
    let url = format!("{}/repos/{}/{}/branches/main", GITHUB_API_BASE, owner, name);
    let response = client
        .get(&url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;

    if response.status().is_success() {
        return Some("main".to_string());
    }

    let url = format!("{}/repos/{}/{}/branches/master", GITHUB_API_BASE, owner, name);
    let response = client
        .get(&url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;

    if response.status().is_success() {
        return Some("master".to_string());
    }

    None
}

async fn get_required_review_count_from_rulesets(client: &Client, token: &str, owner: &str, name: &str) -> Option<u32> {
    let url = format!("{}/repos/{}/{}/rulesets", GITHUB_API_BASE, owner, name);
    let response = client
        .get(&url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let rulesets: Vec<Value> = response.json().await.unwrap_or_default();

    for ruleset in &rulesets {
        if ruleset["enforcement"].as_str() != Some("active") {
            continue;
        }

        // Check if this ruleset applies to the default branch
        let includes = ruleset["conditions"]["ref_name"]["include"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let is_default_branch_rule = includes.iter().any(|pattern| pattern == "~DEFAULT_BRANCH");
        if !is_default_branch_rule {
            continue;
        }

        // Look for pull_request rule with required_approving_review_count
        if let Some(rules) = ruleset["rules"].as_array() {
            for rule in rules {
                if rule["type"].as_str() == Some("pull_request") {
                    if let Some(count) = rule["parameters"]["required_approving_review_count"].as_u64() {
                        return Some(count as u32);
                    }
                }
            }
        }
    }

    None
}

async fn get_approved_review_count(client: &Client, token: &str, owner: &str, name: &str, pr_number: u32) -> u32 {
    let url = format!("{}/repos/{}/{}/pulls/{}/reviews", GITHUB_API_BASE, owner, name, pr_number);
    let response = client
        .get(&url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await;

    let reviews: Vec<Value> = match response {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => return 0,
    };

    reviews
        .iter()
        .filter(|r| r["state"].as_str() == Some("APPROVED"))
        .count() as u32
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
