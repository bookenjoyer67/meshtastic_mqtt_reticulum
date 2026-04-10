# Contributing to Meshtastic MQTT Reticulum Bridge

Thank you for your interest in contributing to the Meshtastic MQTT Reticulum Bridge project! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## How to Contribute

### Reporting Bugs
1. Check if the bug has already been reported in the Issues section
2. If not, create a new issue with:
   - A clear, descriptive title
   - Steps to reproduce the bug
   - Expected behavior
   - Actual behavior
   - Environment details (OS, Rust version, etc.)
   - Relevant logs or error messages

### Suggesting Features
1. Check if the feature has already been suggested
2. Create a new issue with:
   - A clear description of the feature
   - Use cases and benefits
   - Any implementation ideas you have
   - Related issues or references

### Submitting Code Changes
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Add or update tests as needed
5. Ensure code passes existing tests: `cargo test`
6. Format your code: `cargo fmt`
7. Check for clippy warnings: `cargo clippy`
8. Commit your changes with descriptive commit messages
9. Push to your fork
10. Create a Pull Request

## Development Setup

### Prerequisites
- Rust (latest stable version)
- Cargo
- Git

### Building the Project
```bash
git clone https://github.com/yourusername/meshtastic_mqtt_reticulum.git
cd meshtastic_mqtt_reticulum
cargo build
```

### Running Tests
```bash
cargo test
```

### Running Linters
```bash
cargo fmt     # Format code
cargo clippy  # Check for common issues
```

## Code Style Guidelines

### Rust Code
- Follow Rust naming conventions (snake_case for variables/functions, CamelCase for types)
- Use meaningful variable and function names
- Add doc comments for public APIs
- Keep functions focused and small
- Handle errors appropriately (use `Result` and `Option` types)

### Commit Messages
- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 50 characters
- Provide additional details in the body if needed
- Reference issue numbers when applicable

### Documentation
- Update README.md for user-facing changes
- Add comments for complex logic
- Document public APIs with Rust doc comments
- Update configuration examples if needed

## Project Structure

```
meshtastic_mqtt_reticulum/
├── src/                    # Main source code
│   ├── main.rs            # Application entry point
│   ├── config.rs          # Configuration handling
│   ├── mqtt.rs            # MQTT client implementation
│   ├── reticulum.rs       # Reticulum interface management
│   └── message.rs         # Message conversion logic
├── Reticulum-rs/          # Reticulum library fork
├── examples/              # Example code and tests
├── docs/                  # Documentation
└── tests/                 # Integration tests
```

## Testing Guidelines

### Unit Tests
- Test individual functions and modules
- Mock external dependencies when needed
- Cover edge cases and error conditions

### Integration Tests
- Test interactions between components
- Use real MQTT broker for MQTT tests (consider using testcontainers)
- Test message flow through the entire system

### Manual Testing
- Test with actual Meshtastic devices when possible
- Verify different Reticulum interface types
- Test various message types (text, position, telemetry)

## Pull Request Process

1. Ensure your PR addresses a single issue or feature
2. Update documentation as needed
3. Add tests for new functionality
4. Ensure all tests pass
5. Request review from maintainers
6. Address review feedback
7. Once approved, a maintainer will merge your PR

## Review Guidelines

### For Reviewers
- Be constructive and respectful
- Focus on code quality and correctness
- Consider security implications
- Check for performance issues
- Verify documentation is updated

### For Authors
- Be responsive to feedback
- Explain design decisions when needed
- Be open to suggestions
- Update your PR based on feedback

## Release Process

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create a release tag
4. Build release binaries
5. Publish to crates.io (if applicable)
6. Announce the release

## Getting Help

- Check the documentation in the `docs/` directory
- Look at existing issues and PRs
- Ask questions in the Discussions section
- Join the community chat (if available)

## License

By contributing to this project, you agree that your contributions will be licensed under the project's license (to be determined).

Thank you for contributing!