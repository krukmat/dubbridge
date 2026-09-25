FROM python:3.12-slim-bookworm AS asr-python

COPY workers/asr-worker-py/requirements-lock.txt /tmp/asr-requirements-lock.txt

RUN pip install --no-cache-dir -r /tmp/asr-requirements-lock.txt \
    && pip check

FROM rust:1.98.1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        ffmpeg \
    && rm -rf /var/lib/apt/lists/*

COPY --from=asr-python /usr/local /usr/local

WORKDIR /workspace
