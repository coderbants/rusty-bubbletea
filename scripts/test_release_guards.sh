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

if [ "${fail}" -ne 0 ]; then
  exit 1
fi

echo "OK: release guards reject unsafe inputs and accept the valid release path"
