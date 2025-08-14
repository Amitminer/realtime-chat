### http-serve (Static file server)

A tiny Rust HTTP/1.1 server used to serve the `frontend/` directory and a runtime `config.js` for the chat app. Supports optional live-reload when running locally. Intended for local/dev or to sit behind a reverse proxy in production.

#### Code layout
```text
src/
├─ main.rs        # Entrypoint
├─ colors.rs      # ANSI color constants (no hardcoded sequences)
├─ config.rs      # ServerConfig: env + CLI parsing
├─ server/        # Listener and startup logs
│  └─ mod.rs
├─ websocket/     # Minimal WS implementation for live reload
│  └─ mod.rs
├─ handlers.rs    # HTTP handlers, responses, MIME
└─ watcher.rs     # File watcher + client notifications
```

#### Run locally
```bash
# From repo root
cargo run -p http-serve -- --root ./frontend --port 8080
# Or inside http-serve/
# cargo run -- --root ../frontend --port 8080
```

#### Environment
- `HOST` (default `0.0.0.0`)
- `PORT` (default `8080`)
- `ROOT_DIR` (default `frontend`)
- `LIVE_RELOAD` (default `true` locally)
- `WS_URL` (optional; served via `/config.js`)

#### Styling / colors
- Logs use centralized ANSI constants from `src/colors.rs` (e.g., `GREEN`, `YELLOW`, `BOLD`, `RESET`).
- Prefer importing `use crate::colors::*;` rather than embedding escape codes.

#### Docker
```bash
docker build -t realtime-chat-http -f http-serve/Dockerfile .
docker run --rm -p 8080:8080 -e WS_URL=ws://localhost:9001 realtime-chat-http
```

#### Notes
- In Docker, `LIVE_RELOAD` is disabled by default.
- Use a reverse proxy with TLS for production and set `WS_URL` to `wss://...`.
- Without TLS, traffic (including the shared password) can be intercepted/modified.
