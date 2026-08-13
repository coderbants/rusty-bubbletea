# Port Upstream Skill & Workflow

This document defines the 4-phase synchronization workflow for maintaining cleanroom parity between `rusty-bubbletea` and upstream `charmbracelet/bubbletea`.

## Upstream Target Repository
- Repository: `charmbracelet/bubbletea`
- Upstream Tag Target: `v1.3.4`
- Local Checkout Location: `upstream-go/` (ignored by `.gitignore`)

## 4-Phase Synchronization Workflow

### Phase A: Fetch & Update
1. Clone or fetch the latest tags from `charmbracelet/bubbletea` into `upstream-go/`.
2. Checkout the desired release tag (e.g., `git checkout v1.3.4`).
3. Verify directory contents match upstream Go source declarations.

### Phase B: Diff Analysis
1. Calculate structural and code diffs between previous target tag and new release tag:
   ```bash
   git diff v1.3.3..v1.3.4 -- '*.go'
   ```
2. Identify new, modified, or deprecated types, interfaces, functions, commands, or events.

### Phase C: Mechanical Porting
1. Map Go source files 1:1 to Rust modules:
   - `tea.go` -> `src/lib.rs`
   - `model.go` -> `src/model.rs`
   - `program.go` -> `src/program.rs`
   - `commands.go` -> `src/commands.rs`
   - `key.go` -> `src/key.rs`
   - `mouse.go` -> `src/mouse.rs`
2. Preserve comment invariants:
   - Tag all ported Go doc comments with `<upstream-comment>...</upstream-comment>`.
   - Include `<public-docs>...</public-docs>` blocks on user-facing modules.
   - Enforce 100% rustdoc documentation across all public symbols.
3. Convert Go `*_test.go` suites directly to Rust `#[test]` modules under `tests/`.

### Phase D: Versioning & Tagging
1. Update `version` in `Cargo.toml` to match upstream target version.
2. Verify all unit and integration tests pass:
   ```bash
   cargo test
   ```
3. Commit ported changes, tag the git commit with the matching upstream version (e.g. `v1.3.4`), and push back to remote repository.
