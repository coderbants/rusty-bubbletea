#!/usr/bin/env bash
# Verifies that the checked-in Rust toolchain is consistent across local and CI entrypoints.
set -u

cd "$(dirname "$0")/.."
fail=0
toolchain_action_sha="6c977a6ca4077a0ceb28ffbe03f59d46e9ac8772"

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
  action_count="$(grep -cF 'uses: dtolnay/rust-toolchain@' "${workflow}")"
  pinned_action_count="$(grep -cF "uses: dtolnay/rust-toolchain@${toolchain_action_sha}" "${workflow}")"
  selector_count="$(grep -cF 'toolchain:' "${workflow}")"
  matching_count="$(grep -cF "toolchain: ${toolchain}" "${workflow}")"
  if [ "${action_count}" -eq 0 ]; then
    echo "ERROR: ${workflow} does not install a Rust toolchain" >&2
    fail=1
  elif [ "${action_count}" -ne "${pinned_action_count}" ] || [ "${action_count}" -ne "${selector_count}" ] || [ "${action_count}" -ne "${matching_count}" ]; then
    echo "ERROR: every Rust toolchain action in ${workflow} must use the approved immutable pin and select Rust ${toolchain}" >&2
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
