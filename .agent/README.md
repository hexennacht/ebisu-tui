# Ebisu TUI - Agent Context

> **Last Updated:** 2026-01-31  
> **Project Status:** Early Development

---

## 📋 Project Overview

**Ebisu TUI** is a Terminal User Interface (TUI) application built with Rust. The project is in early development phase.

### Project Goals
- Build a modern, performant TUI application
- Follow Rust best practices and idiomatic patterns
- Maintain high code quality with comprehensive testing
- Create an intuitive and responsive user experience

---

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| **Language** | Rust (Edition 2024) |
| **Build System** | Cargo |
| **TUI Framework** | *To be determined* (e.g., ratatui, cursive, tui-rs) |
| **Terminal Backend** | *To be determined* (e.g., crossterm, termion) |

### Potential Dependencies
- `ratatui` - Modern TUI framework
- `crossterm` - Cross-platform terminal manipulation
- `tokio` - Async runtime (if needed)
- `serde` - Serialization/deserialization
- `clap` - CLI argument parsing

---

## 📐 Coding Standards

### Primary Reference
**All Rust code must follow:** [`RUST_GUIDELINES.md`](../RUST_GUIDELINES.md)

### Key Principles
1. **Prefer borrowing over cloning** - Performance first
2. **Use Clippy religiously** - Run `cargo clippy --all-targets --all-features --locked -- -D warnings`
3. **Comprehensive testing** - Unit tests, integration tests, and doc tests
4. **Document public APIs** - All public items must have `///` documentation
5. **Type-safe state management** - Use Type State Pattern where appropriate
6. **Error handling** - Use `thiserror` for library errors, avoid `unwrap()` in production

### Code Quality Checklist
- [ ] Clippy passes with no warnings
- [ ] All tests pass (`cargo test`)
- [ ] Public APIs are documented
- [ ] Error handling is explicit (no unwrap/expect in production)
- [ ] Code follows RUST_GUIDELINES.md patterns

---

## 📁 Project Structure

```
ebisu-tui/
├── .agent/                 # Agent context and workflows
│   ├── README.md          # This file
│   └── workflows/         # Task-specific workflows
├── src/
│   └── main.rs            # Application entry point
├── tests/                 # Integration tests (to be created)
├── rust-best-practices/   # Reference materials
├── Cargo.toml             # Project manifest
├── RUST_GUIDELINES.md     # Coding standards
└── README.md              # Project documentation (to be created)
```

---

## 🚀 Common Commands

### Development
```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the application
cargo run

# Run with arguments
cargo run -- [args]
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run doc tests
cargo test --doc
```

### Code Quality
```bash
# Run Clippy
cargo clippy --all-targets --all-features --locked -- -D warnings

# Format code
cargo fmt

# Format with nightly (for import grouping)
cargo +nightly fmt

# Check without building
cargo check
```

### Performance
```bash
# Benchmark (when benchmarks exist)
cargo bench

# Profile with flamegraph
cargo flamegraph
```

---

## 🎯 Development Guidelines

### When Adding New Features
1. **Design first** - Consider the API and state management
2. **Write tests** - TDD approach preferred
3. **Document** - Add doc comments with examples
4. **Review guidelines** - Check RUST_GUIDELINES.md for patterns
5. **Run quality checks** - Clippy, tests, formatting

### When Fixing Bugs
1. **Write a failing test** - Reproduce the bug
2. **Fix the issue** - Minimal change to fix
3. **Verify** - Ensure test passes
4. **Check for similar issues** - Look for the same pattern elsewhere

### When Refactoring
1. **Ensure tests exist** - Don't refactor untested code
2. **Make incremental changes** - Small, verifiable steps
3. **Run tests frequently** - After each change
4. **Update documentation** - Keep docs in sync

---

## 🔒 Constraints & Requirements

### Rust Version
- **Minimum:** Rust 1.80+ (Edition 2024 requires recent stable)
- **Recommended:** Latest stable Rust version

### Target Platforms
- Windows (primary development platform)
- Linux (cross-platform support)
- macOS (cross-platform support)

### Performance Requirements
- TUI must be responsive (<16ms frame time for 60fps)
- Minimal memory footprint
- Fast startup time (<100ms)

### Code Coverage
- Target: >80% code coverage
- All public APIs must have tests
- Critical paths must have comprehensive tests

---

## 📚 Reference Materials

### Internal Documentation
- **[RUST_GUIDELINES.md](../RUST_GUIDELINES.md)** - Comprehensive Rust coding standards
- **[rust-best-practices/](../rust-best-practices/)** - Original best practices handbook
- **[RUST_TUI_GUIDE.md](RUST_TUI_GUIDE.md)** - A "book" on Rust TUI development using this stack
- **[DEVELOPMENT_KNOWLEDGE.md](DEVELOPMENT_KNOWLEDGE.md)** - Architecture, DB design, and key learnings from development

### External Resources
- [Rust Official API Guidelines](https://rust-lang.github.io/api-guidelines/about.html)
- [Rust Analyzer Style Guide](https://rust-analyzer.github.io/book/contributing/style.html)
- [Ratatui Documentation](https://ratatui.rs/) (if using ratatui)
- [Crossterm Documentation](https://docs.rs/crossterm/) (if using crossterm)

---

## 🔄 Workflows

Workflows are stored in `.agent/workflows/` and can be invoked with `/workflow-name`.

### Available Workflows
*To be created as needed*

Example workflows to consider:
- `/build` - Build and run quality checks
- `/test` - Run comprehensive test suite
- `/release` - Prepare for release
- `/deploy` - Deployment procedures

---

## 💡 Agent Instructions

### When Writing Code
1. **Always reference RUST_GUIDELINES.md** for patterns and best practices
2. **Use descriptive variable and function names** - Clarity over brevity
3. **Prefer type safety** - Use enums and Type State Pattern
4. **Handle errors explicitly** - No unwrap/expect in production code
5. **Write tests alongside code** - Don't defer testing

### When Reviewing Code
1. Check against RUST_GUIDELINES.md
2. Verify Clippy passes
3. Ensure tests exist and pass
4. Confirm documentation is present
5. Look for common anti-patterns

### When Debugging
1. Add targeted tests to reproduce
2. Use `dbg!()` macro for quick debugging
3. Consider using `cargo-expand` for macro debugging
4. Profile with flamegraph if performance-related

---

## 🎨 TUI-Specific Considerations

### UI/UX Principles
- **Responsive** - UI should never freeze
- **Keyboard-first** - All actions accessible via keyboard
- **Visual feedback** - Clear indication of state and actions
- **Graceful degradation** - Handle terminal size changes

### State Management
- Use Type State Pattern for UI states
- Separate business logic from UI rendering
- Consider using message-passing architecture (Elm-like)

### Testing TUI Components
- Unit test business logic separately
- Integration tests for user flows
- Consider snapshot testing for UI layouts

---

## 📝 Notes

### Current Status
- Project initialized with basic Cargo setup
- RUST_GUIDELINES.md established
- Ready for TUI framework selection and initial implementation

### Next Steps
1. Choose TUI framework (ratatui recommended)
2. Set up project structure (modules, error types)
3. Implement basic UI skeleton
4. Add comprehensive testing infrastructure
5. Create README.md with project description

---

## 🤝 Contributing

When contributing to this project:
1. Read and follow RUST_GUIDELINES.md
2. Write tests for new features
3. Ensure Clippy passes with no warnings
4. Document public APIs
5. Keep commits focused and well-described

---

**For questions or clarifications, refer to RUST_GUIDELINES.md or ask for guidance.**
