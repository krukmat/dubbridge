FROM rust:1.98.1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ffmpeg \
        python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
