# Build stage (use slim image with 2024 edition support)
FROM rust:1.89-slim AS builder
WORKDIR /app
COPY backend/Cargo.toml ./
COPY Cargo.lock ./
# Create a dummy file to let cargo fetch dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
# Fetch deps first to cache them
RUN cargo fetch
# Now copy real sources
COPY backend/src ./src
# Build with cached deps
RUN cargo build --release

# Runtime stage (Debian slim)
FROM debian:bookworm-slim
# Create a non-root user and group
RUN groupadd --system appgroup && useradd --system --gid appgroup --no-create-home appuser

# Install required runtime dependencies
RUN apt-get update &&     apt-get install -y --no-install-recommends ca-certificates &&     rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage and set ownership
COPY --from=builder /app/target/release/realtime-chat /usr/local/bin/realtime-chat
RUN chown appuser:appgroup /usr/local/bin/realtime-chat

# Set environment variables
ENV PORT=9001 HOST=0.0.0.0

# Switch to the non-root user
USER appuser

# Expose the port and set the command
EXPOSE 9001
CMD ["/usr/local/bin/realtime-chat"]
