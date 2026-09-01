# Makefile for Solana, Anchor, Node, and NPM commands using docker compose.
# This file provides an interface to run:
#   - "make solana <command>" to run Solana CLI commands.
#   - "make anchor <command>" to run Anchor CLI commands.
#   - "make node <command>" to run the Node CLI (now wrapped in an interactive bash)
#   - "make npm <command>" to run NPM commands (also wrapped in an interactive bash)
#   - "make bash" to open a shell in the container.
#
# For detailed Anchor documentation, see:
# https://www.anchor-lang.com/docs
#
# Recover keypair phrase: cinnamon false fitness napkin aisle palm supreme clump swallow normal suffer soda

SHELL := /bin/bash
DOCKER_COMPOSE ?= docker compose
CONTAINER := solana-dev

# Helper function: extracts all words after the first one as subcommands/arguments.
extract_args = $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))

.PHONY: help solana anchor node npm bash

help:
	@echo "Usage:"
	@echo "  make solana <command>       -> Runs 'solana <command>' inside the container"
	@echo "  make anchor <command>       -> Runs 'anchor <command>' inside the container"
	@echo "  make node <command>         -> Runs 'node <command>' inside the container (using bash -ic)"
	@echo "  make npm <command>          -> Runs 'npm <command>' inside the container (using bash -ic)"
	@echo "  make bash                   -> Opens an interactive shell in the container"
	@echo ""
	@echo "For full Anchor documentation, visit:"
	@echo "https://www.anchor-lang.com/docs"

# Solana target: runs any solana command provided after the target name.
solana:
	$(eval ARGS := $(call extract_args))
	@if [ -z "$(ARGS)" ]; then \
		$(DOCKER_COMPOSE) exec $(CONTAINER) solana $(ARGS) || true; \
	else \
		$(DOCKER_COMPOSE) exec $(CONTAINER) solana $(ARGS); \
	fi

# Anchor target: runs any anchor command provided after the target name.
anchor:
	$(eval ARGS := $(call extract_args))
	@if [ -z "$(ARGS)" ]; then \
		$(DOCKER_COMPOSE) exec $(CONTAINER) anchor $(ARGS) || true; \
	else \
		$(DOCKER_COMPOSE) exec $(CONTAINER) anchor $(ARGS); \
	fi

# Node target: wraps the node command inside an interactive bash shell to ensure the PATH is set properly.
node:
	$(eval ARGS := $(call extract_args))
	$(DOCKER_COMPOSE) exec $(CONTAINER) bash -ic "node $(ARGS)"

# NPM target: wraps the npm command similarly.
npm:
	$(eval ARGS := $(call extract_args))
	$(DOCKER_COMPOSE) exec $(CONTAINER) bash -ic "npm $(ARGS)"

# Bash target: opens an interactive bash shell in the container.
bash:
	$(DOCKER_COMPOSE) exec -it $(CONTAINER) bash

# Catch-all rule: prevents make from misinterpreting additional words as separate targets.
%:
	@:
