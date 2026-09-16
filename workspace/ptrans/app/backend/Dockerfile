# ============================================================
# Backend (Rust + axum) — multi-stage build
# ============================================================

# ---------- Stage 1: builder ----------
FROM rust:1.89-slim AS builder

WORKDIR /build

# Системные зависимости для сборки (openssl, pkg-config)
RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl-dev \
  pkg-config \
  build-essential \
  && rm -rf /var/lib/apt/lists/*

# 1. Копируем манифесты для кеша зависимостей
COPY Cargo.toml Cargo.lock ./

# 2. Создаём заглушку main.rs, чтобы собрать зависимости
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# 3. Собираем зависимости (кешируется, если Cargo.toml/Cargo.lock не менялись)
RUN cargo build --release && rm -rf src

# 4. Копируем реальный код
COPY src ./src
COPY migrations ./migrations

# 5. Пересобираем только наш код (зависимости уже в target/)
RUN touch src/main.rs && cargo build --release

# ---------- Stage 2: runtime ----------
FROM debian:bookworm-slim

WORKDIR /app

# Runtime-зависимости: openssl + ca-certificates + curl (для healthcheck)
RUN apt-get update && apt-get install -y --no-install-recommends \
  libssl3 \
  ca-certificates \
  curl \
  && rm -rf /var/lib/apt/lists/*

# Копируем бинарь из builder
COPY --from=builder /build/target/release/ptrans-backend /app/ptrans-backend

# Копируем миграции (чтобы можно было применить через docker exec)
COPY --from=builder /build/migrations /app/migrations

# Непривилегированный пользователь
RUN useradd -r -u 1001 -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 4001

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:4001/api/health || exit 1

CMD ["/app/ptrans-backend"]
