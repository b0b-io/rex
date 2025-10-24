# 🔭 rex - a terminal native container registry explorer

A fast, modern command-line tool for exploring OCI-compliant container
registries.

## Project Status

⚠️ **Under Active Development** - Rex is currently in the design and early
implementation phase. The API and commands are subject to change.

## About

Rex provides both a CLI for scripting/automation and an interactive TUI for
visual exploration of OCI-compliant container registries. Primary target is
Zot registry with support for any OCI Distribution Specification v1.0+
compliant registry.

For detailed project requirements, see [req.md](req.md).

For architecture and design details:
- [librex/design.md](librex/design.md) - Core library design and architecture
- [rex/design.md](rex/design.md) - CLI and TUI interface design

## Getting Started

### Prerequisites

- Rust 1.90.0 or later
- [Just](https://github.com/casey/just) (command runner)
- [markdownlint-cli2](https://github.com/DavidAnson/markdownlint-cli2) (for
  documentation linting)

### Building

```bash
# Clone the repository
git clone https://github.com/b0b-io/rex.git
cd rex

# Build in debug mode
just build

# Build in release mode
just build-release
```

### Running

```bash
# Run development version
just run -- --help

# Or use cargo directly
cargo run -- --help
```

## Development Commands

All development tasks are managed through [Just](https://github.com/casey/just).
Run `just` to see all available commands.

### Common Tasks

```bash
# Build the project
just build

# Run tests
just test

# Run all checks (docs, format, lint, test)
just check

# Format code
just fmt

# Check formatting without changes
just fmt check

# Lint documentation
just docs

# Auto-fix documentation issues
just docs fix

# Run clippy
just lint

# Auto-fix clippy warnings
just lint fix

# Clean build artifacts
just clean
```

## Project Structure

```text
rex/
├── Cargo.toml              # Workspace manifest
├── justfile                # Command runner tasks
├── librex/                 # Core library crate
│   ├── Cargo.toml
│   ├── design.md           # Library architecture
│   └── src/
├── rex/                    # CLI binary crate
│   ├── Cargo.toml
│   ├── design.md           # CLI and TUI design
│   └── src/
├── req.md                  # Project requirements
├── dev.md                  # Development conventions
└── README.md
```

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run checks (`just check`)
5. Commit your changes using conventional commits
6. Push to your fork
7. Open a Pull Request

### Code Standards

- Follow Rust standard conventions and idioms
- Run `just fmt` before committing
- Ensure `just check` passes (docs, format, lint, tests)
- Add tests for new features
- Update documentation as needed
- Keep line length to 100 characters in markdown files

### Commit Convention

We follow conventional commits:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Test additions or modifications
- `chore:` - Build process or tooling changes

## Documentation

- [Requirements](req.md) - Detailed MVP requirements and scope
- [librex Design](librex/design.md) - Core library architecture and modules
- [rex Design](rex/design.md) - CLI and TUI interface design
- [Development](dev.md) - Development guidelines and practices
- [Public API](librex/API.md) - High-level library API documentation

## License

This project is licensed under the Apache License 2.0 - see the
[LICENSE](LICENSE) file for details.
