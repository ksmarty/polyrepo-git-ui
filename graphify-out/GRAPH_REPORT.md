# Graph Report - polyrepo-git-ui  (2026-07-17)

## Corpus Check
- 31 files · ~36,475 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 412 nodes · 600 edges · 23 communities (20 shown, 3 thin omitted)
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- properties
- definitions
- properties
- lib.rs
- definitions
- package.json
- tauri.conf.json
- tauri.ts
- Polyrepo Git UI
- github.rs
- git.rs
- Config
- compilerOptions
- models.rs
- @tauri-apps/api/core
- graphify.js
- AGENTS.md
- vite-env.d.ts

## God Nodes (most connected - your core abstractions)
1. `AppState` - 23 edges
2. `compilerOptions` - 14 edges
3. `Config` - 13 edges
4. `definitions` - 10 edges
5. `definitions` - 10 edges
6. `add_repo()` - 10 edges
7. `get_sync_status()` - 9 edges
8. `Polyrepo Git UI` - 9 edges
9. `refresh_repo()` - 8 edges
10. `get_remote_url()` - 7 edges

## Surprising Connections (you probably didn't know these)
- `AppState` --references--> `Config`  [EXTRACTED]
  src-tauri/src/lib.rs → src-tauri/src/config.rs

## Import Cycles
- None detected.

## Communities (23 total, 3 thin omitted)

### Community 0 - "properties"
Cohesion: 0.05
Nodes (45): description, properties, required, type, Capability, Identifier, default, description (+37 more)

### Community 1 - "definitions"
Cohesion: 0.05
Nodes (38): anyOf, description, required, type, description, properties, required, type (+30 more)

### Community 2 - "properties"
Cohesion: 0.05
Nodes (39): properties, Identifier, default, description, type, description, oneOf, type (+31 more)

### Community 3 - "lib.rs"
Cohesion: 0.20
Nodes (34): AppConfig, Mutex, add_repo(), AppState, check_git_installed(), clone_repo(), create_group(), delete_group() (+26 more)

### Community 4 - "definitions"
Cohesion: 0.06
Nodes (32): anyOf, description, properties, required, type, definitions, CapabilityRemote, Number (+24 more)

### Community 5 - "package.json"
Cohesion: 0.07
Nodes (28): dependencies, @tauri-apps/api, @tauri-apps/plugin-shell, devDependencies, svelte, svelte-check, @sveltejs/vite-plugin-svelte, @tauri-apps/cli (+20 more)

### Community 6 - "tauri.conf.json"
Cohesion: 0.08
Nodes (25): icons/128x128@2x.png, icons/128x128.png, icons/32x32.png, icons/icon.png, app, security, windows, build (+17 more)

### Community 7 - "tauri.ts"
Cohesion: 0.09
Nodes (6): AppConfig, GitHubAuth, PullRequest, RepoGroup, Repository, SyncStatus

### Community 8 - "Polyrepo Git UI"
Cohesion: 0.10
Nodes (20): 1. Repository Management, 2. Git Operations, 3. PR Tab, 4. GitHub Integration, 5. Configuration, 6. Settings Panel, Feature Breakdown, GitHub Actions Workflow (+12 more)

### Community 9 - "github.rs"
Cohesion: 0.20
Nodes (20): CommitData, CommitNode, CommitsConnection, get_prs(), GitHubPR, GraphQLError, GraphQLResponse, PullRequestsConnection (+12 more)

### Community 10 - "git.rs"
Cohesion: 0.32
Nodes (19): Path, add_repo(), check_git_installed(), clone_repo(), fetch_repo(), get_behind_ahead(), get_current_branch(), get_local_branches() (+11 more)

### Community 11 - "Config"
Cohesion: 0.19
Nodes (12): PathBuf, AppConfig, Config, Default, GitHubAuth, Option, RepoGroup, Repository (+4 more)

### Community 12 - "compilerOptions"
Cohesion: 0.11
Nodes (17): src/**/*.svelte, src/**/*.ts, compilerOptions, allowImportingTsExtensions, isolatedModules, module, moduleDetection, moduleResolution (+9 more)

### Community 13 - "models.rs"
Cohesion: 0.26
Nodes (10): GitHubAuth, PullRequest, RepoGroup, Repository, Default, Option, Self, String (+2 more)

### Community 14 - "@tauri-apps/api/core"
Cohesion: 0.32
Nodes (7): @tauri-apps/api/core, @tauri-apps/plugin-dialog, ./AuthSetup.svelte, ./ImportModal.svelte, ./PRCard.svelte, app, ../types

## Knowledge Gaps
- **162 isolated node(s):** `name`, `private`, `version`, `type`, `packageManager` (+157 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `properties` connect `properties` to `definitions`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **Why does `definitions` connect `definitions` to `properties`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _162 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `properties` be split into smaller, more focused modules?**
  _Cohesion score 0.046464646464646465 - nodes in this community are weakly interconnected._
- **Should `definitions` be split into smaller, more focused modules?**
  _Cohesion score 0.05128205128205128 - nodes in this community are weakly interconnected._
- **Should `properties` be split into smaller, more focused modules?**
  _Cohesion score 0.05398110661268556 - nodes in this community are weakly interconnected._
- **Should `definitions` be split into smaller, more focused modules?**
  _Cohesion score 0.06060606060606061 - nodes in this community are weakly interconnected._