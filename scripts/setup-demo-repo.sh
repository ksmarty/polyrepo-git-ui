#!/bin/bash
# Demo repo reset script for testing Polyrepo features
# Creates a temporary git repo with merge conflict scenarios
# Run: bash scripts/setup-demo-repo.sh

set -e

DEMO_DIR="${HOME}/.polyrepo-demo"
REPO_DIR="${DEMO_DIR}/demo-repo"

echo "Setting up demo repo at ${REPO_DIR}..."

# Clean up existing demo repo
if [ -d "$REPO_DIR" ]; then
  rm -rf "$REPO_DIR"
fi

mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"

# Clean up existing demo repo
if [ -d "demo-repo" ]; then
  rm -rf "demo-repo"
fi

# Initialize repo
git init demo-repo
cd demo-repo
git config user.email "demo@polyrepo.dev"
git config user.name "Demo User"

# Create initial commit on main
echo "# Demo Repository" > README.md
echo "This is a demo repo for testing Polyrepo features." >> README.md
echo "" >> README.md
echo "## Features to test" >> README.md
echo "- Sync status (behind/ahead/dirty)" >> README.md
echo "- Merge conflicts" >> README.md
echo "- Branch management" >> README.md
git add README.md
git commit -m "Initial commit: add README"

# Add a config file
cat > config.json << 'EOF'
{
  "name": "demo-app",
  "version": "1.0.0",
  "settings": {
    "debug": false,
    "theme": "light"
  }
}
EOF
git add config.json
git commit -m "Add config file"

# Add source files
mkdir -p src
cat > src/main.js << 'EOF'
function init() {
  console.log('App initialized');
  loadConfig();
}

function loadConfig() {
  // Load configuration
  return { debug: false };
}

function processData(data) {
  return data.map(item => item * 2);
}

init();
EOF
git add src/main.js
git commit -m "Add main source file"

cat > src/utils.js << 'EOF'
export function formatDate(date) {
  return date.toISOString().split('T')[0];
}

export function validateInput(input) {
  return input && input.length > 0;
}
EOF
git add src/utils.js
git commit -m "Add utility functions"

# Add test file
mkdir -p tests
cat > tests/app.test.js << 'EOF'
import { formatDate, validateInput } from '../src/utils.js';

test('formatDate returns YYYY-MM-DD', () => {
  const date = new Date('2024-01-15');
  expect(formatDate(date)).toBe('2024-01-15');
});

test('validateInput rejects empty strings', () => {
  expect(validateInput('')).toBe(false);
  expect(validateInput(null)).toBe(false);
});
EOF
git add tests/app.test.js
git commit -m "Add test file"

echo "Created main branch with 4 commits"

# Create feature branch with conflicting changes
git checkout -b feature/update-config

# Modify config.json differently (will conflict with main)
cat > config.json << 'EOF'
{
  "name": "demo-app",
  "version": "1.1.0",
  "settings": {
    "debug": true,
    "theme": "dark",
    "newFeature": true
  }
}
EOF
git add config.json
git commit -m "Update config: bump version, enable debug"

# Modify main.js differently (will conflict with main)
cat > src/main.js << 'EOF'
function init() {
  console.log('App initialized v2');
  loadConfig();
  setupEventListeners();
}

function loadConfig() {
  // Enhanced configuration loading
  const config = { debug: true, version: '1.1.0' };
  return config;
}

function setupEventListeners() {
  document.addEventListener('click', handleClick);
}

function handleClick(e) {
  console.log('Clicked:', e.target);
}

function processData(data) {
  return data
    .filter(item => item !== null)
    .map(item => item * 2);
}

init();
EOF
git add src/main.js
git commit -m "Enhance main: add event listeners, improve init"

# Add new file on feature branch
cat > src/features.js << 'EOF'
export function enableNewFeature() {
  console.log('New feature enabled');
}

export function getFeatureFlags() {
  return {
    newFeature: true,
    betaFeatures: false
  };
}
EOF
git add src/features.js
git commit -m "Add new feature module"

echo "Created feature/update-config branch with 3 commits"

# Go back to main and make another commit (to create divergence)
git checkout main

cat > src/main.js << 'EOF'
function init() {
  console.log('App initialized v2.1');
  loadConfig();
  validateEnvironment();
}

function loadConfig() {
  // Improved config loading
  const config = { debug: false, version: '1.0.1' };
  return config;
}

function validateEnvironment() {
  if (typeof window === 'undefined') {
    console.warn('Not in browser environment');
  }
}

function processData(data) {
  return data
    .filter(item => item !== null && item !== undefined)
    .map(item => item * 2)
    .reduce((sum, val) => sum + val, 0);
}

init();
EOF
git add src/main.js
git commit -m "Improve main: add env validation, fix processData"

echo "Created divergent commit on main"
echo ""
echo "=== Demo repo ready! ==="
echo "Branches:"
echo "  main              - 5 commits"
echo "  feature/update-config - diverged (will conflict with main)"
echo ""
echo "To test merge conflict:"
echo "  1. Open Polyrepo and add: ${REPO_DIR}"
echo "  2. Select the repo, click 'Merge' to see the conflict"
echo "  3. Or run: cd ${REPO_DIR} && git checkout feature/update-config && git merge main"
echo ""
echo "To reset the demo repo, run this script again."
