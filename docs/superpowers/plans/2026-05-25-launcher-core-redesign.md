# Launcher Core Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the Rust launcher library around a maintainable facade, typed core model, safe IO/download pipeline, and Fabric/Quilt/Forge/NeoForge loader support.

**Architecture:** The plan is split into four phase files so each execution pass stays reviewable. Execute them in order; each phase contains bite-sized TDD steps, verification commands, and commit instructions.

**Tech Stack:** Rust 2021, `reqwest` blocking client, `serde`, `serde_json`, `thiserror`, `once_cell`, RustCrypto `sha1`/`sha2`/`digest`, `zip`, `tempfile`, Cargo integration tests.

---

## Phase Files

1. [Phase 1: Foundation And Core](/Users/star/Documents/coding/codex/mc-launcher-core/docs/superpowers/plans/2026-05-25-launcher-core-redesign-01-foundation-core.md)
   Modernize dependencies, restore cross-platform compilation, and add typed core primitives.

2. [Phase 2: Command And Install](/Users/star/Documents/coding/codex/mc-launcher-core/docs/superpowers/plans/2026-05-25-launcher-core-redesign-02-command-install.md)
   Build command generation, download planning, safe extraction, vanilla install planning, and the `Launcher` facade.

3. [Phase 3: Loader Providers](/Users/star/Documents/coding/codex/mc-launcher-core/docs/superpowers/plans/2026-05-25-launcher-core-redesign-03-loaders.md)
   Add Fabric, Quilt, Forge, and NeoForge metadata/profile provider support and loader profile installation.

4. [Phase 4: Compatibility, Docs, And Verification](/Users/star/Documents/coding/codex/mc-launcher-core/docs/superpowers/plans/2026-05-25-launcher-core-redesign-04-compat-docs.md)
   Restore documented legacy wrappers, migrate auth hashing, update examples, and complete verification.

## Scope Check

The approved spec covers several layers, but they form one migration path for the launcher SDK. The split phase files keep the work navigable while preserving the dependency order:

- Phase 1 must land before any new command/install code.
- Phase 2 creates the facade and vanilla planning.
- Phase 3 adds loader-specific support on top of the facade.
- Phase 4 handles migration polish, examples, and final verification.

MRPack remains outside this plan.

## Execution Guidance

Use `superpowers:subagent-driven-development` for task-by-task execution when possible. If executing inline, use `superpowers:executing-plans` and treat each phase file as a checkpoint boundary.

After all phases, run:

```bash
cargo fmt -- --check
cargo test
cargo test --examples
cargo test --test live_metadata -- --ignored
git status --short
```

Expected: formatting passes, all default tests and examples pass, live metadata tests pass with network available, and the working tree is clean after the final commit.
