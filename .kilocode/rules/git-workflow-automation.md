# Git Workflow Automation Rules

## Overview
This rule defines automatic Git workflow management for cert-tree.rs project, ensuring best practices are followed without requiring manual reminders. The AI assistant must automatically manage branches, commits, and releases according to established patterns.

## Core Principles

1. **Feature Branch Workflow**: Every change happens in a dedicated branch
2. **Conventional Commits**: All commits follow conventional commit format
3. **Semver Compliance**: Version updates follow semantic versioning rules
4. **Quality Gates**: Code must pass all checks before merging
5. **Clean History**: Maintain clear, linear Git history

## Automatic Workflow Steps

### 1. Task Initialization (AUTOMATIC)

When user requests a feature, fix, or change:

**Step 1a: Check current Git status**
```bash
git status
```

**Step 1b: Create feature branch**
- Branch naming convention:
  - `feature/description` - for new features (MINOR version)
  - `fix/description` - for bug fixes (PATCH version)
  - `refactor/description` - for code refactoring (PATCH version)
  - `docs/description` - for documentation updates (PATCH version)
  - `chore/description` - for maintenance tasks (PATCH version)

```bash
# Example
git checkout -b feature/add-ocsp-support
git checkout -b fix/certificate-parsing-error
```

**Step 1c: Communicate to user**
```
🔀 Created new branch: feature/add-ocsp-support
📋 Starting work on this functionality...
```

### 2. Development Phase (AUTOMATIC)

**Step 2a: Make changes**
- Implement the requested functionality
- Follow Rust guidelines
- Write/update tests
- Update documentation

**Step 2b: Run quality checks**
```bash
just quality
# Equivalent to: fmt clippy test build
```

**Step 2c: Verify all checks pass**
- If checks fail, fix issues before proceeding
- Report issues to user if assistance needed

### 3. Version Management (AUTOMATIC)

**Step 3a: Determine version bump**
Based on change type:
- **MAJOR**: Breaking changes (rarely used)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes, refactoring, docs

**Step 3b: Update version files**
1. `Cargo.toml` - update version field
2. `CHANGELOG.md` - add release notes
3. `README.md` - update version badge if present

**Step 3c: Update memory bank**
```
.kilocode/rules/memory-bank/context.md
```

### 4. Commit Creation (AUTOMATIC)

**Step 4a: Stage changes**
```bash
git add .
```

**Step 4b: Create conventional commit**
Format: `<type>[optional scope]: <description>`

Types:
- `feat`: New feature (MINOR)
- `fix`: Bug fix (PATCH)
- `refactor`: Code refactoring (PATCH)
- `docs`: Documentation changes (PATCH)
- `style`: Code style/formatting (PATCH)
- `test`: Test updates (PATCH)
- `chore`: Maintenance tasks (PATCH)
- `perf`: Performance improvements (PATCH)
- `ci`: CI/CD changes (PATCH)

**Examples:**
```bash
git commit -m "feat: add OCSP certificate validation support"
git commit -m "fix: correct certificate chain parsing for edge cases"
git commit -m "refactor: modularize certificate parsing logic"
git commit -m "docs: update installation instructions"
```

**Step 4c: Communicate commit**
```
✅ Commit created: feat: add OCSP certificate validation support
```

### 5. Pre-Merge Quality (AUTOMATIC)

**Step 5a: Switch to main branch**
```bash
git checkout main
git pull origin main
```

**Step 5b: Merge feature branch**
```bash
git merge --no-ff feature/add-ocsp-support
```

**Step 5c: Run final quality checks**
```bash
just quality-release
```

**Step 5d: If checks fail**
- Switch back to feature branch
- Fix issues
- Repeat merge process

### 6. Push and Cleanup (AUTOMATIC)

**Step 6a: Push to remote**
```bash
git push origin main
```

**Step 6b: Delete feature branch**
```bash
git branch -d feature/add-ocsp-support
git push origin --delete feature/add-ocsp-support
```

**Step 6c: Communicate completion**
```
🚀 Changes successfully merged to main branch
🧹 Feature branch deleted
📦 Version updated to v0.15.0
```

### 7. Release Process (WHEN READY)

**Only execute when user explicitly requests release or confirms readiness**

**Step 7a: Verify everything is committed**
```bash
git status
# Should show clean working tree
```

**Step 7b: Create release via Justfile**
```bash
just release-github
```

This will:
1. Stage and commit release workflow updates
2. Push changes
3. Create GPG-signed tag
4. Push tag to trigger GitHub Actions
5. Automated build and release via CI/CD

**Step 7c: Monitor release**
```
🎉 Release v0.15.0 process started
🤖 GitHub Actions building packages...
📦 Packages will be available in releases page in a few minutes
```

## Workflow Decision Tree

```
User Request
    ↓
Is Git clean?
    ↓ Yes              ↓ No
Create branch      Commit/stash existing changes
    ↓
Make changes
    ↓
Run quality checks
    ↓
Pass? → Yes → Determine version → Update files
    ↓ No
Fix issues → Retry
    ↓
Create commit
    ↓
Merge to main
    ↓
Run final checks
    ↓
Pass? → Yes → Push → Cleanup
    ↓ No
Revert merge → Fix → Retry
    ↓
DONE (or proceed to release if requested)
```

## Communication Guidelines

### Always inform user about:
1. **Branch creation**: "🔀 Created branch: feature/xyz"
2. **Quality checks**: "✅ All quality checks passed successfully"
3. **Version updates**: "📦 Version updated to v0.15.0 (MINOR version)"
4. **Commits**: "✅ Commit: feat: add new feature"
5. **Merge**: "🔀 Changes merged to main"
6. **Issues**: "⚠️ Error: tests failed, fixing..."

### Always ask user when:
1. **Breaking changes**: "Do you want to proceed with MAJOR version bump? (Breaking change)"
2. **Release**: "Do you want to trigger the release process now?"
3. **Conflicts**: "There are merge conflicts. How would you like to resolve them?"
4. **Uncertainty**: "Not sure if this is a feature or fix. How should we classify it?"

## Prohibited Actions (WITHOUT USER APPROVAL)

❌ **Never** create release tags without explicit user request
❌ **Never** force push to main branch
❌ **Never** merge failing quality checks
❌ **Never** commit directly to main (always use feature branches)
❌ **Never** skip version updates when code changes

## Integration with Existing Rules

This workflow integrates with:
- `semver.md` - Version numbering rules
- `memory-bank/` - Context tracking
- `rust-guidelines.md` - Code quality standards
- `Justfile` - Build and test automation

## Example Full Workflow

```
User: "Add OCSP certificate validation support"

AI:
1. 🔀 Created branch: feature/add-ocsp-support
2. 📝 Implementing OCSP validation functionality...
   [makes changes to code]
3. ✅ Running quality checks...
   - cargo fmt ✓
   - cargo clippy ✓
   - cargo test ✓
   - cargo build ✓
4. 📦 Updating version:
   - Cargo.toml: 0.14.8 → 0.15.0 (MINOR - new feature)
   - CHANGELOG.md: added release notes
   - Memory bank: updated context
5. ✅ Commit: feat: add OCSP certificate validation support
6. 🔀 Merged changes to main branch
7. ✅ Final quality checks passed
8. 🚀 Pushed to remote
9. 🧹 Feature branch deleted

✅ All done! Version is now v0.15.0
   
Would you like to trigger the release process (just release-github)?
```

## Emergency Procedures

### If something goes wrong:

**Uncommitted changes conflict:**
```bash
git stash
git checkout -b fix/emergency-fix
git stash pop
# Continue workflow
```

**Merge conflict:**
```bash
git merge --abort
# Report to user, get resolution instructions
```

**Failed quality checks after merge:**
```bash
git reset --hard HEAD~1  # Undo merge
git checkout feature/branch
# Fix issues
# Retry merge
```

## Success Criteria

✅ Every change is in a feature branch
✅ Every commit follows conventional format
✅ Version is updated for every code change
✅ All quality checks pass before merging
✅ Clean Git history maintained
✅ User is informed at every step
✅ Memory bank stays synchronized

## Automation Triggers

Execute workflow automatically when user:
- Requests new feature
- Reports bug to fix
- Asks for refactoring
- Requests documentation update
- Asks for code changes

Do NOT execute when user:
- Asks questions only
- Requests information
- Wants to discuss design
- Reviews existing code