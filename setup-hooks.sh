#!/bin/bash
# Setup script to install git hooks for this project
# Run this script once after cloning the repository

set -e

HOOKS_DIR=".git/hooks"
SOURCE_HOOKS_DIR="hooks"

echo "Setting up git hooks..."

# Check if .git directory exists
if [ ! -d ".git" ]; then
    echo "Error: .git directory not found. Please run this script from the repository root."
    exit 1
fi

# Check if hooks directory exists, create if not
if [ ! -d "$HOOKS_DIR" ]; then
    mkdir -p "$HOOKS_DIR"
fi

# Copy pre-commit hook
if [ -f "$SOURCE_HOOKS_DIR/pre-commit" ]; then
    cp "$SOURCE_HOOKS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
    chmod +x "$HOOKS_DIR/pre-commit"
    echo "✓ Installed pre-commit hook"
else
    echo "Error: $SOURCE_HOOKS_DIR/pre-commit not found"
    exit 1
fi

echo ""
echo "Git hooks installed successfully!"
echo "The following checks will run before each commit:"
echo "  - cargo fmt --check (code formatting)"
echo "  - cargo clippy (linting)"
echo "  - cargo test (unit tests)"
