---
name: project-workflow
description: Complete development workflow for cert-tree.rs — git branching, conventional commits, semver versioning, quality gates, and release process. Use when making code changes, creating commits, managing versions, or preparing releases.
---

# Project Workflow for cert-tree.rs

Complete development workflow combining git management, versioning, and release processes.

## When to Use

- User requests a feature, fix, or change
- Creating commits or branches
- Determining version bumps
- Preparing releases
- Any git operation beyond simple status/log

---

## Part 1: Git Workflow

### Core Principles

1. **Feature Branch Workflow** — every change in dedicated branch
2. **Conventional Commits** — all commits follow conventional format
3. **Semver Compliance** — version updates follow semantic versioning
4. **Quality Gates** — code must pass `just quality` before merging
5. **Clean History** — maintain clear, linear Git history

### Automatic Workflow

#### 1. Task Initialization

```bash
git status

# Branch naming convention
git checkout -b feature/description   # new features (MINOR)
git checkout -b fix/description       # bug fixes (PATCH)
git checkout -b refactor/description  # refactoring (PATCH)
git checkout -b docs/description      # documentation (PATCH)
git checkout -b chore/description     # maintenance (PATCH)
```

**Communicate:** `🔀 Created new branch: feature/xyz`

#### 2. Development Phase

- Implement changes following AGENTS.md guidelines
- Write/update tests
- Run quality checks: `just quality` (fmt + clippy + test + build)
- Fix any failures before proceeding

#### 3. Version Management

Update version based on change type (see Semver section below).

Update files:
1. `Cargo.toml` — version field
2. `CHANGELOG.md` — release notes
3. `README.md` — version badge if present

#### 4. Commit Creation

Format: `<type>[optional scope]: <description>`

Types: `feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`, `perf`, `ci`

```bash
git add .
git commit -m "feat: add OCSP certificate validation support"
```

#### 5. Pre-Merge Quality

```bash
git checkout main
git pull origin main
git merge --no-ff feature/xyz
just quality-release
```

If checks fail → fix on feature branch → retry merge

#### 6. Push and Cleanup

```bash
git push origin main
git branch -d feature/xyz
git push origin --delete feature/xyz
```

---

## Part 2: Semantic Versioning

### Version Format

`MAJOR.MINOR.PATCH`

- **MAJOR** (X.y.z): Breaking changes requiring user intervention
- **MINOR** (x.Y.z): New features, backward compatible
- **PATCH** (x.y.Z): Bug fixes, backward compatible

### Version Increment Rules

| Change Type | Version | Examples |
|-------------|---------|----------|
| **MAJOR** | Breaking CLI changes, output format changes, feature removal | Changing `--tree` to `--format tree`, removing PEM support |
| **MINOR** | New features, CLI enhancements, display improvements | Adding `--json-pretty`, new X.509 extensions, TUI features |
| **PATCH** | Bug fixes, performance, security, documentation | Fixing parsing errors, memory optimization, README updates |

### Commit Message → Version Mapping

- `fix: correct parsing bug` → **PATCH**
- `feat: add XML output` → **MINOR**
- `BREAKING: remove legacy options` → **MAJOR**

### Pre-release Versions

- **Alpha** (`1.0.0-alpha.1`): Early testing, features may change
- **Beta** (`1.0.0-beta.1`): Feature complete, testing phase
- **RC** (`1.0.0-rc.1`): Ready for release, final testing

---

## Part 3: Release Process

**Only execute when user explicitly requests release.**

### Pre-release Checklist

```bash
git status  # Should be clean
just quality-release  # fmt + clippy + test + build-release
```

### Release Steps

1. **Version confirmation** — verify Cargo.toml version is correct
2. **Changelog review** — ensure CHANGELOG.md has entry for this version
3. **Commit release** (if needed):
   ```bash
   git add .
   git commit -m "chore: release v{version}"
   git push origin main
   ```
4. **Create and push tag**:
   ```bash
   git tag -s "v{version}" -m "Release v{version}"
   git push origin "v{version}"
   ```
5. **GitHub Actions** builds and publishes automatically

### Release Communication

```
🎉 Release v{version} process started
🤖 GitHub Actions building packages...
📦 Packages available in releases page in a few minutes
```

---

## Communication Guidelines

**Always inform user about:**
- Branch creation: `🔀 Created branch: feature/xyz`
- Quality checks: `✅ All quality checks passed`
- Version updates: `📦 Version updated to v{version} ({type})`
- Commits: `✅ Commit: {type}: {description}`
- Merge: `🔀 Changes merged to main`
- Issues: `⚠️ Error: {issue}, fixing...`

**Always ask user when:**
- Breaking changes (MAJOR version bump)
- Ready to trigger release
- Merge conflicts need resolution
- Unclear if feature or fix

## Prohibited Actions (WITHOUT USER APPROVAL)

❌ Never create release tags without explicit request
❌ Never force push to main
❌ Never merge failing quality checks
❌ Never commit directly to main
❌ Never skip version updates when code changes

## Emergency Procedures

**Uncommitted changes conflict:**
```bash
git stash
git checkout -b fix/emergency
git stash pop
```

**Merge conflict:**
```bash
git merge --abort
# Report to user, get resolution instructions
```

**Failed checks after merge:**
```bash
git reset --hard HEAD~1
git checkout feature/branch
# Fix, retry
```

## Success Criteria

✅ Every change in feature branch
✅ Every commit follows conventional format
✅ Version updated for every code change
✅ All quality checks pass before merging
✅ Clean Git history
✅ User informed at every step
