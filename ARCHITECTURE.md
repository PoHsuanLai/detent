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
| docket (`../docket`, path deps; a private repo, which detent may name) | `docket-client` with its `dbus` feature (`IntentProvider`, `serve_on`), `docket-core` (manifest and call types), `docket-dbus` (the bus connection type), `docket-settings` (the assistant's places: `Places`, `read_places`, `write_places`; the outside agents: `read_agents`, `write_agent_choice`), `docket-agents` (`AgentsDir`, named by `detent-files` only); `prov` and `porter-core` from `../porter`, the label and name types those signatures use. Used by `detent-intents`, and `docket-settings` by the model, the files and the window. Its `scripts/check-intents.sh` is run on this repo |
| almanac (`../almanac`, path dep) | `almanac-core` (`Removal`, `MemoryFate`, `HistoryFate`: the Space-removal choice and the JSON memory reads) |
| porter (`../porter`, path deps) | `porter-core` (`Tier`), `porter-infer` (`tier_label`, `Why`, `Declined`, `InferEvent`: the picker's vocabulary and the reason footer's facts), `porter-dbus` (`INFERENCE_BUS`, `INFERENCE_SETTINGS_PATH`). `detent-intents` also takes `prov` from here |
| sill, palmrest | nothing at build time. Their `--write-schema` output is the test fixture (`scripts/gen-fixtures.sh`); so is quire's own schema, from `ds-settings`' `write_schema` example |

## 1. Crate map and layers

| # | Crate | What it is | May depend on |
| --- | --- | --- | --- |
| 0 | `detent-testkit` | Dev only: `Scratch` directories under the target directory, the committed schema fixtures | std only |
| 1 | `detent-model` | Pure: `discover` (stale schemas skipped with a `Warning`), `Catalogue` (page then section grouping, fixed page order), `blocks` (what a page draws in order: the Intelligence sections in design/22's order, the local runtimes and the model picker after Models, the Accounts pane before the Accounts page's file sections), `PickerState` and `step` (inferd's live model picker as a pure machine), `AccountsState` and `accounts_step` (accountd's Accounts pane: the live rows read as accounts, one screen at a time, a removal asked about before it is run; `AccountsPane::local_runtimes`: the accounts whose `place` row is this computer), `FirstRun` and `first_run_step` (the welcome window's choices; `runtime_choice` names the picker row and word a runtime is written to), `Reason` (the footer's sentences over `Why` and `Declined`), `Control` per `KeyKind` (a two-word enum is a switch only when it is on and off; a `Rows` key is `Columns`, whose cells are checked against each column), `Range`, `search`, `visible_pages`/`landing` | `ds-settings` (no features), `porter-core`, `porter-infer`, `docket-settings` (the assistant's places), `almanac-core` (the Space-removal choice and its wire form); `regex` (a rows column's pattern), `serde_json` (dev) |
| 2 | `detent-files` | Effects on the settings files: `load` (lenient, unknown keys reported, unrepresentable values kept as written), `save` (one key set in place, atomic temp and rename), `FirstRunMark` (the one-line state file that says the first run happened) | `detent-model`, `docket-settings`, `ds-settings` (no features) |
| 3 | `detent-intents` | What an agent or a command line may do in Settings: the manifest `org.quire.Settings` (read from `dist/intents/`), `SettingsProvider` (the `IntentProvider1` for `settings.page.show`, `settings.key.read`, `settings.key.set`), the undo journal, `serve` and `serve_session`. A `Window` trait is how it tells the window; no UI crate | `detent-model`, `detent-files`, docket and porter (above), `ds-settings` (no features) |
| 3 | `detent-ui` | The window: `App`, `Env`, sidebar, pages, sections, one control component per `Control`, the model picker, the Accounts pane, `inference/runtimes_view.rs` ("Models on this computer"), `first_run/` (the welcome window), `live/` (the look-for-a-daemon loop they share) | `detent-model`, `detent-files`, `almanac-core`, `docket-settings`, `ds`, `ds-settings` (dioxus, live, tokio), `ds-blitz`, `porter-client` (dbus), `porter-core`, `porter-dbus`, `porter-infer`, `porter-provider`; tokio, zbus; dev: `ds-harness`, `ds-lint` |
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
| What the window is told while it runs | `detent-ui::{Notice, inbox}` (a page to open, files to read again, an account to show, a raise) |
| The D-Bus surface of `org.quire.Settings` | `org.quire.IntentProvider1` at `/org/quire/IntentProvider1` (`detent-intents/src/serve.rs`) and `org.freedesktop.Application` at `/org/quire/Settings` (`detent-intents/src/application.rs`: `Activate`, `Open` not supported, `ActivateAction("open-account", [path])`); the second launch's hand-over is `hand_over`, called by `detent/src/main.rs` |
| Which account an object path names | `detent-model::{AccountTarget, AccountsPane::key_of}` (the path prefix is written there once; an unknown segment shows the list) |
| Where the window opens, where files live | `detent-ui::Env` (a context; the binary fills it, a test fills it with scratch paths) |
| What an Intelligence page draws and in what order | `detent-model::blocks` |
| The model picker's state and transitions | `detent-model::{PickerState, step}`; the bus calls in `detent-ui/src/inference/io.rs` and the watch for inferd's name in `follow.rs`, the card in `inference/picker_view.rs` |
| The words of the reason footer | `detent-model::Reason` |
| The Accounts pane's state and transitions; which row is an account's service, permission or action | `detent-model::{AccountsState, accounts_step, AccountsPane::of}`; the bus calls in `detent-ui/src/accounts/drive.rs` (and `add_io.rs`: Add Account… through porter's client), the screens in `list_view.rs`, `detail_view.rs`, `pane_view.rs` |
| A deletion the sync service holds for the person's word (the banner on a sync row, Keep and Remove) | `detent-model::{Holds, hold_step}` (pure), `detent-ui/src/live/holds.rs` (the Sync1 signal, `Watch`, `ConfirmDiscard`), `accounts/hold_drive.rs`; `HoldBanner` in `adapter.rs` |
| The marks of the providers shown | `detent-ui/src/marks.rs` (`ProviderMarks`, from porter's shipped provider files: a face, else the named mark, else `@`) |
| The local runtimes of the Intelligence page; the link from one to its account | `detent-model::AccountsPane::local_runtimes`, `detent-ui/src/inference/runtimes_view.rs`, `detent-ui/src/accounts/link.rs` (`AccountLink`: the page switches and the pane opens the account) |
| The first-run window, and whether it ran | `detent-model::{FirstRun, first_run_step}`, `detent-ui/src/first_run/` (the writes go through `files::apply` and inferd's picker), `detent-files::FirstRunMark` (`$XDG_STATE_HOME/detent/first-run`), the flag in `detent/src/cli.rs` and `main.rs` |
| Which pieces of the pane quire's R2a changes | `detent-ui/src/accounts/adapter.rs` (nothing else names them) |
| Looking for a daemon's live module, following its name and its changes | `detent-ui/src/live/` (`Daemon`, `follow`, `read_once`); `inference/` and `accounts/` only say which rows they read the values of |
| The accounts an agent may list | `settings.accounts.list` and `detent-intents::AccountsSource` (the binary reads them with `detent-ui::read_accounts`) |
| Where the built-in assistant may run: the places, the question before a cloud account is turned on, the models a place offers | `detent-model::{PlacesState, places_step, known_of, offers_for}` over `docket-settings`' types; the file in `detent-files::{load_places, save_places}` (one key of the `assistant` table at a time); the rows in `detent-ui/src/assistants/` |
| The person's other computers: the list (`Inference1.Places`, through porter-client; merged into docket's `assistant` table by `known_of`, so the switch is docket's and the existence porter's), Remove, the Add list of Tailscale lenders, the computers asking to borrow this one's models (answer, forget), and the refusal sentence the service sends | `detent-model::{ComputersState, computers_step, ComputerRow, known_of}` (pure; the guest and candidate rows are porter-core's `lending` types); the bus calls in `detent-ui/src/computers/io.rs` (all through `porter_client::Accounts`: `places`, `guests`, `candidates`, `answer_guest`, `forget_guest`, `add_tailnet_computer`, `remove_computer`, `watch_guests`; a refusal is `TransportError::Computer { words }`, shown as is), the one "Your computers" group drawn by `assistants/places_view.rs` (rows) and `computers/view.rs` (Add list, guests). There is no typed-address form |
| The outside assistants (Claude Code and the like), one row each: docket's facts (name, model, way of signing in, from `docket/agents.toml` and what each last offered) joined with accountd's (how the sign-in stands, the way to the account) | `detent-files::{load_agents, save_agent_choice}` (over `docket-settings`' `read_agents` and `write_agent_choice`; `DataHome` is where docket keeps the offers), `detent-model::{assistants_of, model_menu, way_menu, model_choice, way_choice}`, the rows in `detent-ui/src/assistants/outside_view.rs` (the way to the account is `AccountLink`) |
| accountd's accounts outside the Accounts pane | `detent-ui/src/accounts/watch.rs` (`use_listed`) |
| The desktop-wide Spaces, and what is asked before one is removed | `detent-model::{SpacesState, spaces_step, RemovalSheet}`; the bus calls in `detent-ui/src/spaces/io.rs` (memory's `RemoveSpace`, then the registry's `Remove`), the list in `list_view.rs`, the sheet in `sheet_view.rs` |
| Which bus inferd is looked for on, and where the footer's events come from | `detent-ui::Env` (`bus: BusSource`, `reasons: ReasonFeed`) |
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
  test did, not by how loaded the host is. `tests/ui/notices.rs` drives the inbox; `tests/viewer_page.rs`
  installs a scratch `anyview` schema for the Viewer page.
- `detent-intents/tests`: the provider served on a private `dbus-daemon` (`PrivateBus`), called from
  a connection that owns `org.quire.Intents1` (a provider answers nobody else), the way intentd calls
  an app. `detent/tests/install.rs` runs `scripts/install.sh` into a scratch `DESTDIR`.
- `detent-ui/tests/intelligence.rs` drives the Intelligence page: a stand-in schema for the companion
  (sill has none yet) tests the section order, `intelligence_programs.rs` the rows docket, almanac and cua
  ship (sections, Advanced, writes, the apps rows editor), `support/inferd.rs` is a
  fake `org.quire.SettingsModule1` on a private `dbus-daemon` (`support/bus.rs`), and
  `Sandbox::on_bus` points the app at it. Without it the app gets `BusSource::Absent`.
- `detent-ui/tests/accounts.rs` drives the Accounts page over `support/accountd.rs`, a fake
  accountd on the same kind of private bus (the list, a service switch, Revoke, Sign in again, the
  removal question, a dotted account id, a client id, no accountd, a refusal); it also writes
  pictures of the list, an account and the question to `$DETENT_SHOTS`.
- `detent-ui/tests/local_runtimes.rs` drives the Intelligence page's runtimes over the fake accountd (a
  runtime with a `place` row, a stopped one greyed, one that stops and is announced, the link to its
  account); `first_run.rs` drives the welcome window over a fake inferd and accountd on one bus (the
  default, off, a runtime picked, none found, a refusal, the mark); `detent/tests/first_run.rs` runs the
  binary with the mark present and sees it exit at once. Both draw pictures to `$DETENT_SHOTS`.
- `detent-ui/tests/assistants.rs` drives the assistants block over the fake accountd and a scratch `docket/settings.toml` (this computer without a switch, the question before a cloud account, Cancel, a computer of the person's, Ask me each time, the assistants that sign themselves in). `detent-ui/tests/spaces.rs` drives the Spaces list over `support/spaces.rs`, a fake registry and a fake memory on one private bus that write what they were asked, in order, into one log.
- `detent-ui/tests/assistants.rs` also drives "Your computers" over `support/computers.rs`, a fake inferd (`org.quire.Inference1`: Places, Candidates, AddTailnetComputer, RemoveComputer, Guests, AnswerGuest, ForgetGuest and the two signals) that logs what it was asked to change, in order, and can refuse the next change by name.
- `detent-ui/tests/account_notice.rs` drives `Notice::OpenAccount` over the fake accountd (an account's detail, an unknown one gives the list).
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
class matches its launcher entry. The porter daemons give it the Settings role by its unit, not by its name (sec-3): only the main
process of the user unit `org.quire.Settings.service` (`dist/systemd/org.quire.Settings.service`,
Type=dbus, BusName=org.quire.Settings, ExecStart=/usr/bin/detent) is Settings; the D-Bus service file
names it with `SystemdService=` so activation starts that unit, and `scripts/install.sh` puts both in
place. A detent started any other way (a terminal, an app scope) is an ordinary app with no Settings
role. porter's `dist/callers.toml` and `dist/inferd.toml` name the unit, which detent does not copy.

`dist/skills/settings/` is the companion's skill for this app (docket's skills format). Its `uses`
are `settings.key.read` and `settings.key.set` of the intents manifest, and its body lists the
agent-settable keys of design/22 section 9.7. Check it with
`docket-eval --check-skills <detent> <docket>`; the installer copies it to `share/quire/skills/settings/`.
