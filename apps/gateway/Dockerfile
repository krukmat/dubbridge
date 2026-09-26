# Build stage
FROM rust:1-bookworm@sha256:c49256cbe5ea0188bc658a689500d70c41eb51f009a7a7be209caf60a944f3ec AS builder

# Copy the entire workspace to resolve path dependencies
COPY . /usr/src/app

# Build only the dubbridge-gateway package
WORKDIR /usr/src/app
RUN cargo build --release -p dubbridge-gateway

# Runtime stage
FROM debian:bookworm-slim@sha256:f3034a6ec3c1205360777c4aae76234998866ad18806ae62b63a3f84ccad782b AS runtime

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
WORKDIR /app

# Copy only the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/dubbridge-gateway /app/dubbridge-gateway

# Copy config directory (config files are checked into git, no secrets)
COPY config/ /app/config/

# Set config directory path (DUBBRIDGE_ENV is NOT set here - supplied at runtime)
ENV DUBBRIDGE_CONFIG_DIR=/app/config

# Expose the gateway port
EXPOSE 8081

# Run as PID 1
ENTRYPOINT ["/app/dubbridge-gateway"]
