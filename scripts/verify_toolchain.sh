#!/usr/bin/env bash
# Verifies that the checked-in Rust toolchain is consistent across local and CI entrypoints.
set -u

cd "$(dirname "$0")/.."
fail=0

toolchain="$(grep -m1 '^channel = ' rust-toolchain.toml | sed 's/.*"\(.*\)".*/\1/')"
cargo_version="$(grep -m1 '^rust-version = ' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"

if [ -z "${toolchain}" ]; then
  echo "ERROR: rust-toolchain.toml does not declare a channel" >&2
  fail=1
fi

if [ "${cargo_version}" != "${toolchain}" ]; then
  echo "ERROR: Cargo.toml rust-version '${cargo_version}' does not match rust-toolchain.toml channel '${toolchain}'" >&2
  fail=1
fi

for workflow in .github/workflows/ci.yml .github/workflows/publish.yml; do
  if ! grep -qF "toolchain: ${toolchain}" "${workflow}"; then
    echo "ERROR: ${workflow} does not select Rust ${toolchain}" >&2
    fail=1
  fi
done

if ! grep -qF 'components: clippy, rustfmt' .github/workflows/ci.yml; then
  echo "ERROR: CI does not install the clippy and rustfmt components" >&2
  fail=1
fi

if ! grep -qF 'components: clippy, rustfmt' .github/workflows/publish.yml; then
  echo "ERROR: publish does not install the clippy and rustfmt components" >&2
  fail=1
fi

if ! grep -qF "Rust ${toolchain}" CONTRIBUTING.md; then
  echo "ERROR: CONTRIBUTING.md does not document Rust ${toolchain}" >&2
  fail=1
fi

if ! grep -qF "Rust ${toolchain}" UPSTREAM_MAPPING.md; then
  echo "ERROR: UPSTREAM_MAPPING.md does not document Rust ${toolchain}" >&2
  fail=1
fi

if [ "${fail}" -ne 0 ]; then
  exit 1
fi

echo "OK: Rust ${toolchain} is consistent across the manifest, toolchain file, workflows, contributor docs, and upstream mapping"
