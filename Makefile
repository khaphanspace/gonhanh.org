# ============================================================================
# Gõ Nhanh - Vietnamese Input Method Engine
# ============================================================================

.DEFAULT_GOAL := help

# Version from git tag
TAG := $(shell git describe --tags --abbrev=0 --match "v*" 2>/dev/null || echo v0.0.0)
VER := $(subst v,,$(TAG))
NEXT_PATCH := $(shell echo $(VER) | awk -F. '{print $$1"."$$2"."$$3+1}')
NEXT_MINOR := $(shell echo $(VER) | awk -F. '{print $$1"."$$2+1".0"}')
NEXT_MAJOR := $(shell echo $(VER) | awk -F. '{print $$1+1".0.0"}')

# ============================================================================
# Help
# ============================================================================

.PHONY: help
help:
	@echo "⚡ Gõ Nhanh - Vietnamese Input Method Engine"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "\033[1;34mDev:\033[0m"
	@echo "  \033[1;32mtest\033[0m        Run Rust tests"
	@echo "  \033[1;32mformat\033[0m      Format + lint"
	@echo "  \033[1;32mbuild\033[0m       Build + auto-open app"
	@echo "  \033[1;32mbuild-linux\033[0m Build Linux Fcitx5"
	@echo "  \033[1;32mclean\033[0m       Clean artifacts"
	@echo ""
	@echo "\033[1;35mDebug:\033[0m"
	@echo "  \033[1;32mwatch\033[0m       Tail debug log"
	@echo "  \033[1;32mperf\033[0m        Check RAM/leaks"
	@echo ""
	@echo "\033[1;33mInstall:\033[0m"
	@echo "  \033[1;32msetup\033[0m       Setup dev environment"
	@echo "  \033[1;32minstall\033[0m     Build + copy to /Applications"
	@echo "  \033[1;32mdmg\033[0m         Create DMG installer"
	@echo ""
	@echo "\033[1;31mRelease:\033[0m"
	@echo "  \033[1;32mrelease\033[0m       Patch  $(TAG) → v$(NEXT_PATCH)"
	@echo "  \033[1;32mrelease-minor\033[0m Minor  $(TAG) → v$(NEXT_MINOR)"
	@echo "  \033[1;32mrelease-major\033[0m Major  $(TAG) → v$(NEXT_MAJOR)"

# ============================================================================
# Development
# ============================================================================

.PHONY: test format build build-linux clean all
all: test build

test:
	@cd core && cargo test

format:
	@cd core && cargo fmt && cargo clippy -- -D warnings

build: format ## Build core + macos app
	@./scripts/build/core.sh
	@./scripts/build/macos.sh
	@./scripts/build/windows.sh
	@open platforms/macos/build/Release/GoNhanh.app

build-linux: format
	@cd platforms/linux && ./scripts/build.sh

clean: ## Clean build + settings
	@cd core && cargo clean
	@rm -rf platforms/macos/build
	@rm -rf platforms/linux/build
	@defaults delete org.gonhanh.GoNhanh 2>/dev/null || true
	@echo "✅ Cleaned build artifacts + settings"

# ============================================================================
# Debug
# ============================================================================

.PHONY: watch perf
watch:
	@rm -f /tmp/gonhanh_debug.log && touch /tmp/gonhanh_debug.log
	@echo "📋 Watching /tmp/gonhanh_debug.log (Ctrl+C to stop)"
	@tail -f /tmp/gonhanh_debug.log

perf:
	@PID=$$(pgrep -f "GoNhanh.app" | head -1); \
	if [ -n "$$PID" ]; then \
		echo "📊 GoNhanh (PID $$PID)"; \
		ps -o rss=,vsz= -p $$PID | awk '{printf "RAM: %.1f MB | VSZ: %.0f MB\n", $$1/1024, $$2/1024}'; \
		echo "Threads: $$(ps -M -p $$PID | tail -n +2 | wc -l | tr -d ' ')"; \
		leaks $$PID 2>/dev/null | grep -E "(Physical|leaked)" | head -3; \
	else echo "GoNhanh not running"; fi

# ============================================================================
# Install
# ============================================================================

.PHONY: setup install dmg
setup: ## Setup dev environment
	@./scripts/setup/macos.sh

install: build
	@cp -r platforms/macos/build/Release/GoNhanh.app /Applications/

dmg: build ## Create DMG installer
	@./scripts/release/dmg-background.sh
	@./scripts/release/dmg.sh

# ============================================================================
# Release (auto-versioning from git tags)
# ============================================================================

.PHONY: release release-minor release-major

release: ## Patch release (1.0.9 → 1.0.10)
	@echo "$(TAG) → v$(NEXT_PATCH)"
	@git add -A && git commit -m "release: v$(NEXT_PATCH)" --allow-empty
	@./scripts/release/notes.sh v$(NEXT_PATCH) > /tmp/release_notes.md
	@git tag -a v$(NEXT_PATCH) -F /tmp/release_notes.md --cleanup=verbatim
	@git push origin main v$(NEXT_PATCH)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"

release-minor: ## Minor release (1.0.9 → 1.1.0)
	@echo "$(TAG) → v$(NEXT_MINOR)"
	@git add -A && git commit -m "release: v$(NEXT_MINOR)" --allow-empty
	@./scripts/release/notes.sh v$(NEXT_MINOR) > /tmp/release_notes.md
	@git tag -a v$(NEXT_MINOR) -F /tmp/release_notes.md --cleanup=verbatim
	@git push origin main v$(NEXT_MINOR)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"

release-major: ## Major release (1.0.9 → 2.0.0)
	@echo "$(TAG) → v$(NEXT_MAJOR)"
	@git add -A && git commit -m "release: v$(NEXT_MAJOR)" --allow-empty
	@./scripts/release/notes.sh v$(NEXT_MAJOR) > /tmp/release_notes.md
	@git tag -a v$(NEXT_MAJOR) -F /tmp/release_notes.md --cleanup=verbatim
	@git push origin main v$(NEXT_MAJOR)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"
