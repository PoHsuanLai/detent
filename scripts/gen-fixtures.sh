#!/usr/bin/env bash
# Regenerates fixtures/schemas from the real programs' own `--write-schema`, into a scratch
# directory first. Builds each program in its sibling checkout (../sill, ../palmrest, ../quire).
#
#   sill        sill --write-schema <dir>
#   palmrest    palmrest --write-schema <dir>
#   quire       ds-settings' `write_schema` example (quire has no app binary)
#   inferd      ../porter/dist/inferd.settings.toml (written by hand, no --write-schema yet)
#   docket      ../docket/dist/settings/docket.settings.toml
#   almanac     ../almanac/dist/settings/almanac.settings.toml
#   cua         ../cua/dist/settings/cua.settings.toml
#
# Usage: scripts/gen-fixtures.sh
set -euo pipefail
cd "$(dirname "$0")/.."

scratch="$(mktemp -d "${TMPDIR:-/tmp}/detent-fixtures.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT

# A private home: the programs read no real config while writing a schema.
private() { env -i PATH="$PATH" HOME="$scratch/home" XDG_CONFIG_HOME="$scratch/config" XDG_DATA_HOME="$scratch/data" "$@"; }

cargo build --quiet --manifest-path ../sill/Cargo.toml -p sill
cargo build --quiet --manifest-path ../palmrest/Cargo.toml -p palmrest
cargo build --quiet --manifest-path ../quire/Cargo.toml -p ds-settings --example write_schema

private "$(cargo metadata --format-version 1 --no-deps --manifest-path ../sill/Cargo.toml | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/debug/sill" --write-schema "$scratch/out"
private "$(cargo metadata --format-version 1 --no-deps --manifest-path ../palmrest/Cargo.toml | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/debug/palmrest" --write-schema "$scratch/out"
private "$(cargo metadata --format-version 1 --no-deps --manifest-path ../quire/Cargo.toml | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/debug/examples/write_schema" --write-schema "$scratch/out"

mkdir -p fixtures/schemas
cp "$scratch"/out/*.settings.toml fixtures/schemas/
cp ../porter/dist/inferd.settings.toml fixtures/schemas/
cp ../docket/dist/settings/docket.settings.toml ../almanac/dist/settings/almanac.settings.toml \
  ../cua/dist/settings/cua.settings.toml fixtures/schemas/
ls -l fixtures/schemas
