#!/bin/sh
set -eu

echo "[http-serve] Starting..."
echo "[http-serve] HOST=${HOST:-unset} PORT=${PORT:-unset} ROOT_DIR=${ROOT_DIR:-unset} LIVE_RELOAD=${LIVE_RELOAD:-unset}"

# Ensure ROOT_DIR exists; if not, create a minimal placeholder so server doesn't exit silently
if [ -z "${ROOT_DIR:-}" ]; then
  export ROOT_DIR="/app/frontend"
fi

if [ ! -d "$ROOT_DIR" ]; then
  echo "[http-serve] WARN: ROOT_DIR '$ROOT_DIR' not found. Creating placeholder..."
  mkdir -p "$ROOT_DIR"
  cat >"$ROOT_DIR/index.html" <<'HTML'
<!doctype html>
<html><head><meta charset="utf-8"><title>frontend missing</title></head>
<body style="font-family: sans-serif;">
  <h1>Frontend directory missing in image</h1>
  <p>The container did not find the expected static assets at <code>$ROOT_DIR</code>.</p>
</body></html>
HTML
fi

ls -la "$ROOT_DIR" || true

exec /usr/local/bin/http-serve
