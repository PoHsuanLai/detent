#!/usr/bin/env bash
# Regenerates fixtures/schemas from the real programs, into a scratch directory first.
#
#   sill                           sill --write-schema           (the built sill binary)
#   quire                          examples/write_quire_schema.rs (quire ships no command)
#   palmrest                       has no schema yet, so there is no fixture (FINDINGS.md)
#
# Usage: scripts/gen-fixtures.sh [path/to/sill]   (default: ../sill/target/debug/sill)
set -euo pipefail
cd "$(dirname "$0")/.."

sill="${1:-../sill/target/debug/sill}"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/detent-fixtures.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT

# A private home: the programs read no real config while writing a schema.
env -i HOME="$scratch/home" XDG_CONFIG_HOME="$scratch/config" XDG_DATA_HOME="$scratch/data" \
  "$sill" --write-schema "$scratch/out"
cargo run --quiet -p detent-model --example write_quire_schema -- "$scratch/out"

mkdir -p fixtures/schemas
cp "$scratch"/out/*.settings.toml fixtures/schemas/
ls -l fixtures/schemas
