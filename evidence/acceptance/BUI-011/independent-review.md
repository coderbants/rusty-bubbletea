# BUI-011 Independent Review

Status: implementation self-review complete; protected pull-request review and
exact-head CI remain the independent merge gate.

## Scope

- `src/program.rs`: one-shot lifecycle state, `ProgramHandle`, configured
  renderer/input setup, startup messages, cancellation, panic recovery, render
  ticker shutdown, and graceful/error cleanup.
- `src/renderer.rs` and `src/cursed_renderer.rs`: direct protocol-output
  path for queries that must precede buffered renderer startup output.
- `src/options.rs`: explicit headless input sentinel and FPS normalization.
- `src/commands.rs`: no-op command filtering and singleton command behavior.
- `tests/tea_test.rs`: focused lifecycle, startup, cancellation, panic, FPS,
  command-shape and protocol-output-order regressions.
- `docs/src/lib.rs`: public lifecycle and headless-configuration guidance.
- `.github/workflows/ci.yml`: protected coverage reporting and trusted
  default-branch badge publication.

## Review checks

| Check | Result | Evidence |
| --- | --- | --- |
| Lifecycle has one runner, observable cleanup, graceful quit, kill, interruption, cancellation, and panic paths | Pass | `src/program.rs`; focused lifecycle tests |
| Headless execution avoids raw input setup and honors configured startup values | Pass | `tests/tea_test.rs::test_program_uses_configured_startup_contract` |
| Command no-op filtering preserves empty and singleton semantics | Pass | `tests/tea_test.rs::test_commands_and_messages` |
| Protocol queries precede buffered renderer startup output | Pass | `tests/tea_test.rs::test_protocol_query_precedes_buffered_renderer_startup_output`; targeted PTY parity |
| Documentation uses the current `<user-docs>` contract | Pass | `src/program.rs`, `src/options.rs`, `src/commands.rs`, `docs/src/lib.rs` |
| Focused Rust validation | Pass | `cargo check -p rusty-bubbletea --lib`; `cargo test -p rusty-bubbletea --test tea_test --no-fail-fast`; 14 tests passed |
| Reported protected parity failures | Pass locally | 14 reported examples matched Go traces after the protocol-output fix; the timing-sensitive `send-msg` trace matched across six repeated runs |
| Coverage workflow keeps pull-request candidate jobs read-only | Pass locally | Coverage report upload is push-only; badge commit/push is isolated to a trusted `dev` push job using `HEAD:dev` |

## Findings and limits

The first protected CI attempt exposed a protocol ordering defect in
`verify_examples`: synchronized-output and terminal-color queries were written into the renderer frame buffer after startup control
sequences. The renderer now has a direct protocol-output path, and the focused regression plus targeted PTY parity checks pass.
The second protected CI attempt (run 32947899448) then exposed a workflow
publication defect: coverage reached 74.04%, but the PR merge checkout had no
local `dev` ref and the badge step failed with `src refspec dev does not
match any`. Coverage now keeps `contents: read` for candidate pull-request
execution, uploads the report only on a trusted `dev` push, and performs badge
publication in a push-only job with `HEAD:dev` and `[skip ci]` to avoid
recursive validation. The corrected workflow awaits a new protected exact-head
run.
The worktree cannot run the untouched full dependency graph because the
`rusty-bubbles` dev dependency resolves its `../rusty-bubbletea` path to the
primary checkout, causing Cargo's package-collision error when the isolated
worktree is present. Focused validation therefore temporarily omitted that
dev-only dependency and restored the manifest immediately after each run; the
feature diff retains the original dependency line.

The manifest now declares `rust-version = "1.91"`, matching the workspace
toolchain policy and making the candidate's supported compiler floor explicit.

This record is implementation evidence, not an approval or merge
authorization. The final independent review, protected checks, and aggregate
acceptance remain owned by the repository and Mutate release gates.
