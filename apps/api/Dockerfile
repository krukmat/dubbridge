# Build stage
# TODO(owner): resolve and pin exact digest before deploy
FROM rust:1-bookworm@sha256:PLACEHOLDER AS builder

# Copy the entire workspace to resolve path dependencies
COPY . /usr/src/app

# Build only the dubbridge-api package
WORKDIR /usr/src/app
RUN cargo build --release -p dubbridge-api

# Runtime stage
# TODO(owner): resolve and pin exact digest before deploy
FROM debian:bookworm-slim@sha256:PLACEHOLDER AS runtime

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
WORKDIR /app

# Copy only the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/dubbridge-api /app/dubbridge-api

# Copy config directory (config files are checked into git, no secrets)
COPY config/ /app/config/

# Set config directory path (DUBBRIDGE_ENV is NOT set here - supplied at runtime)
ENV DUBBRIDGE_CONFIG_DIR=/app/config

# Expose the API port
EXPOSE 8080

# Run as PID 1
ENTRYPOINT ["/app/dubbridge-api"]
