# Architecture

detent is the desktop's Settings app: one window on Blitz, drawn only with quire's components.
Programs ship a schema of their settings (quire design/22 section 9); detent renders pages from
every schema it finds and reads and writes the files the schemas name. It never talks to a
program: each program's own directory watch applies what detent writes. CONVENTIONS.md holds the
rules every repo shares; this file is detent's map and rules, and wins where they disagree.

## Sibling repositories

| Repo | What detent takes from it |
| --- | --- |
| quire (`../quire`, path deps) | `ds` (components, `Ds`, `ds::prelude`), `ds-settings` (schema types, `discover`, `Loaded`, `use_environment`), `ds-blitz` (`launch`), `ds-lint` and `ds-harness` (dev only) |
| sill, palmrest | nothing at build time. Their `--write-schema` output is the test fixture (`scripts/gen-fixtures.sh`) |

## 1. Crate map and layers

| # | Crate | What it is | May depend on |
| --- | --- | --- | --- |
| 0 | `detent-testkit` | Dev only: `Scratch` directories under the target directory, the committed schema fixtures | std only |
| 1 | `detent-model` | Pure: `discover` (stale schemas skipped with a `Warning`), `Catalogue` (page then section grouping, fixed page order), `Control` per `KeyKind`, `Range`, `search`, `visible_pages`/`landing` | `ds-settings` (no features) |
| 2 | `detent-files` | Effects on the settings files: `load` (lenient, unknown keys reported), `save` (atomic temp and rename) | `detent-model`, `ds-settings` (no features) |
| 3 | `detent-ui` | The window: `App`, `Env`, sidebar, pages, sections, one control component per `Control` | `detent-model`, `detent-files`, `ds`, `ds-settings` (dioxus), `ds-blitz`; dev: `ds-harness`, `ds-lint` |
| 4 | `detent` | The binary: flags, the environment read once, `ds_blitz::launch` | `detent-ui`, `detent-model`, `detent-files`, `ds-blitz`, `ds-settings`; clap |

`scripts/check-boundary.sh` keeps this table exact (`EDGES`) and checks that `detent-model` and
`detent-files` reach none of dioxus, tokio, blitz-dom, ds, ds-blitz, ds-harness, ds-lint. zbus is not
in that list: `ds-settings` builds it (FINDINGS "ds-settings drags zbus into a pure crate").

## 2. One home

| Concept | Home |
| --- | --- |
| Which schemas exist, which are stale | `detent-model::discover` |
| Pages, sections, order, Basic against Advanced | `detent-model::Catalogue` |
| Which control draws a key, which values it admits | `detent-model::Control` |
| What a search finds, which pages stay in the sidebar | `detent-model::search`, `visible_pages`, `landing` |
| Reading and writing a settings file | `detent-files::{load, save}` |
| Where the window opens, where files live | `detent-ui::Env` (a context; the binary fills it, a test fills it with scratch paths) |
| The layout stylesheet | `detent-ui/src/style.css` (tokens only, linted) |

## 3. Recipes

**A key kind gets a component.** Add the `Control` variant in `detent-model/src/control.rs` (the
match on `KeyKind` is total, so the compiler lists what is missing), add `controls/<name>.rs` in
`detent-ui` with one quire component, and one arm in `controls/mod.rs`.

**A page appears.** Nothing to do: a program ships a schema naming the page, `Catalogue` groups it.
A new fixed page is a variant of `ds_settings::schema::Page` (quire first), then a row in
`detent-model/src/page.rs` and an icon in `detent-ui/src/icons.rs`.

## 4. Tests

- `detent-model`, `detent-files`: table tests and integration tests over `fixtures/schemas`, real
  schemas from `sill --write-schema` and quire's derive. Regenerate with `scripts/gen-fixtures.sh`.
- `detent-ui/tests/support`: `Sandbox` seeds the fixtures into a scratch `$XDG_DATA_HOME`, points
  the app at a scratch `$XDG_CONFIG_HOME`, fixes the desktop preferences, and builds a
  `ds_harness::Harness`. `behaviour.rs` drives it; `coherence.rs` lints the CSS and every page's
  markup; `snapshots.rs` writes PNGs to `$DETENT_SHOTS`.
- No test reads the real config, the session bus or the portal.

## 5. Repo rules

- Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, `cargo test --workspace`, `./scripts/check-boundary.sh`, `cargo deny check licenses`.
  Check every exit code.
- No `unsafe` anywhere (`unsafe_code = "deny"`).
- No widget is drawn locally. A kind quire has no component for is shown read-only with an "Edit in
  file" note and listed in FINDINGS.
- Nothing is written that no schema describes: unknown keys are reported on load and dropped by the
  next save; a schema whose `file` is not `<app>/<name>.toml` is stale and skipped.
