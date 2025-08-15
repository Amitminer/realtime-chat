# Build stage (use slim image with 2024 edition support)
FROM rust:1.85-slim AS builder
WORKDIR /app

# Copy the full crate to avoid "dummy main" caching pitfalls
COPY http-serve /app/http-serve
WORKDIR /app/http-serve

# Install binutils for stripping binary
RUN apt-get update && \
	apt-get install -y --no-install-recommends binutils && \
	rm -rf /var/lib/apt/lists/*

# Build the actual binary explicitly
RUN cargo build --release --bin http-serve

# Strip the binary to reduce size
RUN strip target/release/http-serve

# Runtime stage (Debian slim)
FROM debian:bookworm-slim
RUN apt-get update && \
	apt-get install -y --no-install-recommends ca-certificates && \
	rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Create non-root user
RUN groupadd -g 10001 appgroup && \
	useradd -u 10001 -g appgroup appuser

COPY --from=builder /app/http-serve/target/release/http-serve /usr/local/bin/http-serve
COPY http-serve/entrypoint.sh /usr/local/bin/entrypoint.sh
ENV PORT=8080 HOST=0.0.0.0 ROOT_DIR=/app/frontend LIVE_RELOAD=false
EXPOSE 8080
COPY frontend /app/frontend

# Change ownership to non-root user and switch to it
RUN chown -R 10001:10001 /usr/local/bin/http-serve /usr/local/bin/entrypoint.sh /app/frontend
USER 10001:10001

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
