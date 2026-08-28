# Contributing to `rusty-bubbletea`

Thanks for your interest in contributing! `rusty-bubbletea` is a cleanroom Rust port of
the upstream Go [charmbracelet/bubbletea](https://github.com/charmbracelet/bubbletea) TUI
framework (The Elm Architecture for Go), pinned to upstream tag `v2.0.8`.

Please read the workspace rules in [`AGENTS.md`](AGENTS.md) (and the root
[`AGENTS.md`](../AGENTS.md)) before contributing. This file summarizes the practical
workflow.

## Development setup

- Rust 1.98.0, selected automatically by the checked-in `rust-toolchain.toml`.
- Go (for the upstream parity scripts and the pinned `upstream-go/` checkout).
- No other system dependencies; there are no C build steps.

```sh
cargo build --all-targets
cargo test --all-targets
```

## Repository layout

- `src/` — the ported crate. Every public symbol has rustdoc; every module mirrors an
  upstream Go file.
- `examples/` — executable Rust ports of upstream Go examples.
- `tests/` — Rust integration tests ported from upstream `*_test.go` suites.
- `upstream-go/` — the pinned upstream Go checkout (git-ignored, never commit it).
- `scripts/` — parity, mapping, and toolchain consistency verification helpers.
- `UPSTREAM_MAPPING.md` — the authoritative 1:1 account of every upstream file.

## The cleanroom porting workflow

1. **Upstream sync (Phase A/B).** New upstream releases are fetched into `upstream-go/`
   pinned to the target tag. Diff the new release against the previous one with
   `git diff vA.B.C..vX.Y.Z -- '*.go'` inside `upstream-go/` and update
   `UPSTREAM_MAPPING.md` so every upstream file (source, tests, examples, docs, support
   files) stays accounted for.

2. **Mechanical porting (Phase C).** Port Go source to Rust modules, Go `*_test.go`
   suites to `tests/`, and Go example programs to `examples/`. Every ported file MUST
   start with the header:

   ```rust
   //! Cleanroom Rust port of upstream Go source file: `<upstream-go-filepath>`
   //! Upstream Target Tag / Version: `v2.0.8`
   ```

3. **Comment invariants.** Tag doc comments ported directly from Go with
   `<upstream-comment>...</upstream-comment>`, include `<public-docs>...</public-docs>`
   blocks on user-facing modules, and prefer borrowing (`&str`, `&[T]`) over allocation
   (`Arc`, `Rc`). Maintain 100% rustdoc coverage: `cargo doc --no-deps --all-features`
   must emit no warnings.

4. **Verification.** Before committing:

   ```sh
   cargo test --all-targets
   ./scripts/verify_mapping.sh   # upstream file accounting
   ./scripts/verify_examples.sh  # example parity (upstream Go vs Rust)
   cargo doc --no-deps           # rustdoc coverage
   ```

   Interactive example behavior (key handling, rendering) is exercised with the pty
   driver: `python3 scripts/pty_driver.py --cmd target/debug/examples/<name> ...`.

## Releases

- GitHub releases must match upstream: every tracked upstream release tag must exist as a
  `v*` tag and a GitHub release on this repo.
- To release: `git tag v2.0.8 && git push origin v2.0.8`. The
  [publish workflow](.github/workflows/publish.yml) runs tests and example parity,
  creates the GitHub release, and attempts the crates.io publish (non-fatal without a
  registry token).
- `dev` branch pushes run tests and parity only.

## Versioning

Every release uses the tracked upstream `MAJOR.MINOR.PATCH` version exactly. The crate version,
release tag, and GitHub release must all match the upstream version; there is no fourth-part
iteration or republishing of an existing crates.io version. If upstream has not advanced beyond
the currently published version, do not create a release—wait for the next upstream release.

## Contribution guidelines

- Keep the 1:1 file mapping intact — do not add or remove modules without updating
  `UPSTREAM_MAPPING.md`.
- Match the upstream file layout: a change to an upstream Go file lands in the
  corresponding Rust module.
- Commit messages should describe the upstream behaviour being ported or fixed, e.g.
  `port textinput cursor blinking` or `fix: paginator handles rapid arrow presses`.
- Follow the style of the surrounding code; there are no external formatter
  dependencies beyond `cargo fmt` defaults.

## Reporting issues

- Describe the upstream Go behaviour expected and the Rust behaviour observed.
- Include the terminal emulator and `TERM` value when the issue is input/render related.
- Note the pinned upstream tag (`v2.0.8`) in the report.

## License

[MIT](LICENSE) — same as the upstream project.
