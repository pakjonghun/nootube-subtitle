#!/bin/bash
# Download yt-dlp binary for local development
set -e

BINARIES_DIR="src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

OS=$(uname -s)
ARCH=$(uname -m)

if [ "$OS" = "Darwin" ]; then
  if [ "$ARCH" = "arm64" ]; then
    TARGET="aarch64-apple-darwin"
  else
    TARGET="x86_64-apple-darwin"
  fi
  URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
  FILENAME="yt-dlp-${TARGET}"
elif [ "$OS" = "Linux" ]; then
  TARGET="x86_64-unknown-linux-gnu"
  URL="https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
  FILENAME="yt-dlp-${TARGET}"
else
  echo "Use Windows script instead"
  exit 1
fi

echo "Downloading yt-dlp for ${TARGET}..."
curl -L -o "${BINARIES_DIR}/${FILENAME}" "${URL}"
chmod +x "${BINARIES_DIR}/${FILENAME}"
echo "Done: ${BINARIES_DIR}/${FILENAME}"
