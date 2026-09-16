#!/usr/bin/env bash
#
# Запускает все сервисы ptrans в отдельных tmux-сессиях.
#

set -euo pipefail

# ============ КОНФИГ ============

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PTRANS_DIR="$ROOT_DIR/workspace/ptrans"
FRONTEND_DIR="$PTRANS_DIR/app/frontend"
BACKEND_DIR="$PTRANS_DIR/app/backend"
MERKLE_DIR="$PTRANS_DIR/app/merkle"

# Путь внутри контейнера solana-dev (отличается от пути на хосте!)
PROVER_DIR_CONTAINER="/home/ubuntu/workspace/ptrans/app/prover"

SESSION_PREFIX="ptrans"

SESSIONS=("prover" "backend" "merkle" "frontend")

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============ ФУНКЦИИ ============

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_err() { echo -e "${RED}[ERR]${NC} $*"; }

check_deps() {
  if ! command -v tmux &>/dev/null; then
    log_err "tmux не установлен. Установи: sudo apt install tmux"
    exit 1
  fi
  if ! command -v docker &>/dev/null; then
    log_err "docker не установлен"
    exit 1
  fi
}

start_infra() {
  log_info "Запускаю Postgres + Redis..."
  cd "$ROOT_DIR"
  docker compose up -d postgres redis
  sleep 2

  if ! docker compose ps postgres | grep -q "Up"; then
    log_err "Postgres не запустился"
    exit 1
  fi
  if ! docker compose ps redis | grep -q "Up"; then
    log_err "Redis не запустился"
    exit 1
  fi
  log_ok "Postgres и Redis запущены"
}

start_merkle() {
  local session="${SESSION_PREFIX}-merkle"
  if tmux has-session -t "$session" 2>/dev/null; then
    log_warn "Сессия $session уже существует — пропускаю"
    return
  fi

  log_info "Запускаю merkle..."
  tmux new-session -d -s "$session" -c "$MERKLE_DIR" \
    "node src/server.js 2>&1 | tee /tmp/ptrans-merkle.log"
  log_ok "merkle → tmux attach -t $session"
}

start_prover() {
  local session="${SESSION_PREFIX}-prover"
  if tmux has-session -t "$session" 2>/dev/null; then
    log_warn "Сессия $session уже существует — пропускаю"
    return
  fi

  if ! docker ps --filter "name=solana-dev" --filter "status=running" | grep -q solana-dev; then
    log_warn "Контейнер solana-dev не запущен. Запускаю..."
    cd "$ROOT_DIR"
    docker compose up -d solana-dev
    sleep 3
  fi

  log_info "Запускаю prover (в контейнере solana-dev)..."
  tmux new-session -d -s "$session" -c "$ROOT_DIR" \
    "docker compose exec solana-dev bash -c 'cd $PROVER_DIR_CONTAINER && PROVER_PORT=4002 cargo run' 2>&1 | tee /tmp/ptrans-prover.log"
  log_ok "prover → tmux attach -t $session"
}

start_backend() {
  local session="${SESSION_PREFIX}-backend"
  if tmux has-session -t "$session" 2>/dev/null; then
    log_warn "Сессия $session уже существует — пропускаю"
    return
  fi

  log_info "Запускаю backend..."
  # PORT читается из workspace/ptrans/.env через dotenvy
  tmux new-session -d -s "$session" -c "$BACKEND_DIR" \
    "cargo run 2>&1 | tee /tmp/ptrans-backend.log"
  log_ok "backend → tmux attach -t $session"
}

start_frontend() {
  local session="${SESSION_PREFIX}-frontend"
  if tmux has-session -t "$session" 2>/dev/null; then
    log_warn "Сессия $session уже существует — пропускаю"
    return
  fi

  log_info "Запускаю frontend..."
  tmux new-session -d -s "$session" -c "$FRONTEND_DIR" \
    "npm run dev 2>&1 | tee /tmp/ptrans-frontend.log"
  log_ok "frontend → tmux attach -t $session"
}

wait_services() {
  log_info "Ожидаю готовности сервисов..."

  for i in {1..60}; do
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:4001/api/health 2>/dev/null | grep -q "200"; then
      log_ok "backend готов"
      break
    fi
    sleep 1
    if [ "$i" -eq 60 ]; then
      log_warn "backend не ответил за 60 секунд (проверь: tmux attach -t ${SESSION_PREFIX}-backend)"
    fi
  done

  for i in {1..15}; do
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:4003/health 2>/dev/null | grep -q "200"; then
      log_ok "merkle готов"
      break
    fi
    sleep 1
    if [ "$i" -eq 15 ]; then
      log_warn "merkle не ответил за 15 секунд"
    fi
  done

  for i in {1..15}; do
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:5173 2>/dev/null | grep -q "200"; then
      log_ok "frontend готов"
      break
    fi
    sleep 1
    if [ "$i" -eq 15 ]; then
      log_warn "frontend не ответил за 15 секунд"
    fi
  done
}

stop_all() {
  log_info "Останавливаю все сервисы..."

  for name in "${SESSIONS[@]}"; do
    local session="${SESSION_PREFIX}-${name}"
    if tmux has-session -t "$session" 2>/dev/null; then
      tmux kill-session -t "$session"
      log_ok "Остановлена сессия $session"
    fi
  done

  log_info "Останавливаю Postgres + Redis..."
  cd "$ROOT_DIR"
  docker compose stop postgres redis

  log_ok "Всё остановлено"
}

show_status() {
  echo ""
  echo "=== tmux-сессии ==="
  for name in "${SESSIONS[@]}"; do
    local session="${SESSION_PREFIX}-${name}"
    if tmux has-session -t "$session" 2>/dev/null; then
      echo -e "  ${GREEN}✓${NC} $session"
    else
      echo -e "  ${RED}✗${NC} $session"
    fi
  done

  echo ""
  echo "=== HTTP health ==="
  for url in \
    "http://localhost:4001/api/health|backend" \
    "http://localhost:4003/health|merkle" \
    "http://localhost:5173|frontend"; do
    local endpoint="${url%|*}"
    local name="${url#*|}"
    local code
    code=$(curl -s -o /dev/null -w "%{http_code}" "$endpoint" 2>/dev/null || echo "000")
    if [ "$code" = "200" ]; then
      echo -e "  ${GREEN}✓${NC} $name ($code)"
    else
      echo -e "  ${RED}✗${NC} $name ($code)"
    fi
  done

  echo ""
  echo "=== Docker ==="
  cd "$ROOT_DIR"
  docker compose ps postgres redis 2>/dev/null || true
  echo ""
}

attach_session() {
  local name="${1:-}"
  if [ -z "$name" ]; then
    log_err "Укажи имя сессии: backend | prover | merkle | frontend"
    exit 1
  fi
  local session="${SESSION_PREFIX}-${name}"
  if ! tmux has-session -t "$session" 2>/dev/null; then
    log_err "Сессия $session не найдена"
    exit 1
  fi
  tmux attach -t "$session"
}

show_help() {
  cat <<EOF
Использование: $0 [команда]

Команды:
  (без аргументов)   Запустить все сервисы
  stop               Остановить все сервисы
  status             Показать статус сервисов
  attach <name>      Подключиться к tmux-сессии (backend|prover|merkle|frontend)
  logs <name>        Показать логи сервиса
  help               Показать эту справку

Сервисы:
  backend  — Rust API (порт 4001)
  prover   — Rust proof generation (порт 4002, в контейнере solana-dev)
  merkle   — Node.js Merkle service (порт 4003)
  frontend — Vue/Vite (порт 5173)

Инфраструктура (docker compose):
  postgres — порт 5432
  redis    — порт 6379

EOF
}

show_logs() {
  local name="${1:-}"
  if [ -z "$name" ]; then
    log_err "Укажи имя сервиса: backend | prover | merkle | frontend"
    exit 1
  fi
  local logfile="/tmp/ptrans-${name}.log"
  if [ ! -f "$logfile" ]; then
    log_err "Лог $logfile не найден"
    exit 1
  fi
  tail -f "$logfile"
}

# ============ MAIN ============

main() {
  local cmd="${1:-start}"

  case "$cmd" in
  start)
    check_deps
    start_infra
    start_merkle
    start_prover
    start_backend
    start_frontend
    wait_services
    echo ""
    log_ok "Все сервисы запущены!"
    echo ""
    echo "Подключиться к сессии: tmux attach -t ptrans-backend"
    echo "Показать статус:       $0 status"
    echo "Остановить всё:        $0 stop"
    echo ""
    ;;
  stop)
    check_deps
    stop_all
    ;;
  status)
    show_status
    ;;
  attach)
    attach_session "${2:-}"
    ;;
  logs)
    show_logs "${2:-}"
    ;;
  help | --help | -h)
    show_help
    ;;
  *)
    log_err "Неизвестная команда: $cmd"
    show_help
    exit 1
    ;;
  esac
}

main "$@"
