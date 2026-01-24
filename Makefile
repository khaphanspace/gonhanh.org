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

build: format
	@./scripts/build-core.sh
	@./scripts/build-macos.sh
	@./scripts/build-windows.sh
	@pkill -f "GoNhanh.app" 2>/dev/null || true
	@sleep 1
	@open platforms/macos/build/Release/GoNhanh.app

build-linux: format
	@cd platforms/linux && ./scripts/build.sh

clean:
	@cd core && cargo clean
	@rm -rf platforms/macos/build platforms/linux/build
	@defaults delete org.gonhanh.GoNhanh 2>/dev/null || true
	@echo "✅ Cleaned"

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
setup:
	@./scripts/setup.sh

install: build
	@cp -r platforms/macos/build/Release/GoNhanh.app /Applications/

dmg: build
	@./scripts/create-dmg-background.sh
	@./scripts/create-dmg.sh

# ============================================================================
# Release (auto-versioning from git tags)
# ============================================================================

.PHONY: release release-minor release-major

define do_release
	@echo "$(TAG) → v$(1)"
	@git add -A && git commit -m "release: v$(1)" --allow-empty
	@./scripts/generate-release-notes.sh v$(1) > /tmp/release_notes.md
	@git tag -a v$(1) -F /tmp/release_notes.md --cleanup=verbatim
	@git push origin main v$(1)
	@echo "→ https://github.com/khaphanspace/gonhanh.org/releases"
endef

release:
	$(call do_release,$(NEXT_PATCH))

release-minor:
	$(call do_release,$(NEXT_MINOR))

release-major:
	$(call do_release,$(NEXT_MAJOR))
