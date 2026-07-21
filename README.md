# Polyrepo Git UI

A cross-platform desktop app for managing multiple git repositories. Built with Tauri v2 + Rust + Svelte.

## Download

**[Download the latest release](https://github.com/kyleschwartz/polyrepo-git-ui/releases/latest)** — installers are available for macOS (Intel + Apple Silicon) and Windows.

## Features

- **Repo management** — Add, browse, and organize repos into folders
- **Sync status** — See ahead/behind counts and dirty state at a glance
- **PR view** — List open pull requests across all repos via GitHub API
- **Bulk import** — Scan a parent folder for git repos and import them all
- **Theme support** — 8 built-in themes (Dark, Light, Midnight, Forest, Ocean, Solarized, Catppuccin, System)

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Tauri v2 CLI](https://v2.tauri.app/start/prerequisites/)

### Getting started

```bash
pnpm install
pnpm tauri dev
```

### Running tests

```bash
pnpm verify
```

This runs `svelte-check` for frontend and `cargo check` + `cargo test` for Rust. A git pre-commit hook runs this automatically before each commit.

## GitHub Integration

You can authenticate with GitHub using either a **Personal Access Token** or **OAuth App**.

### Option 1: Personal Access Token (simpler)

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Create a new token with these scopes:
   - `repo` — Full control of private repositories
   - `read:org` — Read organization membership
3. In the app, go to **Settings > GitHub** and paste your token

### Option 2: OAuth App (recommended, works out of the box)

The app includes a bundled OAuth App. Just click **Connect with GitHub** in Settings > GitHub — no setup required.

#### For developers: using your own OAuth App

If you're forking the project or want to use your own OAuth credentials:

1. Go to [GitHub Settings > Developer settings > OAuth Apps](https://github.com/settings/developers)
2. Click **New OAuth App**
3. Fill in:
   - **Application name**: `Polyrepo Git UI` (or anything)
   - **Homepage URL**: `http://localhost:14201`
   - **Authorization callback URL**: `http://localhost:14201/callback`
4. Click **Register application**
5. Copy the **Client ID**
6. Click **Generate a new client secret** and copy the secret (shown once)
7. Copy `.env.example` to `.env` in the project root and fill in your credentials:

```
OAUTH_CLIENT_ID=your_client_id_here
OAUTH_CLIENT_SECRET=your_client_secret_here
```

`.env` is gitignored and won't be committed.

#### How the OAuth flow works

1. In the app, go to **Settings > GitHub**
2. Click **Connect with GitHub**
3. Your browser opens with the GitHub authorization page
4. Click **Authorize**
5. You'll see "Authenticated!" — close the browser tab
6. The app shows your GitHub username

The flow runs a local callback server on port `14201` for 2 minutes. If it times out, try again.

## Configuration

Config is stored at:
- **macOS / Linux**: `~/.config/polyrepo-git-ui/config.toml`
- **Windows**: `%APPDATA%\polyrepo-git-ui\config.toml`
