# Build stage (use slim image with 2024 edition support)
FROM rust:1.85-slim AS builder
WORKDIR /app

# Copy the full crate to avoid "dummy main" caching pitfalls
COPY http-serve /app/http-serve
WORKDIR /app/http-serve

# Build the actual binary explicitly
RUN cargo build --release --bin http-serve
# Runtime stage (Debian slim)
FROM debian:bookworm-slim
RUN apt-get update && \
	apt-get install -y --no-install-recommends ca-certificates && \
	rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/http-serve/target/release/http-serve /usr/local/bin/http-serve
COPY http-serve/entrypoint.sh /usr/local/bin/entrypoint.sh
ENV PORT=8080 HOST=0.0.0.0 ROOT_DIR=/app/frontend LIVE_RELOAD=false
EXPOSE 8080
COPY frontend /app/frontend
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
