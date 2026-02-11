#!/bin/bash
set -e

NEW_VERSION=$1

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: ./scripts/bump_version.sh <version>"
    exit 1
fi

# Remove 'v' prefix if present
VERSION=${NEW_VERSION#v}

echo "🚀 Bumping version to $VERSION..."

# 1. Bump package.json
npm version $VERSION --no-git-tag-version --allow-same-version

# 2. Bump Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# 3. Bump tauri.conf.json
# Using sed for simplicity (jq would be cleaner but adds dep)
sed -i "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

echo "✅ Updated package.json, Cargo.toml, and tauri.conf.json to $VERSION"
echo "Don't forget to commit and tag!"
