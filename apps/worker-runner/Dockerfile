# Build stage
FROM rust:1-bookworm@sha256:6e957ef098dcc77d33e310261e4ed5843bb108d5c3b5dc2b476cbc8b6caf53fa AS builder

# Copy the entire workspace to resolve path dependencies
COPY . /usr/src/app

# Build only the dubbridge-worker-runner package
WORKDIR /usr/src/app
RUN cargo build --release -p dubbridge-worker-runner

# Runtime stage
FROM debian:bookworm-slim@sha256:817e6cf99d6fc127ff4ffe8580049b60deba0adfbbb2bd65ddc3ef8fbb7aade0 AS runtime

# Install minimal runtime dependencies plus native media tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    ffmpeg \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
WORKDIR /app

# Copy only the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/dubbridge-worker-runner /app/dubbridge-worker-runner

# Copy config directory (config files are checked into git, no secrets)
COPY config/ /app/config/

# Set config directory path (DUBBRIDGE_ENV is NOT set here - supplied at runtime)
ENV DUBBRIDGE_CONFIG_DIR=/app/config

COPY workers/asr-worker-py/requirements.txt /app/asr_worker/requirements.txt
RUN pip3 install --no-cache-dir --break-system-packages -r /app/asr_worker/requirements.txt
COPY workers/asr-worker-py/main.py /app/asr_worker/main.py
ENV DUBBRIDGE_ASR_WORKER_PATH=/app/asr_worker/main.py
ENV DUBBRIDGE_ASR_WORKER_PYTHON=python3
ENV ASR_MODEL_SIZE=small

# Run as PID 1
ENTRYPOINT ["/app/dubbridge-worker-runner"]
