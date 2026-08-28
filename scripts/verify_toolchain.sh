#!/usr/bin/env bash
# Verifies that the checked-in Rust toolchain is consistent across local and CI entrypoints.
set -u

if [ -n "${RELEASE_ROOT:-}" ]; then
  cd "${RELEASE_ROOT}"
else
  cd "$(dirname "$0")/.."
fi
fail=0
toolchain_action_sha="6c977a6ca4077a0ceb28ffbe03f59d46e9ac8772"
checkout_sha="11bd71901bbe5b1630ceea73d27597364c9af683"
setup_go_sha="d35c59abb061a4a6fb18e82ac0862c26744d6ab5"
upstream_commit="fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1"

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

publish_checkout_count="$(grep -cF 'uses: actions/checkout@' .github/workflows/publish.yml)"
publish_pinned_checkout_count="$(grep -cF "uses: actions/checkout@${checkout_sha}" .github/workflows/publish.yml)"
publish_credential_count="$(grep -cF 'persist-credentials: false' .github/workflows/publish.yml)"
publish_setup_go_count="$(grep -cF 'uses: actions/setup-go@' .github/workflows/publish.yml)"
publish_pinned_setup_go_count="$(grep -cF "uses: actions/setup-go@${setup_go_sha}" .github/workflows/publish.yml)"
sibling_count="$(grep -cF 'repository: coderbants/' .github/workflows/publish.yml)"
sibling_ref_count="$(grep -Ec '^[[:space:]]+ref: [0-9a-f]{40}$' .github/workflows/publish.yml)"
upstream_checkout_count="$(grep -cF "git checkout --quiet ${upstream_commit}" .github/workflows/publish.yml)"
registry_token_count="$(grep -cF '          CARGO_REGISTRY_TOKEN:' .github/workflows/publish.yml)"
github_token_count="$(grep -cF '          GH_TOKEN:' .github/workflows/publish.yml)"

if [ "${publish_checkout_count}" -eq 0 ] || [ "${publish_checkout_count}" -ne "${publish_pinned_checkout_count}" ] || [ "${publish_checkout_count}" -ne "${publish_credential_count}" ]; then
  echo "ERROR: every publish checkout must use the approved immutable pin without persisted credentials" >&2
  fail=1
fi

if [ "${publish_setup_go_count}" -eq 0 ] || [ "${publish_setup_go_count}" -ne "${publish_pinned_setup_go_count}" ]; then
  echo "ERROR: every publish setup-go action must use the approved immutable pin" >&2
  fail=1
fi

if [ "${sibling_count}" -eq 0 ] || [ "${sibling_count}" -ne "${sibling_ref_count}" ] || grep -qF 'ref: dev' .github/workflows/publish.yml; then
  echo "ERROR: every publish sibling checkout must use an immutable commit ref" >&2
  fail=1
fi

if [ "${upstream_checkout_count}" -ne 1 ]; then
  echo "ERROR: publish must execute the verified upstream commit ${upstream_commit}" >&2
  fail=1
fi

if [ "${registry_token_count}" -ne 1 ] || [ "${github_token_count}" -ne 2 ]; then
  echo "ERROR: release credentials must remain scoped to their individual publication steps" >&2
  fail=1
fi

if grep -qF 'workflow_dispatch:' .github/workflows/publish.yml; then
  echo "ERROR: publish must not be manually dispatchable outside a release tag" >&2
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
