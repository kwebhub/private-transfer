# ============================================================
# Makefile for zk-pool
#
# Service management:
#   make start              — start all services (tmux + docker)
#   make stop               — stop all services
#   make status             — show status
#   make attach <name>      — attach to tmux session
#   make logs <name>        — tail service log
#
# Commands inside solana-dev container:
#   make solana <cmd>       — solana <cmd> in container
#   make anchor <cmd>       — anchor <cmd> in container
#   make node <cmd>         — node <cmd> in container
#   make npm <cmd>          — npm <cmd> in container
#   make bash               — interactive shell in container
#
# ============================================================

SHELL := /bin/bash
DOCKER_COMPOSE ?= docker compose
CONTAINER := solana-dev

# Paths
ROOT_DIR := $(shell pwd)
PTRANS_DIR := $(ROOT_DIR)/workspace/ptrans
FRONTEND_DIR := $(PTRANS_DIR)/app/frontend
BACKEND_DIR := $(PTRANS_DIR)/app/backend
MERKLE_DIR := $(PTRANS_DIR)/app/merkle

# Path inside container (different from host path!)
PROVER_DIR_CONTAINER := /home/ubuntu/workspace/ptrans/app/prover

SESSION_PREFIX := ptrans

# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m

# Helper: extract all arguments after the first word
extract_args = $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))

.PHONY: help start stop status attach logs \
        solana anchor node npm bash

# ============================================================
# HELP
# ============================================================

help:
	@echo "Usage:"
	@echo ""
	@echo "  Service management:"
	@echo "    make start              Start all services (tmux + docker)"
	@echo "    make stop               Stop all services"
	@echo "    make status             Show status"
	@echo "    make attach <name>      Attach to tmux (backend|prover|merkle|frontend)"
	@echo "    make logs <name>        Tail service log"
	@echo ""
	@echo "  Commands inside solana-dev container:"
	@echo "    make solana <cmd>       solana <cmd>"
	@echo "    make anchor <cmd>       anchor <cmd>"
	@echo "    make node <cmd>         node <cmd>"
	@echo "    make npm <cmd>          npm <cmd>"
	@echo "    make bash               Interactive shell"

# ============================================================
# START / STOP / STATUS
# ============================================================

start:
	@$(MAKE) --no-print-directory _start

_start:
	@echo -e "$(BLUE)[INFO]$(NC) Starting Postgres + Redis..."
	@$(DOCKER_COMPOSE) up -d postgres redis
	@sleep 2
	@echo -e "$(GREEN)[OK]$(NC) Postgres and Redis started"
	@$(MAKE) --no-print-directory _start_merkle
	@$(MAKE) --no-print-directory _start_prover
	@$(MAKE) --no-print-directory _start_backend
	@$(MAKE) --no-print-directory _start_frontend
	@$(MAKE) --no-print-directory _wait_services
	@echo ""
	@echo -e "$(GREEN)[OK]$(NC) All services started!"
	@echo ""
	@echo "  Attach to session:  make attach backend"
	@echo "  Show status:        make status"
	@echo "  Stop all:           make stop"
	@echo ""

_start_merkle:
	@if tmux has-session -t $(SESSION_PREFIX)-merkle 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-merkle already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting merkle..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-merkle -c $(MERKLE_DIR) \
			"node src/server.js 2>&1 | tee /tmp/ptrans-merkle.log"; \
		echo -e "$(GREEN)[OK]$(NC) merkle → make attach merkle"; \
	fi

_start_prover:
	@if tmux has-session -t $(SESSION_PREFIX)-prover 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-prover already exists"; \
	else \
		if ! docker ps --filter "name=solana-dev" --filter "status=running" | grep -q solana-dev; then \
			echo -e "$(YELLOW)[WARN]$(NC) Container solana-dev is not running. Starting..."; \
			$(DOCKER_COMPOSE) up -d solana-dev; \
			sleep 3; \
		fi; \
		echo -e "$(BLUE)[INFO]$(NC) Starting prover (inside solana-dev container)..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-prover -c $(ROOT_DIR) \
			"$(DOCKER_COMPOSE) exec solana-dev bash -c 'cd $(PROVER_DIR_CONTAINER) && PROVER_PORT=4002 cargo run' 2>&1 | tee /tmp/ptrans-prover.log"; \
		echo -e "$(GREEN)[OK]$(NC) prover → make attach prover"; \
	fi

_start_backend:
	@if tmux has-session -t $(SESSION_PREFIX)-backend 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-backend already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting backend..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-backend -c $(BACKEND_DIR) \
			"cargo run 2>&1 | tee /tmp/ptrans-backend.log"; \
		echo -e "$(GREEN)[OK]$(NC) backend → make attach backend"; \
	fi

_start_frontend:
	@if tmux has-session -t $(SESSION_PREFIX)-frontend 2>/dev/null; then \
		echo -e "$(YELLOW)[WARN]$(NC) Session $(SESSION_PREFIX)-frontend already exists"; \
	else \
		echo -e "$(BLUE)[INFO]$(NC) Starting frontend..."; \
		tmux new-session -d -s $(SESSION_PREFIX)-frontend -c $(FRONTEND_DIR) \
			"npm run dev 2>&1 | tee /tmp/ptrans-frontend.log"; \
		echo -e "$(GREEN)[OK]$(NC) frontend → make attach frontend"; \
	fi

_wait_services:
	@echo -e "$(BLUE)[INFO]$(NC) Waiting for services to be ready..."
	@for i in $$(seq 1 60); do \
		if curl -s -o /dev/null -w "%{http_code}" http://localhost:4001/api/health 2>/dev/null | grep -q "200"; then \
			echo -e "$(GREEN)[OK]$(NC) backend is ready"; \
			break; \
		fi; \
		sleep 1; \
	done
	@for i in $$(seq 1 15); do \
		if curl -s -o /dev/null -w "%{http_code}" http://localhost:4003/health 2>/dev/null | grep -q "200"; then \
			echo -e "$(GREEN)[OK]$(NC) merkle is ready"; \
			break; \
		fi; \
		sleep 1; \
	done
	@for i in $$(seq 1 15); do \
		if curl -s -o /dev/null -w "%{http_code}" http://localhost:5173 2>/dev/null | grep -q "200"; then \
			echo -e "$(GREEN)[OK]$(NC) frontend is ready"; \
			break; \
		fi; \
		sleep 1; \
	done

stop:
	@echo -e "$(BLUE)[INFO]$(NC) Stopping all services..."
	@for name in prover backend merkle frontend; do \
		session="$(SESSION_PREFIX)-$$name"; \
		if tmux has-session -t "$$session" 2>/dev/null; then \
			tmux kill-session -t "$$session"; \
			echo -e "$(GREEN)[OK]$(NC) Stopped session $$session"; \
		fi; \
	done
	@$(DOCKER_COMPOSE) stop postgres redis
	@echo -e "$(GREEN)[OK]$(NC) All stopped"

status:
	@echo ""
	@echo "=== tmux sessions ==="
	@for name in prover backend merkle frontend; do \
		session="$(SESSION_PREFIX)-$$name"; \
		if tmux has-session -t "$$session" 2>/dev/null; then \
			echo -e "  $(GREEN)✓$(NC) $$session"; \
		else \
			echo -e "  $(RED)✗$(NC) $$session"; \
		fi; \
	done
	@echo ""
	@echo "=== HTTP health ==="
	@for url in \
		"http://localhost:4001/api/health|backend" \
		"http://localhost:4003/health|merkle" \
		"http://localhost:5173|frontend"; do \
		endpoint="$${url%|*}"; \
		name="$${url#*|}"; \
		code=$$(curl -s -o /dev/null -w "%{http_code}" "$$endpoint" 2>/dev/null || echo "000"); \
		if [ "$$code" = "200" ]; then \
			echo -e "  $(GREEN)✓$(NC) $$name ($$code)"; \
		else \
			echo -e "  $(RED)✗$(NC) $$name ($$code)"; \
		fi; \
	done
	@echo ""
	@echo "=== Docker ==="
	@$(DOCKER_COMPOSE) ps postgres redis 2>/dev/null || true
	@echo ""

attach:
	@if [ -z "$(call extract_args)" ]; then \
		echo "Specify session name: backend | prover | merkle | frontend"; \
		exit 1; \
	fi
	@session="$(SESSION_PREFIX)-$(call extract_args)"; \
	if ! tmux has-session -t "$$session" 2>/dev/null; then \
		echo "Session $$session not found"; \
		exit 1; \
	fi; \
	tmux attach -t "$$session"

logs:
	@if [ -z "$(call extract_args)" ]; then \
		echo "Specify service name: backend | prover | merkle | frontend"; \
		exit 1; \
	fi
	@logfile="/tmp/ptrans-$(call extract_args).log"; \
	if [ ! -f "$$logfile" ]; then \
		echo "Log $$logfile not found"; \
		exit 1; \
	fi; \
	tail -f "$$logfile"

# ============================================================
# COMMANDS INSIDE solana-dev CONTAINER
# ============================================================

solana:
	@$(eval ARGS := $(call extract_args))
	@if [ -z "$(ARGS)" ]; then \
		$(DOCKER_COMPOSE) exec $(CONTAINER) solana || true; \
	else \
		$(DOCKER_COMPOSE) exec $(CONTAINER) solana $(ARGS); \
	fi

anchor:
	@$(eval ARGS := $(call extract_args))
	@if [ -z "$(ARGS)" ]; then \
		$(DOCKER_COMPOSE) exec $(CONTAINER) anchor || true; \
	else \
		$(DOCKER_COMPOSE) exec $(CONTAINER) anchor $(ARGS); \
	fi

node:
	@$(eval ARGS := $(call extract_args))
	@$(DOCKER_COMPOSE) exec $(CONTAINER) bash -ic "node $(ARGS)"

npm:
	@$(eval ARGS := $(call extract_args))
	@$(DOCKER_COMPOSE) exec $(CONTAINER) bash -ic "npm $(ARGS)"

bash:
	@$(DOCKER_COMPOSE) exec -it $(CONTAINER) bash

# Catch-all: prevents make from interpreting arguments as targets
%:
	@:
