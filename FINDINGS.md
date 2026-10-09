# Findings

Open items and standing facts. An entry names the condition that closes it.

## quire gaps (detent does not work around them by editing quire)

### KeyKinds with no quire component

`ds_settings::schema::KeyKind` has nine kinds; seven have a component (Fixed: `Label`, Toggle:
`Toggle`, Segmented: `SegmentedControl`, Menu: `PopUpButton`, Bounded: `Slider`, Text: `TextField`,
Colour: `RadioGroup` with `Arrangement::Swatches`). Two have none (and a third, Rows, is composed below) and render read-only with an
"Edit in file" note (`detent-ui/src/controls/file_only.rs`):

- **Shortcut** (a captured key chord): quire has no key-capture field. Closes when quire adds one.
- **Rows** (`Rows { columns }`, cua's `cua.apps`) is edited in `detent-ui/src/controls/rows.rs`, composed of quire's
  `TextField`, `PopUpButton` and `Button`: quire has no rows-editor component of its own, so a
  component there would replace it. Rows the editor cannot hold (an unknown cell, a word outside
  a column) are shown read-only and kept byte for byte.
- **List** (`List(Box<KeyKind>)`): quire has no list editor. Today three sill keys
  are lists (`control_center.modules`, `widgets.center`, `widgets.desktop_widgets`).

Colour is drawn with what quire has and it fits badly: a `Choice` swatch names a CSS custom property
(`swatch_var(Accent)`), so only the eight accents exist as swatches. detent draws those eight for a
Colour key and for any Menu or Segmented key whose words are all accent names
(`appearance.accent`), and stores the accent's word. A free hex colour has no control. No Colour
key exists in any schema yet. Closes when quire has a colour well or a swatch that takes a value.

### Untyped settings I/O

`ds_settings::Store::load` and `save` need a typed `SettingsDoc` with a `const FILE`, and the lenient
reader (`lenient::read`) is crate-private. A schema-driven app knows no struct. `detent-files`
therefore re-implements, small and tested, the two rules that matter: the lenient overlay (a value
the control cannot hold is kept as written and shown read-only, unknown keys reported by dotted path
cut at the first unknown table) and a save that sets one key in place with `toml_edit` through an
atomic temp-and-rename (`.part` beside the file, as `Store::save` does). It returns
`ds_settings::Loaded`, `UnknownKey` and `InvalidKey` so callers see quire's types. Closes when
`ds-settings` offers a table-level `Store::load_table`/`save_table` (or a `RawDoc`) that takes the
schema's keys.

### ds-settings drags zbus into a pure crate

`detent-model` and `detent-files` use only `ds_settings::schema` and `Loaded`, but `ds-settings`
builds zbus for its portal watch, so both reach zbus and `scripts/check-boundary.sh` cannot forbid it
(it forbids dioxus, tokio, blitz-dom and the ds crates). Closes when quire splits the schema module
and the lenient types into a crate with no zbus (or gates the portal behind a feature).

### Schema discovery reports stale files only on stderr

`ds_settings::schema::discover` prints "skipping stale schema" for a file that does not parse and
returns nothing about it. detent adds its own warnings as data (`Warning`: unsupported version, a
`file` that is not `<app>/<name>.toml`, an app an earlier directory already supplied) but a parse
failure is only on stderr. Nothing detects "a schema whose program is gone" (design/22 section 9.2),
because a schema names no binary. Closes when `discover` returns the skipped files and their reason.

### Smaller quire items

- `Slider` sends `onchange` on every move and has no commit event, so a drag rewrites the file many
  times (detent sends only a changed value, which for a whole range is once per unit) and it draws no
  value; detent adds a `Label` beside it.
- `Disclosure` without `common.id` gives every instance the id `ds-disclosure-body`; detent passes an
  id per section. quire should derive a unique one.
- `deep_link` in `ds_settings::schema` answers only a unique match; detent's search ranks (label,
  help, then every word) and highlights all hits. The launcher's provider still uses `deep_link`.
- design/22 section 2 says every file has a top-level `version = 1`. `quire/appearance.toml` has it
  (`AppearanceFile::version`), `sill/settings.toml` does not. detent carries `version` through a
  save when the file has one and never adds it.
- `Page` has no Spotlight page (design/22 section 5 lists one); sill's launcher keys are on
  Appearance. `Apps` is a page nothing fills.

## The Intelligence page

- **Schemas read:** inferd's (`fixtures/schemas/inferd.settings.toml`, 21 rows, hands-off), docket's
  (18 rows: Privacy basic; Review time, Session budgets and Task approvals Advanced), almanac's
  (8 rows: Memory basic, Retention Advanced) and cua's (14 rows: Computer use basic; Run limits,
  Timing and Window detail Advanced). Each is installed to `$XDG_DATA_DIRS/quire/settings/` and
  writes its own `$XDG_CONFIG_HOME/<program>/settings.toml`. Not shipped anywhere yet: sill's
  `companion.*` (sill's `dist/` has no companion schema); the Companion section appears when a
  schema names it, and the layout test uses a stand-in. Closes when sill ships its schema.
- **cua's apps are rows, but "never `always` for a browser" is not enforced here.** The rows editor
  checks each cell against its column (the whole value matches the pattern, a choice is one of its
  words); which apps are browsers is cua's to decide when it reads the file.
- **The reason footer has no producer.** `Reason` reads `InferEvent::Why` and `Declined` from the
  `ReasonFeed` in `Env`, but `Why` is an event of an inferd session, which only that session's caller
  sees. The binary starts with a silent feed, so the footer shows its resting line. Closes when
  inferd (or the companion, which holds the session) offers the last pick to Settings.
- **The picker follows inferd.** It subscribes to `NameOwnerChanged` for inferd's bus name before
  asking, re-reads the module when the name appears and shows the "unavailable" card when it
  vanishes. It never starts inferd (`NameHasOwner` first). A daemon that takes its name before it
  serves the module is read once and shown as failed until its name changes hands again.
- A footer under a `FieldGroup` is drawn by hand (a `Label` in `.picker-reason`): quire's
  `FieldGroup` has no footer slot.
- A "Choose Model..." button beside a declined model is not drawn: the picker is on the same page.

## palmrest's schema

palmrest writes `palmrest.settings.toml` (`palmrest --write-schema <dir>`), one row for every key
of `gestures.toml`, so a save from detent drops nothing palmrest reads. Two things to know:

- The keys are palmrest's own, not design/22 section 3.9's names: lengths are micrometres
  (`scroll.lock_threshold_x_um`), ellipses are x/y pairs, there is no `swipe.workspace_mode`, and
  the taps are six advanced sliders, not one "double-tap sensitivity" control. Mouse & Gestures
  shows what palmrest reads: palm rejection, scrolling in other apps and the six gesture actions
  are Basic; the rest is under Advanced.
- A gesture's action is a `Menu` of the words that need no value (`back`, `forward`,
  `workspace_prev`, ...). An action with a value (`{ kind = "keys", v = "alt+left" }`, a command, a
  shell request) is not one of the words, so detent shows it read-only with "Edit in file" and every save keeps it
  byte for byte. Closes when quire has a kind for a tagged value, or a
  shortcut field detent can use for `keys`.

## Intents (`org.quire.Settings`)

`dist/intents/org.quire.Settings.toml` declares six actions (`settings.page.show`,
`settings.key.read`, `settings.key.set`, `settings.accounts.list`, `settings.accounts.open`,
`settings.intelligence.open`);
`detent-intents` serves them while the window runs.
Open items, each with its closing condition:

- **A schema cannot mark a key agent-settable.** `ds_settings::schema::KeySpec` has no such field, so
  detent reads `agent = "settable"` from each `[[key]]` table of the schema files itself
  (`detent-model/src/agent.rs`; `KeySpec`'s deserialiser ignores the extra field), and a key without
  the mark is hands-off. No installed schema carries the mark yet. Closes when quire adds
  `KeySpec::agent` (and the derive's `#[settings(agent_settable)]`); then `agent.rs` is deleted and
  `discover` reads the field.
- **No headless activation.** The bus file starts `detent`, which opens its window; a read or a set
  that arrives by D-Bus activation therefore shows Settings. A background provider needs
  `ds_blitz::launch_idle` with an `AppHandle` that opens the window only for `settings.page.show`.
- **No single instance, and the window is not raised.** A second `detent` loses the bus name and runs
  without the service, so intents reach the first. `settings.page.show` switches the page of the
  running window but does not focus it (quire has no window-raise request in `ds-blitz`).
- **A key path two programs share names no key.** `Catalogue::entry` answers only an unambiguous
  path; the entity key of `settings.key` is the dotted path alone. Closes if two shipped schemas ever
  reuse a path (then the key becomes `<file>:<path>`).
- **Undo needs the same process.** The journal is in memory (256 changes), so an undo after Settings
  restarted is `Gone`. Undoing a key the file never held writes its default explicitly, since detent
  never removes a key. A value changed since the set is `Conflict` and is left alone.
- **Read results are labelled untrusted (a file's content).** A free-text key can hold anything, so
  the manifest declares `third_party(file)` for `settings.key.read` and every value carries it.
- **The Viewer page has no real schema to draw.** anyview has no `--write-schema` yet (its FINDINGS wait
  for quire's `viewer.*` rows), so `tests/viewer_page.rs` installs a scratch schema of the keys
  anyview's FINDINGS name. Closes when anyview ships `anyview.settings.toml`: replace the scratch schema
  with a fixture from `scripts/gen-fixtures.sh`.
- **No `org.quire.Settings.ui.toml`.** The menu and shortcut rule of cli.md section 6 has nothing to
  check until ds menu items carry an action id (docket-eval says so as a note).

## Accounts pane (`detent-ui/src/accounts/`)

- **The pane's quire seam is `accounts/adapter.rs`.** It holds the button roles, the alert, and the
  client-id `TextField` that keeps its own draft (and is made anew when the stored value changes,
  because quire's `TextField` is not controlled). Closes when quire's R2a lands: the adapter takes
  the controlled field.
- **The list is grouped by the `group` row** (Internet Accounts, Intelligence, Assistants), a row per
  account with a plus and a minus under it; the minus removes the selected account after the usual
  question. An account whose module sends no `group` row is placed by its own rows. A permission's
  `since` and `scope` rows are keyed by the grant (`accounts.<id>.grant.<grant>.since`).
- **Search does not reach the live rows.** The sidebar's search reads the file schemas, so no account
  or service is a hit; the Accounts page is listed only while the search box is empty or matches a
  file key of that page. Closes when a live module's rows can be searched (needs the pane to hand
  its rows to `search`).
- **The manifest declares no mutation at all**, rather than declaring them `reach = "hidden"`: an
  action that is not declared cannot be offered, called or confused with one that is. The list gives
  label, kinds and state per account and nothing that names or reaches one (no id, no grant);
  `detent-intents/tests/accounts.rs` holds that.

## Assistants (`detent-ui/src/assistants/`)

- **There is no Assistants page.** A fixed page is a variant of `ds_settings::schema::Page`, which is
  quire's to add. Until it is, the assistants are a block of the Intelligence page, right after
  Companion (`Block::Assistants`). The runtimes group that was "On this computer" is now "Models on
  this computer", because the assistant's own group has that name. Closes when quire adds the page
  variant and the block moves to its own page.
- **No source lists a person's other computers.** `docket_settings::PlaceSource` has no
  implementation; "Your computers" shows the rows the settings file already holds, each off or on as
  the file says. Cloud accounts come from accountd (signed-in accounts of the Intelligence group).
  Closes when inferd or accountd lists computers.
- **A place offers the models of its own accounts.** The Advanced model picker lists inferd's picker
  words (`<account>/<model>`) that belong to the place's accounts: the local runtimes for this
  computer, the account of that name for a cloud row, none for a computer of the person's.
- **The assistants that sign themselves in have a row and the way to their account, not a model
  picker.** Their model is a `model` key of an `[[agent]]` entry in docket's `agents.toml`, which is
  keyed by the program, and nothing maps an accountd account to that entry or lists the models an
  agent offers (it says them in a live session). Closes when docket publishes both.

## Local runtimes and first run (`detent-ui/src/inference/runtimes_view.rs`, `first_run/`)

- **Where the runtimes list comes from.** The Intelligence page's "Models on this computer" lists accountd's
  accounts whose `accounts.<id>.place` row says `this_computer` and that serve `llm`
  (`AccountsPane::local_runtimes`); inferd reports each probed runtime to accountd (`Peer.ReportLocal`),
  so a stopped one stays as an `Offline` account and is drawn greyed (a muted mark, "Offline") with its
  link. accountd serves no `place` row yet, so the list says "No local runtime is running" against
  today's accountd. Closes when porter takes the ask `~/rs-wt/accounts/ai1b-porter-place-row.diff`
  (the row, and one service switch per kind instead of one per model claim, which duplicated the
  `accounts.<id>.service.llm` row of a runtime with several models; detent keeps the first of a
  repeated path meanwhile).
- **A runtime going offline reaches the row through `Changed`.** accountd announces
  `Changed("accounts.<id>.state", ...)` where it publishes the registry; the list reads the rows
  again on every announcement and when accountd's name changes hands. A daemon that stopped
  announcing would leave the row as it was until the page is opened again.
- **First run remembers itself in a state file, not a settings row.** `$XDG_STATE_HOME/detent/first-run`
  (else `~/.local/state`), one line `done`, written when Done has written everything; a settings row
  would need a schema that owns it, and the first run is not a preference. `detent --first-run` exits
  at once, before reading anything, when the file exists. A window closed without Done leaves no
  file, so it asks again at the next login. sill launches it once per session
  (`~/rs-wt/accounts/ai1b-sill-first-run.diff`, an ask against `dist/sill-session`); until then it is
  run by hand.
- **Choosing a runtime writes one row.** The runtime menu sets the text slot's Balanced tier
  (`ai.model.text.balanced`, or `ai.model.llm.balanced` of an older inferd) to the first model of that
  runtime the picker lists that can be picked; Fast and Best stay at the catalogue's choice and
  Automatic still routes by privacy and load. A runtime that lists no model yet, or an inferd that
  is not running, keeps the window open with the reason; `ai.local_only` is written first and again
  on the next Done. A stopped runtime is greyed in the menu and cannot be picked.
- **The window's size is fixed by the binary** (440 by 520) and the hero mark is quire's violet plate
  with the sparkles glyph; there is no round hero mark in quire to use instead (`Avatar` holds a
  letter, `ProviderMark` is a square letter chip), so the runtimes' marks are `Avatar` discs in the
  person-hue of the runtime's name, and a stopped one is `Muting::Muted`. Closes when quire has a
  mark for a local runtime.
- **`settings.intelligence.open`** only shows the page. Every `ai.*` row is hands-off
  (`detent-intents/tests/intelligence.rs`), and the manifest has no other `settings.intelligence.*`
  action.

## Not in v1

- **Live modules** (design/22 section 9.4, `org.quire.SettingsModule1` over D-Bus): only inferd's
  model picker (Intelligence page, `detent-ui/src/inference/`) and accountd's Accounts pane
  (`detent-ui/src/accounts/`) are drawn, both over the one look-for-it-and-follow-it loop in
  `detent-ui/src/live/`. Wi-Fi, Bluetooth, sound and displays' live state have no page. Closes
  when sill's services serve `Describe/Get/Set/Changed` and a page asks for them.
- **External edits**: values are read once at start, and again when the intents provider tells the
  window it wrote a key (`Notice::Reload`). A change made by another program or an editor
  shows after a restart (only `quire/appearance.toml` is watched, by `use_environment`, so the window
  follows the theme live). `Store::watch` is typed, so a schema-driven watch needs the same
  untyped API as above.
- **Unknown and invalid keys** are printed on stderr at load (`detent-ui/src/files.rs`) and left in
  the file untouched by every save (detent is not the owning program). A value a control cannot hold
  is shown read-only with "Edit in file". There is no other in-window notice.
- A search hit is highlighted and its disclosure opened, but the page is not scrolled to it.
- The window's size and the sidebar's width are not remembered; there are no back and forward buttons.

## Manual checks

- `detent` on a real session: opens, follows the desktop theme, a toggle on the Dock page changes
  `~/.config/sill/settings.toml` and sill's dock reacts. Only the harness has run the window: a
  launch of the binary on scratch directories reached window creation and stopped with "no
  adapter found" (the build sandbox had no Vulkan device), so the real window is unchecked.
