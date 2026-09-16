# ============================================================
# Merkle service (Node.js + Fastify + noir_js)
# ============================================================

# ---------- Stage 1: builder ----------
FROM node:26-slim AS builder

WORKDIR /build

# pnpm
RUN corepack enable && corepack prepare pnpm@9 --activate

# Копируем манифесты
COPY package.json pnpm-lock.yaml ./

# Устанавливаем зависимости (production only)
RUN pnpm install --frozen-lockfile --prod

# ---------- Stage 2: runtime ----------
FROM node:26-slim

WORKDIR /app

# Runtime: pnpm уже есть в corepack
RUN corepack enable && corepack prepare pnpm@9 --activate

# Копируем зависимости из builder
COPY --from=builder /build/node_modules ./node_modules

# Копируем исходники
COPY package.json ./
COPY src ./src
COPY circuits ./circuits

# Непривилегированный пользователь
RUN useradd -r -u 1001 -s /bin/false appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 4003

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD node -e "fetch('http://localhost:4003/health').then(r => process.exit(r.ok ? 0 : 1)).catch(() => process.exit(1))"

CMD ["node", "src/server.js"]
