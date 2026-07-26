---
name: develop-mc-launcher-core
description: Manage the mc-launcher-core Git development and release workflow. Use when starting or implementing a feature, fix, refactor, documentation, test, CI, performance, or maintenance change; creating a work branch; integrating verified work into dev; synchronizing dev and main; or preparing and publishing a user-approved version with its Git tag.
---

# Develop mc-launcher-core

Follow a `dev`-first workflow. Keep routine work off `dev` and `main`, integrate
verified changes into `dev`, and release to `main` only after the user explicitly
confirms the exact version.

## Protect the repository

- Inspect `git status --short --branch`, branches, remotes, and relevant worktrees
  before changing branches.
- Preserve uncommitted or unrelated user changes. Never discard, overwrite, stash,
  or include them in a commit without the user's authorization.
- Fetch remote state before creating, integrating, or releasing a branch.
- Use fast-forward-only pulls. Never force-push, rewrite published history, move an
  existing tag, or delete a branch or tag unless the user explicitly requests it.
- Stop and report unexpected divergence, conflicts, failed checks, or a version/tag
  that already exists. Do not bypass checks to complete the workflow.
- Use non-interactive Git commands and show the resulting branch, commit, and remote
  state after every integration or release.

## Choose the work branch

Create routine branches from the latest `origin/dev`. Select the narrowest prefix:

- `feat/` for user-visible capabilities
- `fix/` for defects
- `refactor/` for behavior-preserving code restructuring
- `perf/` for performance work
- `test/` for test-only changes
- `docs/` for documentation-only changes
- `ci/` for automation and workflow changes
- `chore/` for maintenance that fits no narrower prefix

Use a short kebab-case description, such as `feat/modpack-install` or
`fix/forge-classpath`. Honor an explicit valid branch name from the user. Do not use
`release/` for routine work.

Create a new routine branch with this sequence:

```bash
git fetch --prune origin
git switch dev
git pull --ff-only origin dev
git switch -c <type>/<short-kebab-description>
```

If the branch already exists, inspect it and reuse it only when that matches the
user's intent.

## Develop and verify

Implement only the requested scope. Keep commits reviewable and use Conventional
Commit subjects matching the change, for example `feat: add modpack import` or
`fix: resolve forge classpath`.

Run checks appropriate to the change. Before integrating code into `dev`, run the
full local gate from the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
```

Also run focused tests during development. For documentation-only or CI-only changes,
run every applicable check and explain any intentionally skipped command. Do not run
ignored live/network tests unless the user requests them or the change specifically
depends on them.

## Integrate into dev

Treat an explicit request to merge into `dev`, or the user's confirmation that the
verified work is ready, as integration approval. Otherwise leave the work branch
unmerged and report its status.

Before integration:

1. Require a clean worktree and committed intended changes.
2. Fetch `origin` and confirm the work branch descends from `dev`.
3. Incorporate the latest `origin/dev` into the work branch and resolve any conflict
   on the work branch, never directly on `dev`.
4. Rerun the full local gate.
5. Summarize the commits and successful checks.

Then integrate and push:

```bash
git switch dev
git pull --ff-only origin dev
git merge --no-ff <work-branch>
git push origin dev
```

Keep the work branch unless the user asks to delete it. Verify that local `dev` and
`origin/dev` resolve to the same commit.

## Release an official version

Treat release as a protected operation. Proceed only when the user explicitly says
to publish/release and confirms one exact SemVer version, such as `0.2.0` or
`0.2.0-rc.1`. A suggested, inferred, or merely discussed version is not approval.
Use the package version without `v` and the tag as `v<version>`.

Pushing `main` triggers `.github/workflows/publish-crate.yml`, which can publish the
crate to crates.io. State this consequence before making the release push.

### Prepare the release

1. Fetch `origin` and require clean, non-divergent `dev` and `main` histories.
2. Confirm that the version is valid SemVer and greater than the current
   `Cargo.toml` package version.
3. Confirm that neither local nor remote tag `v<version>` exists.
4. Create `release/v<version>` from the latest `origin/dev`.
5. Set `[package].version` in `Cargo.toml` to `<version>`.
6. Refresh `Cargo.lock` with Cargo and verify that its root package version matches.
7. Update README or other current-version references that are intended to show the
   latest release. Preserve historical version references.
8. Commit the release preparation as `release: prepare v<version>`.
9. Run the full local gate plus:

```bash
cargo publish --dry-run --locked
```

Do not merge or publish when any release check fails.

### Promote dev to main

Fast-forward the verified release branch into `dev`, push `dev`, and confirm
`origin/dev` points at the release commit:

```bash
git switch dev
git pull --ff-only origin dev
git merge --ff-only release/v<version>
git push origin dev
```

If `dev` moved after the release checks, update the release branch from the new
`dev`, rerun the release checks, and retry. Then promote `dev`:

```bash
git switch main
git pull --ff-only origin main
git merge --ff-only dev
```

Require `origin/main` to be an ancestor of `dev`. If a fast-forward is impossible,
stop and report the divergent commits instead of creating an unreviewed release
merge.

Confirm immediately before tagging that:

- `Cargo.toml` and `Cargo.lock` contain `<version>`;
- `HEAD`, local `dev`, and the intended release commit are identical;
- the full checks and publish dry run passed;
- tag `v<version>` still does not exist locally or remotely.

Create an annotated tag on `main` and atomically push the branch and tag:

```bash
git tag -a v<version> -m "Release v<version>"
git push --atomic origin main refs/tags/v<version>
```

Verify `origin/main`, `origin/dev`, local `main`, local `dev`, and
`v<version>` resolve to the intended release commit. Report the commit, tag,
checks, and push result. Do not claim crates.io publication succeeded unless it
was separately verified.
