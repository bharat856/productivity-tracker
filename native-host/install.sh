#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 EXTENSION_ID" >&2
  exit 2
fi

EXTENSION_ID="$1"
ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
HOST_PATH="$ROOT_DIR/native-host/host.py"
TARGET_DIR="$HOME/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"
TARGET="$TARGET_DIR/com.daymark.productivity.json"

chmod +x "$HOST_PATH"
mkdir -p "$TARGET_DIR"
sed -e "s|__DAYMARK_HOST_PATH__|$HOST_PATH|g" -e "s|__DAYMARK_EXTENSION_ID__|$EXTENSION_ID|g" \
  "$ROOT_DIR/native-host/com.daymark.productivity.json" > "$TARGET"
printf 'Installed Native Messaging host at %s\n' "$TARGET"
