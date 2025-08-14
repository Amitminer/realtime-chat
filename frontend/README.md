### Frontend (Terminal-style UI)

Vanilla HTML/CSS/JS chat client with a terminal aesthetic.

#### Run locally
```bash
# Serve with the bundled http server
cargo run -p http-serve -- --root ./frontend --port 8080
# Then open http://localhost:8080
```

#### How it connects
- Reads `WS_URL` from `/config.js` provided by `http-serve`.
- If not set, defaults to `ws://<host>:9001`.
- For production, prefer `wss://` behind a reverse proxy. Without TLS, traffic can be intercepted/modified.

#### Security notes
- Symmetric encryption using a single shared password; no forward secrecy.
- Optionally stores password/username in cookies for auto-login. Disable or clear on shared machines.

#### Build/Deploy
- Static assets only. Serve `frontend/` behind any web server.
- For Docker, use the provided `http-serve/Dockerfile` via compose.
