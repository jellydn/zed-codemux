#!/bin/bash

# Setup script for GitHub secrets required by the release workflow
# CARGO_REGISTRY_TOKEN - crates.io API token for publishing
# HOMEBREW_TAP_TOKEN - GitHub PAT with repo access to jellydn/homebrew-tap

set -e

REPO="jellydn/zed-codemux"
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔐 GitHub Secrets Setup for zed-codemux${NC}"
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ GitHub CLI (gh) is not installed${NC}"
    echo ""
    echo "Install it first:"
    echo "  macOS:    brew install gh"
    echo "  Linux:    https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
    echo "  Windows:  winget install --id GitHub.cli"
    echo ""
    exit 1
fi

# Check if logged in
echo -e "${BLUE}Checking GitHub authentication...${NC}"
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}⚠️  Not authenticated with GitHub${NC}"
    echo "Run: gh auth login"
    exit 1
fi
echo -e "${GREEN}✓ Authenticated with GitHub${NC}"
echo ""

# Check current secrets
echo -e "${BLUE}Checking existing secrets in ${REPO}...${NC}"
EXISTING_SECRETS=$(gh secret list -R "$REPO" 2>/dev/null || echo "")

if echo "$EXISTING_SECRETS" | grep -q "CARGO_REGISTRY_TOKEN"; then
    echo -e "${GREEN}✓ CARGO_REGISTRY_TOKEN is set${NC}"
else
    echo -e "${YELLOW}⚠️  CARGO_REGISTRY_TOKEN is NOT set${NC}"
fi

if echo "$EXISTING_SECRETS" | grep -q "HOMEBREW_TAP_TOKEN"; then
    echo -e "${GREEN}✓ HOMEBREW_TAP_TOKEN is set${NC}"
else
    echo -e "${YELLOW}⚠️  HOMEBREW_TAP_TOKEN is NOT set${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# CARGO_REGISTRY_TOKEN setup
if ! echo "$EXISTING_SECRETS" | grep -q "CARGO_REGISTRY_TOKEN"; then
    echo -e "${BLUE}📦 Setting up CARGO_REGISTRY_TOKEN${NC}"
    echo ""
    echo "To publish to crates.io, you need an API token:"
    echo ""
    echo "1. Go to https://crates.io/settings/tokens"
    echo "2. Click 'New Token'"
    echo "3. Name it 'zed-codemux-release'"
    echo "4. Select scopes: 'publish-update'"
    echo "5. Copy the token (starts with 'cro-')"
    echo ""
    echo -e "${YELLOW}Paste your crates.io token:${NC}"
    read -rs CARGO_TOKEN
    echo ""

    if [[ -n "$CARGO_TOKEN" ]]; then
        echo "$CARGO_TOKEN" | gh secret set CARGO_REGISTRY_TOKEN -R "$REPO"
        echo -e "${GREEN}✓ CARGO_REGISTRY_TOKEN added to ${REPO}${NC}"
    else
        echo -e "${RED}❌ No token provided, skipping...${NC}"
    fi
    echo ""
fi

# HOMEBREW_TAP_TOKEN setup
if ! echo "$EXISTING_SECRETS" | grep -q "HOMEBREW_TAP_TOKEN"; then
    echo -e "${BLUE}🍺 Setting up HOMEBREW_TAP_TOKEN${NC}"
    echo ""
    echo "To update the Homebrew formula, you need a GitHub PAT:"
    echo ""
    echo "1. Go to https://github.com/settings/tokens/new"
    echo "2. Name it 'homebrew-tap-zed-codemux'"
    echo "3. Select scopes: 'repo' (full control of private repositories)"
    echo "4. Generate and copy the token"
    echo ""
    echo -e "${YELLOW}Paste your GitHub PAT:${NC}"
    read -rs HOMEBREW_TOKEN
    echo ""

    if [[ -n "$HOMEBREW_TOKEN" ]]; then
        echo "$HOMEBREW_TOKEN" | gh secret set HOMEBREW_TAP_TOKEN -R "$REPO"
        echo -e "${GREEN}✓ HOMEBREW_TAP_TOKEN added to ${REPO}${NC}"
    else
        echo -e "${RED}❌ No token provided, skipping...${NC}"
    fi
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verify final state
FINAL_SECRETS=$(gh secret list -R "$REPO" 2>/dev/null || echo "")

if echo "$FINAL_SECRETS" | grep -q "CARGO_REGISTRY_TOKEN" && echo "$FINAL_SECRETS" | grep -q "HOMEBREW_TAP_TOKEN"; then
    echo -e "${GREEN}🎉 All secrets configured!${NC}"
    echo ""
    echo "You're ready to run:"
    echo "  git tag -a v0.1.0 -m 'Release v0.1.0'"
    echo "  git push origin v0.1.0"
    echo ""
    echo "The release workflow will:"
    echo "  • Build binaries for Linux, macOS (Intel/ARM), Windows"
    echo "  • Publish to crates.io"
    echo "  • Create GitHub Release"
    echo "  • Update Homebrew formula"
else
    echo -e "${YELLOW}⚠️  Some secrets are still missing${NC}"
    echo ""
    echo "Run this script again when you have the tokens ready."
fi

echo ""
