
# 🔐 Realtime-Chat

**Password-gated, AES-256-GCM–encrypted chat over WebSockets** — built in Rust for backend, vanilla HTML/JS for frontend.
⚠ **Not for production security** — no forward secrecy, no per-user keys, no audit.

---

## ✨ Why this exists

* Learn **WebSocket basics** in Rust with \[tokio] + \[tokio-tungstenite].
* Experiment with **symmetric encryption** in both Rust and browser JS.
* Explore **simple ops**: one `docker compose up` to run full stack.

---

## 🏗 Stack

**Backend** — Rust, tokio, tokio-tungstenite

* Auth via shared password.
* AES-256-GCM encryption with key = SHA-256(password).
* Broadcasts encrypted messages to all connected peers.

**Frontend** — Vanilla HTML/CSS/JS (terminal-style UI)

* Web Crypto API AES-GCM, same key derivation as server.
* Auth prompt → username set → encrypt/decrypt messages.

**Static server** — Minimal Rust HTTP server

* Serves `frontend/` and `/config.js` (runtime WS URL).
* Optional file-watch + live reload in dev.

**Docker** — Multi-service compose

* One command to build + run backend + static server.

---

## 📸 Screenshots

| Login                      | Auth                     | Chat                     |
| -------------------------- | ------------------------ | ------------------------ |
| ![Login](assets/login.png) | ![Auth](assets/auth.png) | ![Chat](assets/chat.png) |

---

## 🔍 Features

* **Symmetric encryption** — AES-256-GCM with SHA-256(password) key.
* **Password gate** — must auth before joining.
* **Runtime config** — frontend gets `WS_URL` from `/config.js`.
* **One-command ops** — `docker compose up -d` to run all services.

---

## 🗂 Architecture

**backend/**

* Listens for WS on `HOST:PORT` (default `0.0.0.0:9001`).
* First frame must be `{ "password": "..." }` JSON.
* On success → joins broadcast group.

**http-serve/**

* Serves static files + `/config.js`.
* Live reload in dev.
* CLI with extensive options (run with `--help` for details).

**frontend/**

* `index.html` + Tailwind CDN.
* JS encrypts outbound, decrypts inbound messages.

---

## 📁 Directory Layout

```text
.
├─ backend/              # WebSocket server + encryption
├─ http-serve/           # Static file + config server
├─ frontend/             # HTML/CSS/JS client
├─ docker-compose.yml
└─ .env.example
```

## ⚡ Quickstart (Local Dev)

```bash
cp .env.example .env
# edit SERVER_PASSWORD, etc.

# terminal 1
cd backend
cargo run

# terminal 2
cd http-serve
# See all CLI options
cargo run -- --help
# Run with custom settings
cargo run -- --root ../frontend --port 8080

# open http://localhost:8080
```

---

## 🐳 Docker (Recommended for Deploy)

```bash
cp .env.example .env
# set SERVER_PASSWORD, WS_URL if needed
docker compose up -d --build
```

* HTTP: `http://localhost:8080`
* WS: `ws://localhost:9001`

---

## 🔑 WebSocket Protocol

**Auth (client → server)**

```json
{ "password": "<SERVER_PASSWORD>" }
```

**Auth OK (server → client)**

```json
{ "success": true, "message": "Authentication successful" }
```

**Encrypted message (client → server)**

```json
{
  "username": "alice",
  "encrypted_message": "<base64>",
  "nonce": "<base64>",
  "message_type": "message"
}
```

---

## ⚠ Security Limitations

* **Single shared key** — everyone with password can read all messages.
* **No forward secrecy** — one leak = full history exposed.
* **No TLS** unless you add it via reverse proxy.
* **Minimal HTTP server** — put behind Nginx/Caddy/Traefik.

**Best practices for running it:**

* Always deploy behind TLS (`wss://`).
* Use a strong password.
* Rotate credentials periodically.

---

## 📜 License

MIT
