# Contributing to Gõ Nhanh

Thank you for your interest in contributing! 🎉

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/gonhanh.org`
3. Run setup: `./scripts/setup.sh`
4. Create a branch: `git checkout -b feature/my-feature`

## Development Workflow

### For Rust Core

```bash
cd core

# Make changes to src/

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Build
cargo build --release
```

### For macOS UI

1. Open `platforms/macos/GoNhanh.xcodeproj` in Xcode
2. Make changes to Swift files
3. Test in Xcode
4. Build and run

## Coding Standards

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- No `cargo clippy` warnings
- Add tests for new features
- Document public APIs

### Swift

- Follow [Swift Style Guide](https://google.github.io/swift/)
- Use SwiftUI best practices
- Add comments for complex logic

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add VNI support
fix: keyboard hook on Sonoma
docs: update installation guide
test: add engine tests
```

## Pull Requests

1. Update documentation if needed
2. Add tests for new features
3. Ensure all tests pass
4. Update CHANGELOG.md
5. Create PR with clear description

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How did you test this?

## Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
```

## Code Review

- Be respectful and constructive
- Focus on code, not the person
- Suggest improvements, don't demand
- Approve when ready

## Questions?

Open an issue or discussion on GitHub!
