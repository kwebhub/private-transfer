# ============================================================
# Backend (Rust + axum) — multi-stage build
# ============================================================

# ---------- Stage 1: builder ----------
FROM rust:1.98-slim AS builder

WORKDIR /build

# Системные зависимости для сборки
RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl-dev \
  pkg-config \
  build-essential \
  && rm -rf /var/lib/apt/lists/*

# 1. Копируем манифесты
COPY Cargo.toml Cargo.lock ./

# 2. Копируем весь src + migrations (для сборки нужны lib.rs и main.rs)
COPY src ./src
COPY migrations ./migrations

# 3. Собираем release
RUN cargo build --release

# ---------- Stage 2: runtime ----------
FROM debian:bookworm-slim

WORKDIR /app

# Runtime-зависимости
RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl3 \
  ca-certificates \
  curl \
  && rm -rf /var/lib/apt/lists/*

# Копируем бинарь из builder
COPY --from=builder /build/target/release/ptrans-backend /app/ptrans-backend

# Копируем миграции
COPY --from=builder /build/migrations /app/migrations

# Непривилегированный пользователь
RUN useradd -u 1001 -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 4001

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:4001/api/health || exit 1

CMD ["/app/ptrans-backend"]
