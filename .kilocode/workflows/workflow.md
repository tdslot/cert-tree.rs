# Software Development Workflow Rules

This document outlines the comprehensive workflow guidelines for the cert-tree.rs project, incorporating industry best practices for Rust development, version control, testing, and release management.

## Core Principles

### 1. Memory Bank First
- **Always read ALL memory bank files** at the start of every task
- Update memory bank files when significant changes occur
- Use memory bank as the single source of truth for project knowledge

### 2. Semantic Versioning
- Follow [semver rules](semver.md) for all version increments
- Update `Cargo.toml` version according to semver guidelines
- Update `CHANGELOG.md` with detailed release notes
- Use conventional commit messages with semver impact indicators

### 3. Commit Message Generation
- Use the [commit message generator](commit-message-generator-with-gitmojis.md) for all commits
- Generate conventional commit messages with gitmojis based on staged changes
- Follow the gitmoji ↔ conventional commits mapping for appropriate emoji and type selection
- Include scope when applicable (api, ui, auth, db, config, deps, docs)
- Keep descriptions imperative, concise, and under 50 characters
- Add body for complex changes explaining what and why
- Include breaking change footers when applicable

### 3. Justfile Automation
- Use justfile recipes for all repetitive development tasks
- Prefer justfile commands over manual CLI invocations
- Keep justfile updated with new workflows and recipes

## Development Workflow

### Daily Development Cycle

1. **Start of Day**
   ```bash
   just status          # Check git status
   just pull           # Pull latest changes
   just check          # Verify code compiles
   ```

2. **Code Changes**
   ```bash
   just build          # Build debug version
   just test           # Run test suite
   just fmt            # Format code
   just clippy         # Run linter
   ```

3. **Quality Assurance**
   ```bash
   just quality        # Full quality check (fmt + clippy + test + build)
   just run-test-cert  # Test with sample certificate
   ```

4. **End of Day**
   ```bash
   just status         # Review changes
   # Generate commit message using commit-message-generator-with-gitmojis.md
   just commit "<generated-message>"  # Commit with generated conventional message
   just push           # Push changes
   ```

### Feature Development

1. **Planning**
   - Read relevant memory bank files
   - Plan implementation in context.md
   - Identify affected files and components

2. **Implementation**
   ```bash
   just dev-check      # Quick development checks
   just test-verbose   # Run tests with output
   just run-test-cert-tui  # Test TUI functionality
   ```

3. **Code Review**
   - Self-review code changes
   - Run full quality suite: `just quality-release`
   - Test edge cases and error conditions

4. **Documentation**
   - Update inline code documentation
   - Update README.md if user-facing changes
   - Update memory bank files

## Code Quality Standards

### Rust Best Practices

1. **Memory Safety**
   - Leverage Rust's ownership system
   - Avoid unsafe code unless absolutely necessary
   - Use references over owned values when possible

2. **Performance**
   - Follow [Rust performance guidelines](rust-performance-guidelines.md)
   - Profile with `just profile`
   - Optimize hot paths identified by profiling

3. **Error Handling**
   - Use `Result<T, E>` for recoverable errors
   - Implement proper error types with `thiserror`
   - Provide meaningful error messages

### Code Style

1. **Formatting**
   - Use `rustfmt` via `just fmt`
   - Follow standard Rust formatting conventions

2. **Linting**
   - Run `clippy` via `just clippy`
   - Fix all warnings before committing
   - Configure clippy for project-specific rules

3. **Documentation**
   - Document public APIs with `///` comments
   - Use `cargo doc` via `just doc` to generate docs
   - Keep documentation up-to-date

## Testing Strategy

### Unit Testing

1. **Test Coverage**
   - Write unit tests for all public functions
   - Test edge cases and error conditions
   - Use `cargo test` via `just test`

2. **Test Organization**
   - Place unit tests in same file as implementation
   - Use descriptive test names
   - Test both success and failure paths

### Integration Testing

1. **CLI Testing**
   - Test command-line interface with various inputs
   - Verify help output: `just help`
   - Test with real certificate files

2. **Certificate Parsing**
   - Test with various certificate formats (PEM, DER)
   - Test certificate chain parsing
   - Verify error handling for malformed certificates

### Test Automation

```bash
just test            # Run all tests
just test-verbose    # Run tests with output
just run-test-cert   # Test with sample certificate
just run-test-cert-text  # Test text output mode
just run-test-cert-tui   # Test TUI mode
```

## Version Control

### Git Workflow

1. **Branching Strategy**
   - Use feature branches for new development
   - Keep main branch stable and releasable
   - Use descriptive branch names

2. **Commit Standards**
   - Use the [commit message generator](commit-message-generator-with-gitmojis.md) to create conventional commits with gitmojis
   - Follow gitmoji ↔ conventional commits mapping for appropriate emoji and type selection
   - Include scope when applicable and keep descriptions imperative and concise
   - Reference issue numbers when applicable
   - Keep commits focused and atomic

3. **Pull Requests**
   - Create PRs for significant changes
   - Include description of changes
   - Request review from team members

### Git Automation

```bash
just status          # Check git status
just log            # View recent commits
# Use commit message generator for proper conventional commits with gitmojis
just commit "<generated-message>"  # Create commit with generated message
just push           # Push changes
just pull           # Pull changes
```

## Release Process

### Pre-Release Checklist

1. **Code Quality**
   ```bash
   just quality-release  # Full quality check for release
   just prepare-release  # Prepare for release
   ```

2. **Version Update**
   - Update version in `Cargo.toml` per semver rules
   - Update `CHANGELOG.md` with release notes
   - Commit version changes

3. **Testing**
   ```bash
   just run-test-cert-release  # Test release binary
   just version               # Verify version
   ```

### Release Steps

1. **Build Release**
   ```bash
   just build-release       # Build optimized binary
   just release-archive     # Create release archive
   just release            # Full release workflow
   ```

2. **Distribution**
   - Create git tag: `just tag v1.0.0 "Release v1.0.0"`
   - Upload release archive to GitHub Releases
   - Update documentation with new version

3. **Post-Release**
   - Update memory bank context.md
   - Announce release to community
   - Monitor for issues

## Dependency Management

### Regular Maintenance

1. **Updates**
   ```bash
   just outdated        # Check for outdated dependencies
   just update-deps     # Update dependencies
   just audit          # Security audit
   ```

2. **Security**
   - Run security audits regularly
   - Update dependencies promptly for security fixes
   - Review dependency changes in PRs

## Documentation

### Types of Documentation

1. **Code Documentation**
   - Inline documentation with `///`
   - Function and struct documentation
   - Example usage in doc comments

2. **User Documentation**
   - README.md for installation and usage
   - CHANGELOG.md for version history
   - API documentation via `cargo doc`

3. **Internal Documentation**
   - Memory bank files for project knowledge
   - Workflow documentation (this file)
   - Architecture documentation

### Documentation Automation

```bash
just doc             # Generate and open documentation
just doc-build       # Generate documentation
just doc-check       # Check documentation
```

## Continuous Integration

### CI Pipeline

1. **Automated Checks**
   - Code formatting check
   - Linting with clippy
   - Unit and integration tests
   - Build verification

2. **Cross-Platform Testing**
   - Test on Linux, macOS, Windows
   - Use cross-compilation: `just build-all`

3. **Release Automation**
   - Automated release builds
   - Binary distribution
   - Documentation deployment

## Security Practices

### Code Security

1. **Input Validation**
   - Validate all user inputs
   - Handle malformed certificates gracefully
   - Use safe parsing libraries

2. **Dependency Security**
   ```bash
   just audit          # Regular security audits
   just outdated       # Check for vulnerable versions
   ```

3. **Memory Safety**
   - Rely on Rust's memory safety guarantees
   - Avoid unsafe code blocks
   - Regular code reviews for security

## Performance Monitoring

### Performance Guidelines

1. **Benchmarking**
   ```bash
   just bench          # Run benchmarks
   just profile        # Profile binary size and performance
   ```

2. **Optimization**
   - Profile before optimizing
   - Focus on hot paths
   - Measure improvements quantitatively

3. **Memory Usage**
   - Monitor memory consumption
   - Optimize data structures
   - Use efficient algorithms

## Team Collaboration

### Communication

1. **Code Reviews**
   - Review all changes before merging
   - Use constructive feedback
   - Share knowledge through reviews

2. **Knowledge Sharing**
   - Update memory bank with new learnings
   - Document complex implementations
   - Share best practices

3. **Issue Tracking**
   - Use GitHub issues for bugs and features
   - Reference issues in commits
   - Keep issues updated

## Emergency Procedures

### Rollback Process

1. **Identify Issue**
   - Monitor for critical bugs
   - Check user reports

2. **Rollback Steps**
   ```bash
   git revert <commit>   # Revert problematic commit
   just build-release    # Build fixed version
   just release         # Deploy hotfix
   ```

3. **Post-Mortem**
   - Analyze root cause
   - Update processes to prevent recurrence
   - Document lessons learned

## Tooling and Automation

### Essential Tools

1. **Development Tools**
   ```bash
   just setup           # Setup development environment
   just install-tools   # Install additional tools
   ```

2. **Environment Management**
   ```bash
   just env             # Show environment variables
   just sysinfo         # Show system information
   ```

3. **Backup and Recovery**
   ```bash
   just backup          # Create backup of important files
   just emergency-clean # Emergency cleanup
   ```

## Continuous Improvement

### Process Refinement

1. **Regular Reviews**
   - Review workflow effectiveness quarterly
   - Update processes based on lessons learned
   - Incorporate new best practices

2. **Tool Updates**
   - Keep development tools current
   - Update justfile with new recipes
   - Maintain CI/CD pipelines

3. **Knowledge Management**
   - Keep memory bank current
   - Document new patterns and practices
   - Share improvements with team

## Compliance and Standards

### Industry Standards

1. **Rust Ecosystem**
   - Follow Rust API guidelines
   - Use standard library patterns
   - Contribute to community crates

2. **Security Standards**
   - Follow OWASP guidelines
   - Implement secure coding practices
   - Regular security training

3. **Open Source**
   - Respect licenses
   - Follow contribution guidelines
   - Maintain professional standards

This workflow ensures consistent, high-quality development while leveraging automation through justfile recipes to streamline the development process.