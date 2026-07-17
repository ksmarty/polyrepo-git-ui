# AGENTS.md — Polyrepo Git UI

Cross-platform desktop app for managing multiple git repos. Tauri v2 + Rust + Svelte 5 + Vite 8.

## Commands

| Command | What it does |
|---|---|
| `pnpm install` | Install dependencies |
| `pnpm tauri dev` | Run in dev mode |
| `pnpm tauri build` | Build for production |
| `pnpm verify` | Run svelte-check + cargo check + cargo test |
| `pnpm check:svelte` | Frontend type/lint check only |
| `pnpm check:rust` | Rust check + tests only |
| `pnpm update:check` | Show outdated dependencies |
| `pnpm update:apply` | Interactive update all deps |

**Pre-commit hook**: `.git/hooks/pre-commit` runs `pnpm verify` automatically.

## Architecture

```
src/                          # Svelte 5 frontend
  App.svelte                  # Root: tabs (Repos, PRs, Settings), theme, repo selection
  lib/
    components/
      Sidebar.svelte          # Repo tree, drag-and-drop, search, import/folder buttons
      PRList.svelte           # Filterable PR list
      PRCard.svelte           # PR card with sync dropdown
      Settings.svelte         # Sidebar nav (General, Repos, Folders, GitHub), auto-save
      AuthSetup.svelte        # OAuth + PAT setup
      ImportModal.svelte      # Bulk import modal, auto-scan
    types.ts                  # TypeScript interfaces (Repository, RepoGroup, PullRequest, etc.)

src-tauri/                    # Rust backend
  src/
    lib.rs                    # 25+ Tauri commands, AppState
    config.rs                 # AppConfig, Config, TOML persistence, 5 unit tests
    git.rs                    # Git CLI operations, scan_directory_for_repos, 12 unit tests
    github.rs                 # GraphQL PR queries, REST API, OAuth with local TCP callback
    models.rs                 # Repository, RepoGroup, PullRequest, GitHubAuth, 8 unit tests
  capabilities/
    default.json              # Tauri v2 plugin permissions (dialog, shell)
  Cargo.toml                  # Rust deps including tempfile for tests
```

## Key Decisions

- **Rust + Tauri v2**: ~5-10MB binaries, native OS webview (WKWebView macOS, WebView2 Windows)
- **Svelte 5 (not SvelteKit)**: smallest bundle, runes mode (`$state`, `$props`), built-in TS
- **pnpm**: `packageManager: "pnpm@11.8.0"` in package.json, CI uses `pnpm/action-setup@v4`
- **Git CLI over libgit2**: simpler, always up-to-date
- **OAuth via local TCP server**: port 14201, 2-min timeout
- **CSS themes from happyhues.co**: 8 themes mapped to semantic CSS variables via `data-theme` attribute
- **TypeScript ~5.8.3**: pinned because TS 7 is incompatible with svelte-check 4.x
- **Vite 8 + vite-plugin-svelte 7**: uses `svelte()` plugin, NOT `sveltekit()`
- **svelte.config.js**: just `export default {}` — Svelte 5 has built-in TS, no svelte-preprocess

## Conventions

### Frontend (Svelte 5)
- Use `$state()` for reactive state, `$props()` for component props
- Use `createEventDispatcher` for parent communication (NOT `$bindable`)
- Event handlers: `onclick`, `onchange`, `ondragstart` (lowercase, not `on:click`)
- Use `scheduleSave()` with 400ms debounce for auto-save (no Save button)
- Import icons from `@lucide/svelte` (e.g., `RefreshCw`, `ExternalLink`, `FolderGit2`)
- Import Tauri APIs lazily: `const { invoke } = await import('@tauri-apps/api/core')`
- Dialog: `const { open } = await import('@tauri-apps/plugin-dialog')`
- Shell: `const { open } = await import('@tauri-apps/plugin-shell')`

### Rust
- Tauri commands: `#[tauri::command]` with snake_case params → frontend camelCase
- Lock mutex properly: never hold `state.config.lock().await` across `.await` points
- Extract repo path from config first, drop lock, then do git operations
- `Config` wraps `AppConfig` + `repos` + `groups` + `github_auth`
- All commands registered in `lib.rs` `invoke_handler`
- Config saved to OS-standard path via `dirs` crate

### Theming
- `data-theme` attribute on `<html>` element
- CSS variables: `--bg-primary`, `--bg-secondary`, `--bg-tertiary`, `--text-primary`, `--text-secondary`, `--accent`, `--success`, `--warning`, `--danger`, `--info`, `--border`
- 8 themes: midnight (default dark), light (default light), dark, forest, ocean, solarized, catppuccin, system
- Themes sorted alphabetically in Settings

### Drag and Drop
- Single event delegation: all `ondragover`/`ondrop` on `.repo-tree` container
- Use `data-*` attributes (`data-repo-id`, `data-group-id`, `data-ungrouped`) to identify targets
- Use `e.target.closest('[data-repo-id]')` to find nearest drop target
- Determine position (above/below/inside) from cursor Y position relative to element
- Visual feedback: `.drag-over-above` (blue border top), `.drag-over-below` (blue border bottom), `.drag-over` (dashed outline on groups)

## Data Model

### Repository
```typescript
interface Repository {
  id: string;              // UUID
  name: string;            // directory name
  path: string;            // absolute path
  remote_url: string | null;
  github_owner: string | null;
  github_repo: string | null;
  default_branch: string | null;  // per-repo override
  group_id: string | null;        // folder assignment
  order: number;                  // sort order within group
  local_branches: string[];
  current_branch: string;
  sync_status: SyncStatus | null;
}
```

### RepoGroup
```typescript
interface RepoGroup {
  id: string;              // UUID
  name: string;
  parent_id: string | null;  // nested groups
  order: number;
}
```

## Tauri Commands (25+)

| Command | Description |
|---|---|
| `get_repos` | List all repos |
| `add_repo` | Add repo by path, optional group_id |
| `remove_repo` | Remove repo by id |
| `refresh_repo` | Re-read git status (preserves id/group_id/default_branch) |
| `refresh_all_repos` | Refresh all repos |
| `fetch_repo` | Run `git fetch --all` |
| `pull_repo` | Run `git pull` (optional rebase) |
| `get_groups` | List all groups |
| `create_group` | Create folder, optional parent_id |
| `update_group` | Rename folder |
| `delete_group` | Delete folder |
| `move_repo_to_group` | Move repo to folder (or null for ungrouped) |
| `reorder_groups` | Update group order from ID list |
| `reorder_repos` | Update repo order from ID list |
| `update_repo_default_branch` | Set per-repo default branch override |
| `get_all_prs` | Fetch PRs from GitHub for all repos |
| `sync_pr` | Sync a PR branch (merge or rebase) |
| `get_config` | Get AppConfig |
| `update_config` | Save AppConfig |
| `clone_repo` | Clone a git repo |
| `check_git_installed` | Verify git is available |
| `get_github_auth` | Get auth status |
| `set_github_pat` | Set personal access token |
| `start_github_oauth` | Start OAuth flow (local TCP server) |
| `disconnect_github` | Clear auth |
| `scan_directory_for_repos` | Find git repos in a directory |

## Sync Status Logic

`get_behind_ahead` in `git.rs`:
1. Check if branch has upstream tracking via `git rev-parse --abbrev-ref branch@{upstream}`
2. If upstream exists, compare against it with `git rev-list --left-right --count upstream...branch`
3. If no upstream or comparison fails, try `origin/main`
4. If that fails, try `origin/master`
5. Only returns `(0, 0)` if all candidates fail

`is_dirty`: `git status --porcelain` — any output means dirty.

## Config File (TOML)

```toml
[app_config]
default_branch = "main"
theme = "midnight"
auto_fetch_on_open = true
fetch_interval_seconds = 300
sidebar_width = 300

[[repos]]
id = "uuid"
name = "my-repo"
path = "/Users/..."
group_id = "uuid-or-null"
order = 0
default_branch = "develop"  # per-repo override

[[groups]]
id = "uuid"
name = "Frontend"
parent_id = null
order = 0

[github_auth]
method = "oauth"  # or "pat"
token = "ghp_..."
user = "username"
```

## CI/CD

- `.github/workflows/build.yml`: Build matrix (macOS, Windows), triggered on push/PR
- `.github/workflows/release.yml`: Build + create GitHub release on tag push

## Workflow Notes

- Always commit changes once they're complete — don't leave uncommitted work after finishing a task.

## Pitfalls

- `vite.config.ts` must use `import { svelte } from '@sveltejs/vite-plugin-svelte'` — NOT `sveltekit()`
- `svelte.config.js` must be just `export default {}` — no svelte-preprocess
- `start_github_oauth` requires env vars: `GITHUB_OAUTH_CLIENT_ID` and `GITHUB_OAUTH_CLIENT_SECRET`
- `frontendDist` in tauri.conf.json points to `../dist` — `dist/` dir must exist for `cargo check`
- Dialog plugin needs capability in `src-tauri/capabilities/default.json`
- `refresh_repo` must preserve existing `id`, `group_id`, `default_branch` when updating
- `AppConfig` derives `PartialEq` (needed for test assertions)
- `update_config` Rust param is `config` (not `config_update`) to match frontend `{ config }`
- Svelte 5 reactivity: create new `Set()` when toggling (don't mutate in place)
- Svelte 5 reactivity: create new array with `.map()` (don't mutate)
- Checkbox reactivity: must create `new Set()` when toggling, not mutate in place
- `pnpm verify` runs svelte-check + cargo check + cargo test — enforced by pre-commit hook
- TypeScript 7 is incompatible with svelte-check 4.x — pinned to ~5.8.3

## Git History

Recent commits (newest first):
- `fix: sync status, drag-and-drop, open-in-github button`
- `fix: repo selection, folder toggle, drag-and-drop sidebar`
- `fix: folder dropdowns, ungrouped repos, refresh buttons, per-repo branch`
- `chore: update vite 8.1, vite-plugin-svelte 7.2, svelte-check; add update scripts`
- `feat: import supports repos + folders, auto-scan on browse, auto-save settings`
- `chore: icons, theme rename, a11y fixes, modern UI polish`
- `feat: add 8 happyhues.co themes, auto-save settings, browse button`
- `feat: import repos via file browser or path entry, with bulk import modal`
- `feat: implement core app structure with sidebar, PR list, and settings`
- `chore: init project`
