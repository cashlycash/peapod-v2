#!/bin/bash
set -e

REPO="cashlycash/peapod-v2"
OS=$(uname -s)
ARCH=$(uname -m)

echo "🫛 PeaPod Installer (Alpha)"
echo "---------------------------"

# Detect Target
if [ "$OS" == "Darwin" ]; then
    EXT="dmg"
    PLATFORM="macOS"
elif [ "$OS" == "Linux" ]; then
    EXT="AppImage"
    PLATFORM="Linux"
else
    echo "❌ Unsupported OS: $OS"
    exit 1
fi

echo "🔍 Detecting latest release..."
API_URL="https://api.github.com/repos/$REPO/releases/latest"
RELEASE_JSON=$(curl -s "$API_URL")

# Check for rate limits or errors
if echo "$RELEASE_JSON" | grep -q "API rate limit exceeded"; then
    echo "❌ GitHub API rate limit exceeded. Please try again later."
    exit 1
fi

TAG=$(echo "$RELEASE_JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$TAG" ]; then
    echo "❌ Error: Could not find latest release tag."
    echo "Debug: $RELEASE_JSON"
    exit 1
fi

echo "✅ Found version: $TAG"

# Find Asset URL
ASSET_URL=$(echo "$RELEASE_JSON" | grep "browser_download_url" | grep "$EXT" | head -n 1 | cut -d '"' -f 4)

if [ -z "$ASSET_URL" ]; then
    echo "❌ Error: Could not find binary for $PLATFORM ($EXT) in release $TAG."
    echo "Visit: https://github.com/$REPO/releases/tag/$TAG"
    exit 1
fi

FILENAME="PeaPod-$TAG.$EXT"

echo "⬇️  Downloading $FILENAME..."
curl -L -o "$FILENAME" "$ASSET_URL"
chmod +x "$FILENAME"

echo ""
echo "✅ Installation Complete!"
echo "Run with: ./$FILENAME"
