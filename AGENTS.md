# Agent Instructions for `charming-bubbletea`

> [!IMPORTANT]
> **Subsequent Cycle Requirement**: On every development cycle, before doing any work, the agent MUST inspect [`UPSTREAM_MAPPING.md`](file:///Users/jonny/Projects/charming/charming-bubbletea/UPSTREAM_MAPPING.md) to verify that all upstream Go files and examples are accounted for. When adding, modifying, or refactoring files, the agent MUST update [`UPSTREAM_MAPPING.md`](file:///Users/jonny/Projects/charming/charming-bubbletea/UPSTREAM_MAPPING.md) to reflect the current state.

## Core Rules & Workflow
1. Refer to the workspace-level rule in [`/Users/jonny/Projects/charming/AGENTS.md`](file:///Users/jonny/Projects/charming/AGENTS.md).
2. Maintain 100% rustdoc documentation.
3. Every ported file MUST include the guiding comment header:
   ```rust
   //! Cleanroom Rust port of upstream Go source file: `<upstream-go-filepath>`
   //! Upstream Target Tag / Version: `v2.0.8`
   ```
4. Verify all tests pass with `cargo test --all-targets` before committing.
