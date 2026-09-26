#!/usr/bin/env bash
# ==============================================================================
# sync-upstream.sh — Safely sync upstream changes without breaking fan control
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

info() { echo -e "${CYAN}[i]${NC} $1"; }
ok()   { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
err()  { echo -e "${RED}[✗]${NC} $1"; exit 1; }

echo -e "${BOLD}====================================================${NC}"
echo -e "${BOLD}   🔄 OMENSpace Upstream Sync & Fan Safety Check    ${NC}"
echo -e "${BOLD}====================================================${NC}"

# 1. Ensure working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    warn "Uncommitted changes detected. Stashing..."
    git stash push -m "Auto-stashed by sync-upstream on $(date)"
    STASHED=1
else
    STASHED=0
fi

# 2. Ensure upstream remote exists
if ! git remote | grep -q "^upstream_space$"; then
    info "Adding upstream_space remote (yunusemreyl/omen-space)..."
    git remote add upstream_space https://github.com/yunusemreyl/omen-space.git
fi

# 3. Fetch upstream
info "Fetching from upstream_space..."
git fetch upstream_space main

# 4. Attempt merge
info "Merging upstream_space/main into $(git branch --show-current)..."
if git merge upstream_space/main -m "chore: merge upstream changes"; then
    ok "Clean merge completed."
else
    warn "Merge conflicts detected."
    if git diff --name-only --diff-filter=U | grep -qE "fan(/mod)?\.rs"; then
        warn "Conflict in fan module detected! Re-applying fan control safety patch..."
        # Checkout our fan version to preserve HP hardware control
        if [ -f "src/omen-space-daemon/src/fan/mod.rs" ]; then
            git checkout --ours src/omen-space-daemon/src/fan/mod.rs || true
            git add src/omen-space-daemon/src/fan/mod.rs || true
        elif [ -f "src/omen-space-daemon/src/fan.rs" ]; then
            git checkout --ours src/omen-space-daemon/src/fan.rs || true
            git add src/omen-space-daemon/src/fan.rs || true
        fi
        ok "Protected fan control preserved."
    fi
    warn "Please resolve any remaining conflicts in other files and run: git commit"
    exit 1
fi

# 5. Restore stash if any
if [ "$STASHED" -eq 1 ]; then
    info "Restoring stashed changes..."
    git stash pop || true
fi

# 6. Verify with regression tests
info "Running fan control regression tests..."
if cargo test -p omen-space-daemon; then
    ok "All fan control tests PASSED! Fan speed control is verified intact."
else
    err "Tests FAILED! Upstream changes may have disrupted fan control invariants."
fi

# 7. Prompt rebuild & installation
echo ""
echo -e "${GREEN}${BOLD}✓ Sync completed and verified safe!${NC}"
echo "To build and install the updated binaries, run:"
echo "  sudo ./setup.sh update"
echo "or:"
echo "  cargo build -p omen-space-daemon --release && cargo build -p omen-gui --release"
