use reqwest::Client;
use serde::Deserialize;

use crate::models::PullRequest;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_GRAPHQL_URL: &str = "https://api.github.com/graphql";

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct RepositoriesData {
    repository: RepositoryData,
}

#[derive(Debug, Deserialize)]
struct RepositoryData {
    pull_requests: PullRequestsConnection,
}

#[derive(Debug, Deserialize)]
struct PullRequestsConnection {
    nodes: Vec<GitHubPR>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GitHubPR {
    number: u32,
    title: String,
    headRefName: String,
    baseRefName: String,
    mergeable: String,
    reviewDecision: Option<String>,
    commits: CommitsConnection,
}

#[derive(Debug, Deserialize)]
struct CommitsConnection {
    nodes: Vec<CommitNode>,
}

#[derive(Debug, Deserialize)]
struct CommitNode {
    commit: CommitData,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct CommitData {
    statusCheckRollup: Option<StatusCheckRollup>,
}

#[derive(Debug, Deserialize)]
struct StatusCheckRollup {
    state: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RestPR {
    id: String,
    number: u32,
    title: String,
    head: RestBranch,
    base: RestBranch,
    state: String,
    mergeable: Option<String>,
    draft: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RestBranch {
    ref_name: String,
}

pub async fn get_prs(token: &str, owner: &str, name: &str, repo_id: &str) -> Result<Vec<PullRequest>, String> {
    let query = r#"
        query RepoPRs($owner: String!, $name: String!) {
            repository(owner: $owner, name: $name) {
                pullRequests(first: 50, states: OPEN) {
                    nodes {
                        number
                        title
                        headRefName
                        baseRefName
                        mergeable
                        reviewDecision
                        commits(last: 1) {
                            nodes {
                                commit {
                                    statusCheckRollup {
                                        state
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let client = Client::new();
    let body = serde_json::json!({
        "query": query,
        "variables": {
            "owner": owner,
            "name": name
        }
    });

    let response = client
        .post(GITHUB_GRAPHQL_URL)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    let graphql_response: GraphQLResponse<RepositoriesData> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(errors) = graphql_response.errors {
        if !errors.is_empty() {
            return Err(format!("GraphQL error: {}", errors[0].message));
        }
    }

    let data = graphql_response
        .data
        .ok_or("No data in response")?;

    let prs: Vec<PullRequest> = data
        .repository
        .pull_requests
        .nodes
        .into_iter()
        .map(|pr| {
            let checks_status = pr
                .commits
                .nodes
                .last()
                .and_then(|node| node.commit.statusCheckRollup.as_ref())
                .map(|rollup| rollup.state.to_lowercase())
                .unwrap_or_else(|| "pending".to_string());

            PullRequest {
                id: format!("{}/{}/{}", owner, name, pr.number),
                number: pr.number,
                title: pr.title,
                head_ref: pr.headRefName,
                base_ref: pr.baseRefName,
                repo_id: repo_id.to_string(),
                repo_name: name.to_string(),
                state: "open".to_string(),
                mergeable: pr.mergeable == "MERGEABLE",
                behind_count: 0,
                checks_status,
                review_decision: pr.reviewDecision,
            }
        })
        .collect();

    Ok(prs)
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

    // Get the PR details first
    let pr_url = format!("{}/repos/{}/{}/pulls", GITHUB_API_BASE, owner, repo);
    let _pr: RestPR = client
        .get(&pr_url)
        .bearer_auth(token)
        .header("User-Agent", "polyrepo-git-ui")
        .send()
        .await
        .map_err(|e| format!("Failed to get PR: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse PR: {}", e))?;

    // For now, we'll just fetch the repo to update it
    // A full implementation would handle the actual sync operation
    // by pushing the updated branch or creating a merge commit
    let _ = use_rebase;

    Ok(())
}
