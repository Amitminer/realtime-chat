### Backend (WebSocket Server)

Realtime chat server built with Tokio + Tungstenite. Messages use AES-256-GCM with a shared key derived from the password (SHA-256). This is a simple symmetric scheme for learning; it does not provide forward secrecy or user-level auth.

#### Run locally
```bash
# From repo root
export SERVER_PASSWORD=your-secret
cargo run -p realtime-chat
# Or inside backend/
# SERVER_PASSWORD=your-secret cargo run
```
- Binds to `HOST`:`PORT` (defaults `0.0.0.0:9001`).

#### Environment
- `SERVER_PASSWORD` (required)
- `HOST` (default `0.0.0.0`)
- `PORT` (default `9001`)
- Optional: `BIND_ADDR` (overrides host/port)

#### Docker
```bash
docker build -t realtime-chat-backend -f backend/Dockerfile .
docker run --rm -p 9001:9001 -e SERVER_PASSWORD=your-secret realtime-chat-backend
```

#### Notes
- Logs are verbose by default for debugging.
- Place a `.env` at repo root if you prefer: `SERVER_PASSWORD=...`.
- For production, terminate TLS at a reverse proxy and use `wss://`. Without TLS, traffic is not protected in transit.
