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
| docket (`../docket`, path deps; a private repo, which detent may name) | `docket-client` with its `dbus` feature (`IntentProvider`, `serve_on`), `docket-core` (manifest and call types), `docket-dbus` (the bus connection type); `prov` and `porter-core` from `../porter`, the label and name types those signatures use. Used by `detent-intents` only. Its `scripts/check-intents.sh` is run on this repo |
| sill, palmrest | nothing at build time. Their `--write-schema` output is the test fixture (`scripts/gen-fixtures.sh`); so is quire's own schema, from `ds-settings`' `write_schema` example |

## 1. Crate map and layers

| # | Crate | What it is | May depend on |
| --- | --- | --- | --- |
| 0 | `detent-testkit` | Dev only: `Scratch` directories under the target directory, the committed schema fixtures | std only |
| 1 | `detent-model` | Pure: `discover` (stale schemas skipped with a `Warning`), `Catalogue` (page then section grouping, fixed page order), `Control` per `KeyKind` (a two-word enum is a switch only when it is on and off), `Range`, `search`, `visible_pages`/`landing` | `ds-settings` (no features) |
| 2 | `detent-files` | Effects on the settings files: `load` (lenient, unknown keys reported, unrepresentable values kept as written), `save` (one key set in place, atomic temp and rename) | `detent-model`, `ds-settings` (no features) |
| 3 | `detent-intents` | What an agent or a command line may do in Settings: the manifest `org.quire.Settings` (read from `dist/intents/`), `SettingsProvider` (the `IntentProvider1` for `settings.page.open`, `settings.key.read`, `settings.key.set`), the undo journal, `serve` and `serve_session`. A `Window` trait is how it tells the window; no UI crate | `detent-model`, `detent-files`, docket and porter (above), `ds-settings` (no features) |
| 3 | `detent-ui` | The window: `App`, `Env`, sidebar, pages, sections, one control component per `Control` | `detent-model`, `detent-files`, `ds`, `ds-settings` (dioxus), `ds-blitz`; dev: `ds-harness`, `ds-lint` |
| 4 | `detent` | The binary: flags, the environment read once, the intents service on the process-wide runtime, `ds_blitz::launch`; `WindowLink` turns the provider's `Window` calls into `detent-ui` notices | `detent-ui`, `detent-intents`, `detent-model`, `detent-files`, `ds-blitz`, `ds-settings`; clap, tokio |

`scripts/check-boundary.sh` keeps this table exact (`EDGES`) and checks that `detent-model` and
`detent-files` reach none of dioxus, tokio, blitz-dom, ds, ds-blitz, ds-harness, ds-lint. zbus is not
in that list: `ds-settings` builds it (FINDINGS "ds-settings drags zbus into a pure crate").

## 2. One home

| Concept | Home |
| --- | --- |
| Which schemas exist, which are stale | `detent-model::discover` |
| Pages, sections, order; Basic sections, and the page's one Advanced group | `detent-model::Catalogue` |
| Which control draws a key, which values it admits; whether two words are a switch | `detent-model::Control`, `TogglePair::of` |
| What a search finds, which pages stay in the sidebar | `detent-model::search`, `visible_pages`, `landing` |
| Reading and writing a settings file | `detent-files::{load, save}` |
| Whether an agent may set a key | the schema's `agent = "settable"` mark, read by `detent-model::discover` into `KeyEntry::agent` (a key without it is hands-off) |
| A key's value as text, and text as a value | `detent-model::{Control::from_text, show_value}` |
| What the desktop's intents may do here | `dist/intents/org.quire.Settings.toml` (the one declaration) and `detent-intents::SettingsProvider` |
| What the window is told while it runs | `detent-ui::{Notice, inbox}` (a page to open, files to read again) |
| Where the window opens, where files live | `detent-ui::Env` (a context; the binary fills it, a test fills it with scratch paths) |
| The layout stylesheet | `detent-ui/src/style.css` (tokens only, linted) |

## 3. Recipes

**A key kind gets a component.** Add the `Control` variant in `detent-model/src/control.rs` (the
match on `KeyKind` is total, so the compiler lists what is missing), add `controls/<name>.rs` in
`detent-ui` with one quire component, and one arm in `controls/mod.rs`.

**A page appears.** Nothing to do: a program ships a schema naming the page, `Catalogue` groups it.
A new fixed page is a variant of `ds_settings::schema::Page` (quire first), then a row in
`detent-model/src/page.rs` and an icon in `detent-ui/src/icons.rs`.

**An action is added.** Declare it in `dist/intents/org.quire.Settings.toml` (every field written,
a `params` list even when empty), add its arm in `detent-intents/src/ask.rs` and in
`SettingsProvider::perform`, and a test on the private bus (`detent-intents/tests`). Run
`../docket/scripts/check-intents.sh`; the installer (`scripts/install.sh`) already ships the file.

## 4. Tests

- `detent-model`, `detent-files`: table tests and integration tests over `fixtures/schemas`, real
  schemas from `sill --write-schema`, `palmrest --write-schema` and quire's `write_schema` example.
  Regenerate with `scripts/gen-fixtures.sh`.
- `detent-ui/tests/support`: `Sandbox` seeds the fixtures into a scratch `$XDG_DATA_HOME`, points
  the app at a scratch `$XDG_CONFIG_HOME`, fixes the desktop preferences, and builds a
  `ds_harness::Harness`. `behaviour.rs` drives it; `coherence.rs` lints the CSS and every page's
  markup; `snapshots.rs` writes PNGs to `$DETENT_SHOTS`.
- `Sandbox` runs the harness on its virtual clock (`Clock::Virtual`): a save is judged by what the
  test did, not by how loaded the host is. `tests/notices.rs` drives the inbox; `tests/viewer_page.rs`
  installs a scratch `anyview` schema for the Viewer page.
- `detent-intents/tests`: the provider served on a private `dbus-daemon` (`PrivateBus`), called from
  a connection that owns `org.quire.Intents1` (a provider answers nobody else), the way intentd calls
  an app. `detent/tests/install.rs` runs `scripts/install.sh` into a scratch `DESTDIR`.
- No test reads the real config, the session bus or the portal.

## 5. Repo rules

- Gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, `cargo test --workspace`, `./scripts/check-boundary.sh`, `cargo deny check licenses`, and `./scripts/check-intents.sh` (docket's
  conformance check: a `.desktop` file needs a valid manifest; it builds `docket-eval` from `../docket`).
  Check every exit code.
- No `unsafe` anywhere (`unsafe_code = "deny"`).
- No widget is drawn locally. A kind quire has no component for is shown read-only with an "Edit in
  file" note and listed in FINDINGS.
- detent never removes anything: a save sets only the key the person changed, and every other key,
  known or not, keeps its text (only the owning program may drop what it does not read). A value a
  key's control cannot hold is shown read-only with "Edit in file". A schema whose `file` is not `<app>/<name>.toml` is stale and skipped.

## 6. App id and porter callers

detent's app id is `org.quire.Settings` (porter refuses a name that is not reverse-DNS), and
`dist/org.quire.Settings.desktop` is named after it, with `StartupWMClass` the same, so the window's
class matches its launcher entry. Two caller rows let it reach the porter daemons' settings modules;
they are samples to merge into the installed porter config, not installed files:

- `dist/porter/inferd.callers.toml`: `settings = ["org.quire.Settings"]` under `[callers]` in
  `$XDG_CONFIG_HOME/quire/inferd.toml`.
- `dist/porter/callers.toml`: the `role = "settings"` row of accountd's `/etc/porter/callers.toml`.
