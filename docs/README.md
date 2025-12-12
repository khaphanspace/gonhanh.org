# Gõ Nhanh Documentation

Welcome to the Gõ Nhanh Vietnamese Input Method Engine documentation. This folder contains comprehensive guides for understanding, developing, and contributing to the project.

## Quick Navigation

### New to the Project?
Start here for context and onboarding:

1. **[Project Overview & PDR](./project-overview-pdr.md)** (10 min read)
   - Project vision, goals, and requirements
   - Target users and success metrics
   - Roadmap and development standards
   - Read this first to understand "why"

2. **[Codebase Summary](./codebase-summary.md)** (15 min read)
   - Complete directory structure
   - Module responsibilities and dependencies
   - Entry points for common development tasks
   - Read this to understand "what exists where"

### Ready to Contribute?

3. **[Code Standards](./code-standards.md)** (15 min read)
   - Rust and Swift coding standards
   - FFI conventions and safety guidelines
   - Commit message format (Conventional Commits)
   - PR guidelines and review requirements
   - Follow these before submitting code

4. **[System Architecture](./system-architecture.md)** (20 min read)
   - High-level system diagrams
   - Data flow: keystroke to output
   - FFI interface specification
   - Platform integration details (macOS CGEventTap)
   - Performance characteristics and scalability
   - Read this to understand "how it all works together"

## Documentation Map

```
project-overview-pdr.md
  ├─ Vision & goals
  ├─ Requirements & standards
  ├─ Success metrics
  └─ Development roadmap

codebase-summary.md
  ├─ Directory structure (100% coverage)
  ├─ Module responsibilities
  ├─ Dependency graph
  └─ Development entry points

code-standards.md
  ├─ Rust standards (fmt, clippy, no-deps)
  ├─ Swift standards (Google style)
  ├─ FFI conventions (C ABI, alignment)
  ├─ Commit message format
  └─ PR guidelines

system-architecture.md
  ├─ High-level diagram (7-layer)
  ├─ Keystroke data flow (2 examples)
  ├─ FFI specification
  ├─ Platform integration (CGEventTap)
  ├─ Component interactions
  └─ Performance characteristics
```

## Common Tasks

### I want to...

- **Understand the project**: Read project-overview-pdr.md first
- **Find a specific module**: Check codebase-summary.md → Module index
- **Add a new input method**: See "Entry Points" in codebase-summary.md
- **Fix a bug**: Check code-standards.md for commit format, then dive into relevant module
- **Understand FFI**: Read system-architecture.md → FFI Interface section
- **Optimize performance**: See system-architecture.md → Performance Characteristics
- **Port to Windows/Linux**: Review system-architecture.md → Architecture overview (core is platform-agnostic)
- **Review someone's code**: Use code-standards.md as the checklist
- **Write documentation**: Follow doc style in code-standards.md

## File Sizes & Reading Time

| File | Size | Lines | Read Time |
|------|------|-------|-----------|
| project-overview-pdr.md | 6.0 KB | 188 | 10 min |
| codebase-summary.md | 9.9 KB | 237 | 15 min |
| code-standards.md | 8.5 KB | 314 | 15 min |
| system-architecture.md | 16 KB | 351 | 20 min |
| **Total** | **40 KB** | **1,090** | **60 min** |

## Key Concepts

### Architecture Layers
1. **User Input** → CGEventTap keyboard hook (macOS)
2. **Platform Bridge** → RustBridge (Swift ↔ Rust FFI)
3. **Core Engine** → Rust (buffer → validation → transform → output)
4. **Data Layer** → Static tables (vowels, consonants, keycodes)
5. **UI Layer** → SwiftUI (menu bar, settings, update check)
6. **Persistence** → UserDefaults (settings, preferences)
7. **Distribution** → GitHub releases (auto-update)

### Core Technologies
- **Rust**: Core engine (3,500 lines, zero dependencies)
- **Swift**: macOS UI (765 lines, SwiftUI)
- **FFI**: C ABI bridge (ImeResult struct, 6 exported functions)
- **CGEventTap**: System-wide keyboard hook with Accessibility permission

### Performance Targets
- Keystroke latency: <1ms (actual: ~0.2-0.5ms)
- Memory usage: ~5MB resident set
- CPU: <2% during typing
- Test coverage: 160+ integration tests

## Standards at a Glance

### Code Quality
```bash
make format    # cargo fmt + clippy (must pass)
make test      # Run 160+ tests (all must pass)
make build     # Full release build
```

### Commit Format
```
type(scope): subject

Body (optional)

Footer (optional, use "Closes #123")
```

Examples:
- `feat(engine): add shortcut table support`
- `fix(transform): correct ư vowel placement`
- `docs(ffi): clarify memory ownership`

### PR Checklist
- [ ] Code follows standards (make format passes)
- [ ] Tests added/updated
- [ ] All tests pass (make test)
- [ ] Documentation updated (if applicable)
- [ ] Commit message follows Conventional Commits

## Contributing Guide

1. **Read**: Start with project-overview-pdr.md
2. **Navigate**: Use codebase-summary.md to find relevant code
3. **Understand**: Review system-architecture.md for context
4. **Code**: Follow standards in code-standards.md
5. **Test**: Run `make test` (all 160+ tests must pass)
6. **Commit**: Use Conventional Commits format
7. **PR**: Reference code-standards.md → PR Guidelines

## Maintenance

These docs are living documents. When you:
- Add a module → Update codebase-summary.md
- Add FFI function → Update code-standards.md + system-architecture.md
- Change performance targets → Update project-overview-pdr.md
- Modify architecture → Update system-architecture.md

Always include documentation updates in the same PR as code changes.

## Contact & Support

- **Issues**: https://github.com/khaphanspace/gonhanh.org/issues
- **Discussions**: https://github.com/khaphanspace/gonhanh.org/discussions
- **License**: GPL-3.0-or-later

## Statistics

- **Rust Code**: ~3,500 lines (core engine, platform-agnostic)
- **Swift Code**: ~1,700 lines (macOS UI + bridge)
- **C# Code**: ~1,400 lines (Windows UI + bridge)
- **Tests**: 160+ integration tests, 2100+ lines
- **Documentation**: 2,200+ lines (this folder + CLAUDE.md)
- **Coverage**: 100% of modules documented
- **Platforms**: macOS (production), Windows (production), Linux (placeholder)

### Build Statistics
- **Total Repository**: ~100,800 tokens, 385,000+ characters (repomix analysis)
- **Rust only**: ~51,159 tokens
- **All platforms**: ~100,838 tokens

---

**Last Updated**: 2025-12-10
**Status**: Production-ready for team development
**Platforms**: macOS (v1.0.21+), Windows (production), Linux (planned)
**Repository**: https://github.com/khaphanspace/gonhanh.org
