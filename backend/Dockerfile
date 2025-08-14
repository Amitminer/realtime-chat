# Build stage (use slim image with 2024 edition support)
FROM rust:1.85-slim as builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
# Create a dummy src to cache deps
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && cargo build --release || true
# Now copy real sources
COPY backend/src ./src
RUN cargo build --release

# Runtime stage (Debian slim)
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/realtime-chat /usr/local/bin/realtime-chat
ENV PORT=9001 HOST=0.0.0.0
EXPOSE 9001
CMD ["/usr/local/bin/realtime-chat"]
