#!/usr/bin/env bash
# Crate boundaries, mechanically enforced.
#
# `cargo tree -i <dep>` exits 101 when the dependency is absent, which is precisely the state
# we want. Checking the exit status would therefore fail whenever the boundary holds, so we
# check for OUTPUT instead: any line naming the dependency is a leak.
set -uo pipefail
cd "$(dirname "$0")/.."

# ARCHITECTURE.md section 1 is the crate map; this file is its mechanical form, in two parts.
#
# RULES: what a crate reaches through ANY path (`cargo tree -i`, transitive).
# detent-model and detent-files are pure: no UI, no renderer, no runtime. They name ds-settings
# for its schema types and its lenient-read types, and ds-settings builds zbus for its portal
# watch, so zbus cannot be in the forbidden list until ds-settings splits its schema module out
# (FINDINGS.md "ds-settings drags zbus into a pure crate"). detent-testkit is std only.
# detent-intents answers the desktop's intents with no window: it reaches no UI crate.
PURE="dioxus dioxus-core tokio blitz-dom ds ds-blitz ds-harness ds-lint"
RULES=(
  "detent-model: $PURE"
  "detent-files: $PURE"
  "detent-testkit: $PURE zbus ds-settings detent-model detent-files detent-ui detent"
  "detent-ui: detent detent-intents"
  "detent-intents: dioxus dioxus-core blitz-dom ds ds-blitz ds-harness ds-lint detent-ui detent"
)
fail=0

for rule in "${RULES[@]}"; do
  crate="${rule%%:*}"
  read -r -a forbidden <<<"${rule#*:}"
  # A crate that cargo cannot find would make every check below pass vacuously.
  if ! cargo tree -p "$crate" --depth 0 >/dev/null 2>&1; then
    echo "ERROR: cargo tree cannot resolve $crate; the boundary was not checked"
    fail=1
    continue
  fi
  leaked=0
  for dep in "${forbidden[@]}"; do
    if cargo tree -p "$crate" -i "$dep" -e normal,build 2>/dev/null | grep -q .; then
      echo "LEAK: $crate depends on $dep"
      cargo tree -p "$crate" -i "$dep" -e normal,build 2>/dev/null | head -20
      leaked=1
      fail=1
    fi
  done
  if [ "$leaked" -eq 0 ]; then
    echo "boundary holds: $crate reaches none of ${forbidden[*]}"
  fi
done

# The allowed edges between our own crates and quire (ARCHITECTURE.md section 1): each crate's
# DIRECT normal and build dependencies that are path dependencies, and nothing else. A
# dependency on a crate not listed is a leak; so is one the crate no longer has, so the table
# stays exact. Dev dependencies are outside it (a test may reach further than the crate does).
EDGES=(
  "detent-testkit:"
  "detent-model: ds-settings porter-core porter-infer"
  "detent-files: detent-model ds-settings"
  "detent-intents: detent-files detent-model docket-client docket-core docket-dbus ds-settings porter-core prov"
  "detent-ui: detent-files detent-model ds ds-blitz ds-settings porter-dbus porter-infer"
  "detent: detent-files detent-intents detent-model detent-ui ds-blitz ds-settings"
)
for edge in "${EDGES[@]}"; do
  crate="${edge%%:*}"
  read -r -a allowed <<<"${edge#*:}"
  found=$(cargo tree -p "$crate" --depth 1 -e normal,build --prefix none --all-features 2>/dev/null \
    | grep '(/' | awk '{print $1}' | grep -vx "$crate" | sort -u | tr '\n' ' ')
  want=$(printf '%s\n' "${allowed[@]}" | grep . | sort -u | tr '\n' ' ')
  if [ "$found" != "$want" ]; then
    echo "EDGE: $crate depends on [${found% }], the table allows [${want% }]"
    fail=1
  else
    echo "edges hold: $crate depends on [${found% }]"
  fi
done

# Every workspace member has a row above, so a new crate cannot slip in unchecked.
for member in $(cargo metadata --no-deps --format-version 1 2>/dev/null | grep -o '"name":"detent[a-z-]*"' | cut -d'"' -f4 | sort -u); do
  printf '%s\n' "${EDGES[@]}" | grep -q "^$member:" || { echo "ERROR: $member has no row in EDGES"; fail=1; }
done

exit "$fail"
