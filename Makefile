.PHONY: help all test format build clean setup install release release-minor release-major

# Auto-versioning
TAG := $(shell git describe --tags --abbrev=0 2>/dev/null || echo v0.0.0)
VER := $(subst v,,$(TAG))
NEXT_PATCH := $(shell echo $(VER) | awk -F. '{print $$1"."$$2"."$$3+1}')
NEXT_MINOR := $(shell echo $(VER) | awk -F. '{print $$1"."$$2+1".0"}')
NEXT_MAJOR := $(shell echo $(VER) | awk -F. '{print $$1+1".0.0"}')

# Default target
.DEFAULT_GOAL := help

help: ## Show this help
	@echo "⚡ Gõ Nhanh - Vietnamese Input Method Engine"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "\033[1;34mDevelopment:\033[0m"
	@grep -E '^(test|format|build|clean):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[1;32m%-12s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "\033[1;33mSetup & Install:\033[0m"
	@grep -E '^(setup|install):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[1;32m%-12s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "\033[1;31mRelease:\033[0m"
	@grep -E '^(release|release-minor|release-major|all):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[1;32m%-15s\033[0m %s\n", $$1, $$2}'

all: test build ## Run test + build

test: ## Run tests
	@cd core && cargo test

format: ## Format & lint
	@cd core && cargo fmt && cargo clippy -- -D warnings

build: format ## Build core + macos app
	@./scripts/build-core.sh
	@./scripts/build-macos.sh

clean: ## Clean build artifacts
	@cd core && cargo clean
	@rm -rf platforms/macos/build

setup: ## Setup dev environment
	@./scripts/setup.sh

install: build ## Install app to /Applications
	@cp -r platforms/macos/build/Release/GoNhanh.app /Applications/

release: ## Patch release (1.0.9 → 1.0.10)
	@echo "$(TAG) → v$(NEXT_PATCH)"
	@git add -A && git commit -m "release: v$(NEXT_PATCH)" --allow-empty
	@git tag v$(NEXT_PATCH) && git push origin main v$(NEXT_PATCH)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"

release-minor: ## Minor release (1.0.9 → 1.1.0)
	@echo "$(TAG) → v$(NEXT_MINOR)"
	@git add -A && git commit -m "release: v$(NEXT_MINOR)" --allow-empty
	@git tag v$(NEXT_MINOR) && git push origin main v$(NEXT_MINOR)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"

release-major: ## Major release (1.0.9 → 2.0.0)
	@echo "$(TAG) → v$(NEXT_MAJOR)"
	@git add -A && git commit -m "release: v$(NEXT_MAJOR)" --allow-empty
	@git tag v$(NEXT_MAJOR) && git push origin main v$(NEXT_MAJOR)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"
