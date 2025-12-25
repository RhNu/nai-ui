# Backend image for nai-ui (Rust + axum) that also serves built frontend assets

FROM node:20-bookworm-slim AS frontend
WORKDIR /web
ENV PNPM_HOME=/root/.local/share/pnpm
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

# Build frontend with relative API base (use same host)
ARG VITE_BACKEND_URL=
ENV VITE_BACKEND_URL=${VITE_BACKEND_URL}
COPY web/package.json web/pnpm-lock.yaml web/tsconfig*.json web/vite.config.ts web/index.html ./
COPY web/src ./src
COPY web/public ./public
RUN pnpm install --frozen-lockfile
RUN pnpm build

FROM rust:1-bookworm AS builder
WORKDIR /app

# Pre-copy manifests for better caching
COPY Cargo.toml .
COPY crates ./crates
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

# Create data directory for outputs and sqlite
RUN mkdir -p /data/outputs

# Copy binary and frontend assets
COPY --from=builder /app/target/release/nai-ui /app/nai-ui
COPY --from=frontend /web/dist /app/frontend

ENV OUTPUT_DIR=/data/outputs \
    BIND=0.0.0.0:11451 \
    STATIC_DIR=/app/frontend

EXPOSE 11451
CMD ["/app/nai-ui"]
