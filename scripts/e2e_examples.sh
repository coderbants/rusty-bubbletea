#!/usr/bin/env bash
# Dual-build example E2E sweep.
#
# Every example spec in scripts/e2e_specs/<name>.json is run against BOTH
# the upstream Go build and the Rust port through the identical scripted
# input sequence. The test PASSES only when:
#   1. the Go build satisfies the spec's `expect` fragments (the example
#      actually performs its documented functionality), and
#   2. the Rust build's per-phase screens and exit are 1:1 identical to Go.
#
# Usage: scripts/e2e_examples.sh [example_name ...]   (no args: all specs)

set -u
cd "$(dirname "$0")/.."
export TERM=xterm-256color
export LANG=C

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

GOBIN="$TMP/go-bin"
mkdir -p "$GOBIN" "$TMP/out"

# Map Rust example names to upstream Go dirs (dashes vs underscores etc.).
GO_NAME() { echo "$1" | tr '_' '-'; }
RS_NAME() { echo "$1" | tr '-' '_'; }

SPECS="$@"
if [ -z "$SPECS" ]; then
  SPECS=$(ls scripts/e2e_specs/*.json 2>/dev/null | sed 's/.*\///; s/\.json$//')
fi

cargo build --examples 2>&1 | grep -E "^error" | head -5

pass=0; fail=0; skipped=0

for name in $SPECS; do
  [ -z "$name" ] && continue
  spec="scripts/e2e_specs/$name.json"
  godir=$(GO_NAME "$name")
  rsbin="examples/$(RS_NAME "$name")"
  if [ ! -f "$spec" ] || [ ! -f "$rsbin.rs" ] || [ ! -d "upstream-go/examples/$godir" ]; then
    echo "SKIP  $name (missing source)"
    skipped=$((skipped+1))
    continue
  fi
  if ! (cd upstream-go/examples && go build -o "$GOBIN/$name" "./$godir" 2>/dev/null); then
    echo "SKIP  $name (go build failed)"
    skipped=$((skipped+1))
    continue
  fi

  # The Go examples resolve fixtures (e.g. the pager's artichoke.md) relative
  # to their source directory, so run each from there.
  timeout 40 python3 scripts/e2e.py --cmd bash --args=-c --args="cd upstream-go/examples/$godir && exec \"$GOBIN/$name\"" --spec "$spec" --out "$TMP/out/go.$name.json" 2>/dev/null
  timeout 40 python3 scripts/e2e.py --cmd "target/debug/$rsbin" --spec "$spec" --out "$TMP/out/rs.$name.json" 2>/dev/null

  python3 - "$spec" "$TMP/out/go.$name.json" "$TMP/out/rs.$name.json" << 'PYEOF'
import json, sys
spec = json.load(open(sys.argv[1]))
go = json.load(open(sys.argv[2]))
rs = json.load(open(sys.argv[3]))

fails = []
# 1. Functionality: the Go build must satisfy every expect fragment.
for i, phase in enumerate(spec.get("phases", [])):
    scr = go["screens"][i] if i < len(go["screens"]) else {"cells": [], "rows": 0, "cols": 0}
    text = ""
    cells = {tuple(c[:2]): c[2][0] for c in scr.get("cells", [])}
    text = "\n".join("".join(cells.get((x, y), " ") for x in range(scr["cols"])).rstrip()
                     for y in range(scr["rows"]))
    for frag in phase.get("expect", []):
        if frag not in text:
            fails.append(f"phase {i}: GO missing expected text {frag!r}")
    for frag in phase.get("expect_not", []):
        if frag in text:
            fails.append(f"phase {i}: GO unexpectedly contains {frag!r}")
if spec.get("expect_exit", True) and not go["exited"]:
    fails.append("GO did not exit")

# 2. 1:1 parity: Rust screens + exit must equal Go (animation cells are
# already zeroed by the driver per the spec's ignore_cells).
if go["screens"] != rs["screens"]:
    fails.append("Rust screens differ from Go (1:1 parity failed)")
if go["exited"] != rs["exited"]:
    fails.append("Rust exit behavior differs from Go")

if fails:
    print("FAIL  %s" % "|".join(fails[:3]))
    sys.exit(1)
print("PASS")
PYEOF
  rc=$?
  if [ $rc -eq 0 ]; then
    echo "PASS  $name"
    pass=$((pass+1))
  else
    echo "FAIL  $name"
    fail=$((fail+1))
  fi
done

echo
echo "pass=$pass fail=$fail skipped=$skipped"
[ "$fail" -eq 0 ] || exit 1
