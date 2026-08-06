---
name: project-workflow
description: Git workflow, versioning, and release process for cert-tree.rs. Use when making code changes, creating commits, managing versions, or preparing releases.
---

# Project Workflow

Git branching, conventional commits, semantic versioning, and release process for cert-tree.rs.

## When to Use

- Making code changes (features, fixes, refactoring)
- Creating commits or managing branches
- Determining version bumps
- Preparing and executing releases

## Prerequisites

- Working directory: clean or changes intentionally stashed
- Rust toolchain available (`cargo`, `just` in PATH)
- GPG key configured for signed tags (releases only)

## Procedure

### 1. Start Work

```bash
git status                                    # Verify clean state
git checkout -b <type>/<description>          # Create feature branch
```

Branch types and their version impact:

| Prefix | Use Case | Version |
|--------|----------|---------|
| `feature/` | New functionality | MINOR |
| `fix/` | Bug fixes | PATCH |
| `refactor/` | Code restructuring | PATCH |
| `docs/` | Documentation | PATCH |
| `chore/` | Maintenance | PATCH |

### 2. Develop

- Follow [AGENTS.md](../AGENTS.md) coding guidelines
- Write/update tests in `#[cfg(test)]` modules
- Run quality checks: `just quality` (fmt + clippy + test + build)

### 3. Update Version

Determine bump type from change:

| Change | Version | Example |
|--------|---------|---------|
| Breaking CLI/output changes | MAJOR | Remove `--format` option |
| New features, backward compatible | MINOR | Add `--json-pretty` |
| Bug fixes, performance, docs | PATCH | Fix parsing error |

Update these files:
1. `Cargo.toml` — `version` field
2. `CHANGELOG.md` — release notes entry
3. `README.md` — version badge (if present)

### 4. Commit

Format: `<type>[scope]: <description>`

| Type | Version | Example |
|------|---------|---------|
| `feat` | MINOR | `feat: add OCSP validation support` |
| `fix` | PATCH | `fix: correct certificate chain parsing` |
| `refactor` | PATCH | `refactor: simplify display logic` |
| `docs` | PATCH | `docs: update installation guide` |
| `chore` | PATCH | `chore: update dependencies` |

```bash
git add .
git commit -m "<type>: <description>"
```

### 5. Merge to Main

```bash
git checkout main
git pull origin main
git merge --no-ff <branch-name>
just quality-release              # Final verification
git push origin main
git branch -d <branch-name>
git push origin --delete <branch-name>
```

### 6. Release (User-Requested Only)

**Prerequisites:** Clean main branch, all quality checks pass.

```bash
# Verify
git status                        # Must be clean
just quality-release              # Must pass

# Update version (if not done in step 3)
# Edit Cargo.toml, CHANGELOG.md, README.md

git add .
git commit -m "chore: release v<version>"
git push origin main

# Create signed tag
git tag -s "v<version>" -m "Release v<version>"
git push origin "v<version>"
```

GitHub Actions builds multi-platform binaries automatically.

## Pitfalls

| Problem | Solution |
|---------|----------|
| Quality checks fail | Fix on feature branch, never force merge |
| Merge conflicts | `git merge --abort`, ask user for resolution |
| Uncommitted changes block branch | `git stash`, branch, `git stash pop` |
| Failed post-merge checks | `git reset --hard HEAD~1`, fix on branch, retry |
| Unclear version bump | Ask user: "Is this a feature (MINOR) or fix (PATCH)?" |

## Prohibited Without User Approval

- Creating release tags
- Force pushing to main
- Merging with failing quality checks
- Committing directly to main
- Skipping version updates for code changes

## Verification

After completing workflow, confirm:

- [ ] Branch created with correct prefix
- [ ] All commits follow conventional format
- [ ] Version bumped appropriately (MAJOR/MINOR/PATCH)
- [ ] `just quality` passes before merge
- [ ] `just quality-release` passes after merge
- [ ] CHANGELOG.md updated for user-facing changes
- [ ] Clean git history (no fixup commits)
