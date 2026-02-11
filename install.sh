#!/bin/bash

REPO="cashlycash/peapod-v2"
OS=$(uname -s)
ARCH=$(uname -m)

echo "🫛 PeaPod Installer (Alpha)"

# Detect OS
if [ "$OS" == "Darwin" ]; then
    TARGET="darwin"
elif [ "$OS" == "Linux" ]; then
    TARGET="linux"
else
    echo "Unsupported OS: $OS"
    exit 1
fi

# Detect Arch
if [ "$ARCH" == "x86_64" ]; then
    TARGET_ARCH="x86_64"
elif [ "$ARCH" == "arm64" ] || [ "$ARCH" == "aarch64" ]; then
    TARGET_ARCH="aarch64"
else
    echo "Unsupported Architecture: $ARCH"
    exit 1
fi

echo "Detecting latest release..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not find latest release."
    exit 1
fi

echo "Found version: $LATEST_TAG"
echo "⚠️  Auto-download logic coming in v0.1.2. For now, please visit:"
echo "https://github.com/$REPO/releases/tag/$LATEST_TAG"
