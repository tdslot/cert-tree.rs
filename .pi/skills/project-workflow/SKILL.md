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

**Prerequisites:** Clean main branch, changelog entry for the new version.

```bash
# 1. Bump version (updates Cargo.toml, README badge, CHANGELOG template)
just version-bump patch          # or minor / major

# 2. Edit CHANGELOG.md — fill in release notes under the new version heading

# 3. Commit the bump
git add -A
git commit -m "chore: bump version to v<version>"
git push origin main

# 4. Release — pre-flight checks (clean tree, main branch, changelog,
#    no existing tag) + confirmation prompt before creating signed tag
just release-github
```

`just release-github` depends on `prepare-release` (full QA) and **never rewrites existing tags**.

GitHub Actions then:
1. Validates tag ↔ `Cargo.toml` version match
2. Builds multi-platform binaries (Linux, macOS x86_64/ARM64, RPM, DEB)
3. **Fails if no changelog section** exists for the version
4. Creates a **draft release** — review artifacts, publish manually in GitHub UI

### Failed Release Recovery

If release CI fails after the tag was pushed, deleting the remote tag is the **only exception** to "never delete tags":

```bash
git push origin --delete v<version>
git tag -d v<version>
# Fix the issue, then run just release-github again
```

## Pitfalls

| Problem | Solution |
|---------|----------|
| Quality checks fail | Fix on feature branch, never force merge |
| Merge conflicts | `git merge --abort`, ask user for resolution |
| Uncommitted changes block branch | `git stash`, branch, `git stash pop` |
| Failed post-merge checks | `git reset --hard HEAD~1`, fix on branch, retry |
| Unclear version bump | Ask user: "Is this a feature (MINOR) or fix (PATCH)?" |
| Release CI fails after tag push | Delete remote tag (only exception), fix, re-run `just release-github` |
| Release appears as draft | Expected — review artifacts, publish manually in GitHub UI |

## Prohibited Without User Approval

- Creating release tags (always use `just release-github` on explicit request)
- Deleting or rewriting published tags (except failed-release recovery)
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
