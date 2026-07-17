# Polyrepo Git UI

A cross-platform desktop application for managing multiple git repositories, built with **Rust + Tauri v2** (backend) and **Svelte** (frontend). The app focuses on keeping feature branches in sync with the default branch and providing at-a-glance PR status across all repos.

## Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Desktop framework | Tauri v2.11.5 | ~5-10MB binaries, low memory, native OS webview |
| Backend | Rust | System-level git operations, small binaries |
| Frontend | Svelte (via Vite) | Smallest bundle size, excellent performance |
| Git operations | Shell out to `git` CLI | Simpler, always up-to-date with git features |
| GitHub integration | REST API v3 + GraphQL v4 | REST for simple ops, GraphQL for batch queries (PR status) |
| Auth | OAuth (SSO) + PAT (fine-grained) | Flexibility for enterprise and personal use |
| Config storage | OS standard (`~/.config/polyrepo-git-ui` on Linux, `~/Library/Application Support/polyrepo-git-ui` on macOS, `%APPDATA%/polyrepo-git-ui` on Windows) |

## Project Structure

```
polyrepo-git-ui/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── git.rs            # git operations (clone, pull, fetch, status)
│   │   │   ├── repos.rs          # repo management (add, remove, list, groups)
│   │   │   ├── github.rs         # GitHub API operations
│   │   │   └── pr.rs             # PR operations (list, create, update, merge)
│   │   ├── git/                  # Git module
│   │   │   ├── mod.rs
│   │   │   ├── commands.rs       # Low-level git command execution
│   │   │   ├── status.rs         # Branch status, sync status
│   │   │   └── remote.rs         # Remote URL parsing, GitHub detection
│   │   ├── github/               # GitHub API module
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs           # OAuth flow, PAT management
│   │   │   ├── client.rs         # HTTP client for GitHub API
│   │   │   ├── graphql.rs        # GraphQL queries (PR status, branch comparison)
│   │   │   └── rest.rs           # REST endpoints (clone URL, repo info)
│   │   ├── config/               # Configuration management
│   │   │   ├── mod.rs
│   │   │   ├── app_config.rs     # App-wide settings (default branch, theme)
│   │   │   ├── repo_config.rs    # Per-repo settings (group, default branch override)
│   │   │   └── auth_config.rs    # Stored tokens (encrypted)
│   │   └── models/               # Data models
│   │       ├── mod.rs
│   │       ├── repo.rs           # Repository, RepoGroup
│   │       ├── pr.rs             # PullRequest, PRStatus
│   │       └── auth.rs           # AuthToken, OAuthCredentials
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── Sidebar.svelte    # Repo list with folder groups
│   │   │   ├── RepoCard.svelte   # Individual repo display
│   │   │   ├── PRList.svelte     # PR tab view
│   │   │   ├── PRCard.svelte     # Individual PR with sync status
│   │   │   ├── SyncButton.svelte # One-click merge/rebase
│   │   │   ├── Settings.svelte   # Settings panel
│   │   │   ├── AuthSetup.svelte  # GitHub auth configuration
│   │   │   └── FolderTree.svelte # Virtual folder tree view
│   │   ├── stores/
│   │   │   ├── repos.ts          # Repo state management
│   │   │   ├── prs.ts            # PR state management
│   │   │   ├── auth.ts           # Auth state
│   │   │   └── config.ts         # Config state
│   │   └── utils/
│   │       ├── tauri.ts          # Tauri invoke wrappers
│   │       └── format.ts         # Formatting helpers
│   ├── routes/                   # SvelteKit-style routing (if using SvelteKit)
│   ├── App.svelte
│   └── main.ts
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── .github/
    └── workflows/
        ├── build.yml             # Build for mac & windows on push/tag
        └── release.yml           # Create GitHub releases
```

## Feature Breakdown

### 1. Repository Management

- **Add repos**: User selects directories containing git repos (scans for `.git` folders)
- **Virtual folders**: Organizational groups (not filesystem-based). Stored in config.
- **Sidebar**: Tree view showing repos grouped by folder. Shows sync status indicator per repo.
- **Repo detection**: Auto-detect if repo is a GitHub repo from remote URL.

**Data Model**:

```rust
struct Repository {
    id: Uuid,
    name: String,
    path: PathBuf,
    remote_url: Option<String>,      // parsed from git remote
    github_owner: Option<String>,     // extracted from GitHub URL
    github_repo: Option<String>,     // extracted from GitHub URL
    default_branch: Option<String>,  // per-repo override
    group_id: Option<Uuid>,          // virtual folder
    local_branches: Vec<String>,
    current_branch: String,
}

struct RepoGroup {
    id: Uuid,
    name: String,
    parent_id: Option<Uuid>,  // for nested folders
    order: u32,
}
```

### 2. Git Operations

All operations shell out to `git` CLI:

- **Status**: `git status --porcelain -b` for branch info and dirty state
- **Fetch**: `git fetch --all` across repos
- **Pull**: `git pull` (respecting rebase config)
- **Branch comparison**: `git log --oneline HEAD..origin/{branch}` to count commits behind/ahead
- **Clone**: `git clone {url} {path}` for new repos
- **Stash**: `git stash` / `git stash pop` for dirty working trees

**Sync status detection**:

```rust
struct SyncStatus {
    behind: u32,      // commits behind default branch
    ahead: u32,       // commits ahead of default branch
    is_dirty: bool,   // uncommitted changes
    up_to_date: bool, // behind == 0
}
```

### 3. PR Tab

- **List all PRs** across all repos (using GitHub GraphQL API for efficiency)
- **At-a-glance sync status**: Show if PR branch is behind default branch
- **Sync status indicator**:
  - Green: up to date
  - Yellow: behind by N commits
  - Red: conflicts detected
  - Gray: checks failing
- **One-click update**: Button that offers merge or rebase choice
- **PR management** (when GitHub auth is configured):
  - View PR details (description, checks, reviews)
  - Approve/request changes
  - Add comments
  - Merge PR

**GraphQL query for batch PR status**:

```graphql
query RepoPRs($owner: String!, $name: String!) {
  repository(owner: $owner, name: $name) {
    pullRequests(first: 50, states: OPEN) {
      nodes {
        number
        title
        headRefName
        baseRefName
        commits {
          nodes {
            commit {
              statusCheckRollup {
                state
              }
            }
          }
        }
        reviewDecision
        mergeable
      }
    }
  }
}
```

### 4. GitHub Integration

**Authentication options**:

1. **OAuth (SSO)**: Opens system browser for GitHub OAuth flow. Stores refresh token securely in OS keychain.
2. **PAT (Fine-grained)**: User provides token. App shows exact permissions needed:
   - `Contents`: Read (for clone/fetch)
   - `Pull requests`: Read & Write (for PR management)
   - `Metadata`: Read (always required)
   - `Commit statuses`: Read (for check status)

**Token storage**: Use OS keychain via `keyring` crate.

### 5. Configuration

**App config** (`~/.config/polyrepo-git-ui/config.toml`):

```toml
[app]
default_branch = "main"  # global fallback
theme = "system"
auto_fetch_on_open = true
fetch_interval_seconds = 300

[github]
auth_method = "oauth"  # or "pat"

[ui]
sidebar_width = 300
show_unified_pr_view = true
```

**Repo config** (stored in app config, keyed by repo path):

```toml
[[repos]]
path = "/path/to/repo"
default_branch = "develop"  # override
group_id = "uuid-here"
```

### 6. Settings Panel

- Configure global default branch
- Add/remove repos
- Manage virtual folders
- GitHub auth setup (OAuth or PAT)
- Show required PAT permissions
- Theme selection
- Auto-fetch interval

## UI Layout

```
┌─────────────────────────────────────────────────────────┐
│  Polyrepo Git UI                         [Settings] [⚙] │
├──────────┬──────────────────────────────────────────────┤
│          │  ┌─────────────────────────────────────────┐ │
│ SIDEBAR  │  │  [Repos] [Pull Requests] [Settings]    │ │
│          │  ├─────────────────────────────────────────┤ │
│ ▼ Team A │  │                                         │ │
│   repo1  │  │  (Content area based on selected tab)   │ │
│   repo2  │  │                                         │ │
│ ▼ Team B │  │  Repos tab: Grid/list of repos with     │ │
│   repo3  │  │  sync status indicators                 │ │
│   repo4  │  │                                         │ │
│          │  │  PRs tab: Unified PR list across repos   │ │
│          │  │  with sync status and one-click update   │ │
│          │  │                                         │ │
│          │  └─────────────────────────────────────────┘ │
└──────────┴──────────────────────────────────────────────┘
```

## GitHub Actions Workflow

**Build workflow** (`.github/workflows/build.yml`):

- Trigger: Push to main, tags `v*`
- Matrix: macOS (ARM64, x64), Windows (x64)
- Steps:
  1. Checkout
  2. Setup Rust toolchain
  3. Setup Node.js
  4. `npm install`
  5. `cargo tauri build`
  6. Upload artifacts

**Release workflow** (`.github/workflows/release.yml`):

- Trigger: Tag `v*`
- Steps:
  1. Build all platforms
  2. Create GitHub release with binaries
  3. Auto-generate changelog

## Implementation Phases

### Phase 1: Core (Week 1-2)

- Project scaffolding (Tauri + Svelte)
- Basic repo management (add/remove repos)
- Virtual folder system
- Sidebar with repo list
- Git status detection

### Phase 2: Git Operations (Week 3)

- Fetch/pull operations
- Branch comparison (ahead/behind)
- Sync status indicators
- Clone new repos

### Phase 3: GitHub Integration (Week 4)

- OAuth flow
- PAT setup with permissions guide
- PR listing via GraphQL
- PR status display

### Phase 4: PR Management (Week 5)

- One-click merge/rebase
- PR details view
- Approve/comment
- Conflict detection

### Phase 5: Polish (Week 6)

- Settings panel
- Auto-fetch
- Error handling
- Cross-platform testing
- GitHub Actions workflows

## Key Dependencies

**Rust (Cargo.toml)**:

```toml
[dependencies]
tauri = { version = "2.11", features = ["shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
keyring = "3"              # OS keychain
uuid = { version = "1", features = ["v4"] }
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"                 # OS directories
open = "5"                 # Open URLs in browser
```

**Frontend (package.json)**:

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0"
  },
  "devDependencies": {
    "svelte": "^5.0.0",
    "@sveltejs/kit": "^2.0.0",
    "vite": "^6.0.0",
    "@sveltejs/vite-plugin-svelte": "^5.0.0"
  }
}
```

## PAT Permissions Guide (In-App)

The app should display a clear permissions guide when setting up a PAT:

```
To use Polyrepo Git UI with a Personal Access Token, create a
fine-grained token at: https://github.com/settings/tokens?type=beta

Required Permissions:
┌─────────────────────┬─────────────────────┐
│ Permission          │ Access Level        │
├─────────────────────┼─────────────────────┤
│ Contents            │ Read and Write      │
│ Pull requests       │ Read and Write      │
│ Metadata            │ Read-only           │
│ Commit statuses     │ Read-only           │
└─────────────────────┴─────────────────────┘

Repository Access: Select "Only select repositories" and
choose the repos you want to manage, or "All repositories"
for full access.
```
