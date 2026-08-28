#!/usr/bin/env bash
# Exercises the release-policy guards with both valid and invalid inputs.
set -u

cd "$(dirname "$0")/.."
fail=0

if ! ./scripts/verify_toolchain.sh >/dev/null; then
  echo "ERROR: toolchain/release structure verification failed" >&2
  fail=1
fi

if ! ./scripts/verify_upstream_version.sh v2.0.8 >/dev/null; then
  echo "ERROR: valid upstream release tag was rejected" >&2
  fail=1
fi

if UPSTREAM_VERSION_SOURCE=rust-toolchain.toml ./scripts/verify_upstream_version.sh v2.0.8 >/dev/null 2>&1; then
  echo "ERROR: missing upstream header was accepted" >&2
  fail=1
else
  echo "OK: missing upstream header is rejected"
fi

fixture_root="$(mktemp -d)"
trap 'rm -rf "${fixture_root}"' EXIT
mkdir -p "${fixture_root}/.github/workflows" "${fixture_root}/scripts"
cp Cargo.toml rust-toolchain.toml CONTRIBUTING.md UPSTREAM_MAPPING.md "${fixture_root}/"
cp .github/workflows/ci.yml .github/workflows/publish.yml "${fixture_root}/.github/workflows/"
cp scripts/verify_toolchain.sh "${fixture_root}/scripts/"

perl -0pi -e 's/actions\/checkout\@11bd71901bbe5b1630ceea73d27597364c9af683/actions\/checkout\@v4/' "${fixture_root}/.github/workflows/publish.yml"
if RELEASE_ROOT="${fixture_root}" "${fixture_root}/scripts/verify_toolchain.sh" >/dev/null 2>&1; then
  echo "ERROR: mutable checkout action pin was accepted" >&2
  fail=1
else
  echo "OK: mutable checkout action pin is rejected"
fi

cp .github/workflows/publish.yml "${fixture_root}/.github/workflows/publish.yml"
perl -0pi -e 's/ref: [0-9a-f]{40}/ref: dev/' "${fixture_root}/.github/workflows/publish.yml"
if RELEASE_ROOT="${fixture_root}" "${fixture_root}/scripts/verify_toolchain.sh" >/dev/null 2>&1; then
  echo "ERROR: mutable sibling ref was accepted" >&2
  fail=1
else
  echo "OK: mutable sibling ref is rejected"
fi

cp .github/workflows/publish.yml "${fixture_root}/.github/workflows/publish.yml"
perl -0pi -e 's/persist-credentials: false/persist-credentials: true/' "${fixture_root}/.github/workflows/publish.yml"
if RELEASE_ROOT="${fixture_root}" "${fixture_root}/scripts/verify_toolchain.sh" >/dev/null 2>&1; then
  echo "ERROR: persisted checkout credentials were accepted" >&2
  fail=1
else
  echo "OK: persisted checkout credentials are rejected"
fi

cp .github/workflows/publish.yml "${fixture_root}/.github/workflows/publish.yml"
perl -0pi -e 's/fc707bb7ea0161405bb6c653ec93f6a9c6a72fe1/v2.0.8/' "${fixture_root}/.github/workflows/publish.yml"
if RELEASE_ROOT="${fixture_root}" "${fixture_root}/scripts/verify_toolchain.sh" >/dev/null 2>&1; then
  echo "ERROR: mutable upstream ref was accepted" >&2
  fail=1
else
  echo "OK: mutable upstream ref is rejected"
fi

cp .github/workflows/publish.yml "${fixture_root}/.github/workflows/publish.yml"
perl -0pi -e 's/          CARGO_REGISTRY_TOKEN:/      CARGO_REGISTRY_TOKEN:/' "${fixture_root}/.github/workflows/publish.yml"
if RELEASE_ROOT="${fixture_root}" "${fixture_root}/scripts/verify_toolchain.sh" >/dev/null 2>&1; then
  echo "ERROR: job-scoped registry credentials were accepted" >&2
  fail=1
else
  echo "OK: job-scoped registry credentials are rejected"
fi

if [ "${fail}" -ne 0 ]; then
  exit 1
fi

echo "OK: release guards reject unsafe inputs and accept the valid release path"
