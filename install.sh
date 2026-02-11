#!/bin/bash

REPO="cashlycash/peapod-v2"
OS=$(uname -s)
ARCH=$(uname -m)

echo "🫛 PeaPod Installer (Alpha)"

# Detect Target
if [ "$OS" == "Darwin" ]; then
    PLATFORM="darwin"
    EXT="dmg" # or app.tar.gz depending on bundle
    if [ "$ARCH" == "arm64" ]; then ARCH="aarch64"; else ARCH="x86_64"; fi
elif [ "$OS" == "Linux" ]; then
    PLATFORM="linux"
    EXT="AppImage" # or deb
    ARCH="x86_64" # Assume x64 for now
else
    echo "Unsupported OS: $OS"
    exit 1
fi

echo "Detecting latest release..."
LATEST_DATA=$(curl -s "https://api.github.com/repos/$REPO/releases/latest")
TAG=$(echo "$LATEST_DATA" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$TAG" ]; then
    echo "Error: Could not find latest release."
    exit 1
fi

echo "Found version: $TAG"

# Find Asset URL (Rough match, can be improved)
# Looks for files ending in .$EXT (e.g. .AppImage)
ASSET_URL=$(echo "$LATEST_DATA" | grep "browser_download_url" | grep "$EXT" | head -n 1 | cut -d '"' -f 4)

if [ -z "$ASSET_URL" ]; then
    echo "Error: Could not find binary for $OS ($EXT)."
    echo "Visit: https://github.com/$REPO/releases/tag/$TAG"
    exit 1
fi

echo "Downloading $ASSET_URL..."
curl -L -o "PeaPod.$EXT" "$ASSET_URL"
chmod +x "PeaPod.$EXT"

echo "✅ Download complete: PeaPod.$EXT"
echo "Run it with: ./PeaPod.$EXT"
