# Contributing to EDJC

Thank you for your interest in contributing to EDJC (Elite Dangerous Jump Calculator)! This document provides guidelines for contributing to the project.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/edjc.git
   cd edjc
   ```
3. **Install Rust** (if not already installed):
   - Visit [rustup.rs](https://rustup.rs/) for installation instructions
4. **Build the project**:
   ```bash
   cargo build
   ```
5. **Run tests**:
   ```bash
   cargo test
   ```

## Development Workflow

1. **Create a new branch** for your feature or bug fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. **Make your changes** following the coding standards below
3. **Test your changes** thoroughly
4. **Commit your changes** with clear, descriptive messages
5. **Push to your fork** and create a Pull Request

## Coding Standards

### Rust Code Style
- Follow the standard Rust formatting: `cargo fmt`
- Use `cargo clippy` to catch common issues
- Write documentation for public APIs
- Include unit tests for new functionality

### Error Handling
- Use `anyhow::Result` for most error handling
- Create specific error types in `types.rs` when appropriate
- Log errors appropriately with the `log` crate

### API Integration
- Cache API responses appropriately
- Handle rate limiting gracefully
- Provide meaningful error messages for API failures

### Documentation
- Update README.md for user-facing changes
- Update inline documentation for code changes
- Include examples in documentation where helpful

## Project Structure

```
src/
├── lib.rs              # Main plugin entry point and HexChat integration
├── hexchat.rs          # HexChat FFI bindings and utilities
├── edsm.rs             # EDSM API client with caching
├── jump_calculator.rs  # Core jump calculation logic
├── config.rs           # Configuration management
└── types.rs            # Shared data structures and types
```

## Testing

### Unit Tests
- Write unit tests for new functions and methods
- Test error conditions and edge cases
- Use `cargo test` to run all tests

### Integration Testing
- Test API integration with mock responses when possible
- Verify HexChat plugin loading (manual testing required)
- Test configuration file handling

### Manual Testing
- Test with real HexChat installation
- Verify RATSIGNAL message parsing
- Test with various system names and distances

## Submitting Changes

### Pull Request Process
1. **Update documentation** as needed
2. **Add/update tests** for your changes
3. **Ensure all tests pass**: `cargo test`
4. **Ensure code passes linting**: `cargo clippy`
5. **Format your code**: `cargo fmt`
6. **Write a clear PR description** explaining:
   - What changes you made
   - Why you made them
   - How to test them

### Pull Request Requirements
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is properly formatted
- [ ] Documentation is updated
- [ ] PR description clearly explains changes

## Issue Reporting

### Bug Reports
When reporting bugs, please include:
- **Operating system** and version
- **HexChat version**
- **Rust version** (`rustc --version`)
- **Steps to reproduce** the issue
- **Expected behavior**
- **Actual behavior**
- **Log output** (if applicable)

### Feature Requests
When requesting features, please include:
- **Clear description** of the feature
- **Use case** or motivation
- **Proposed implementation** (if you have ideas)
- **Potential impact** on existing functionality

## API Considerations

### EDSM API
- Respect rate limits (recommended: reasonable spacing between requests)
- Cache responses appropriately (default: 1 hour for system data, shorter for commander location)
- Handle API errors gracefully
- Follow EDSM's Terms of Service

### HexChat Plugin API
- Follow HexChat plugin conventions
- Handle memory management carefully in FFI code
- Test plugin loading/unloading
- Ensure thread safety where applicable

## Code Review Guidelines

### For Contributors
- Be open to feedback and suggestions
- Respond to review comments promptly
- Make requested changes in a timely manner
- Ask questions if review comments are unclear

### For Reviewers
- Be respectful and constructive
- Focus on code quality and maintainability
- Consider security implications
- Test changes when possible

## Release Process

1. **Version bumping** follows [Semantic Versioning](https://semver.org/)
2. **Changelog** is updated for each release
3. **Tags** are created for releases
4. **Release notes** summarize changes for users

## Questions and Support

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For general questions and ideas
- **Code Review**: Use PR comments for code-specific questions

## License

By contributing to EDJC, you agree that your contributions will be licensed under the MIT License.

## Acknowledgments

- Elite Dangerous community for inspiration and feedback
- Fuel Rats for the rescue service that inspired this plugin
- EDSM for providing the system data API
- HexChat for the plugin platform
