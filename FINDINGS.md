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

## Assistants and Spaces (`detent-ui/src/assistants/`, `spaces/`)

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
- **The Spaces list is read when the page opens.** `Spaces1.Changed` is not followed, so a Space an
  app makes meanwhile shows after the page is opened again.
- **Removal order.** `Control.RemoveSpace` goes to memory first and `Spaces1.Remove` second. Memory
  not being on the bus means nothing is settled and the registry is still asked.

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

## Lending this computer's models (`detent-ui/src/sharing.rs`, `lender.rs`)

- The switch `ai.tailnet.serve` ("Let my other computers use this computer's models") is not a
  plain write. `Window` hands an edit of that key to `SharingSwitch`, which runs
  `detent-model::sharing_step`: on asks first (an Alert: "Let your other computers use this
  computer's models?", Allow and Cancel), Allow calls porter's `TailnetLending::enable()` and only
  when that is `Ok` is the setting written; off calls `disable()` and writes "off" even when
  `disable` failed (the service answers nobody while the setting is off), telling the person why
  it failed. A failure is `LendingError`'s own plain words in a toast; the switch stays off.
- `LendingState::Differs` (a file of another making is there) adds one sentence to the question:
  allowing replaces it. `Installed` and `NotInstalled` ask the same question.
- `Lender` (in `Env`) is the one handle: `Lender::from_env()` by default (porter's
  `LendingConfig::from_env`: the shipped file `share/porter/inferd-tailnet.conf` is found under
  `XDG_DATA_HOME`/`XDG_DATA_DIRS`, the person's units under `XDG_CONFIG_HOME/systemd/user`; with
  no home folder every change answers with a sentence), `Lender::under_prefix(prefix, config_home)`
  for a fixed prefix, `Lender::over(config, manager)` for a test. The real manager is
  `SessionUnits` (session bus), connected when a change is made, never when Settings opens.
  There is no command-line flag for the prefix; `XDG_DATA_DIRS` is how detent finds schemas too.
- Tests (`tests/ui/lending.rs`) run over scratch folders and a fake `UnitManager`
  (`support/lending.rs`); `Sandbox` always hands the window that fake, so no test can reach the
  real systemd or `~/.config`.
- **One bad row hides every computer.** `Inference1.Places`, `Guests` and `Candidates` are decoded
  inside porter-client (`transport/dbus.rs`, `dbus_computers.rs`: `rows.into_iter().map(..).collect::<Result<..>>()`),
  so a single row with an invalid id fails the whole call and detent shows no computers. detent
  only sees the finished `Vec`, so it cannot skip the row; the fix belongs in porter-client (decode
  row by row, drop and log the bad one). Open until porter does it; no test here for it.

## Shortcuts (Keyboard page)

- The chord recorder is quire's `ShortcutField`, named only in `detent-ui/src/shortcuts/field.rs`.
  Clearing is its Delete key; there is no separate Clear button.
- `detent-files::KeymapFiles` wraps keycap's `KeycapSource` (keycap-client, by git rev, as casement
  and sill take it): this computer's layer (`machine()`, from the daemon or its file), the desktop's
  own bindings (`system_bindings`, the base of "Standard") and `watch`. The person's file is read
  as text here, not through `person()`, so a save keeps its comments and unreadable lines.
  Tests use `KeymapFiles::under`, which points the source at a socket that does not exist.
- Actions of apps are not listed; a change to one in the person's file is kept as it is.
- The page is listed only while some schema names it (sill's does); like Spaces it is not forced.

## Software Update (`detent-model/src/updates/`, `detent-ui/src/updates/`)

- **Not a fixed page of quire.** `ds_settings::schema::Page` has no Software Update variant, and detent
  does not edit quire. The catalogue holds the page as `Page::App("Software Update")`
  (`detent-model/src/updates/page.rs` is the only place that knows): its label is that name, its slug is
  `software-update` (hyphenated because slipway's notification runs `detent --page software-update`; the
  fixed pages use underscores), it is always listed (no file names it, like Accounts) and sits after
  Viewer. Closes when quire adds `Page::SoftwareUpdate`: delete that file's sentinel and the three
  guards on `Page::App` (`page.rs`, `layout.rs`, `icons.rs`).
- **No "General" group.** macOS has General > Software Update; detent's sidebar is flat, so the page is a
  sidebar row of its own. Closes when the sidebar has groups.
- **Wire words are parsed by hand.** `UpdatePhase::of_word` is a hand-written match: `detent-model` has no
  `ds-core` (the `Word` derive) and gaining it is a Cargo change. Closes with that dependency.
- **The service is looked for, never started.** slipway ships no D-Bus activation file, so the page reads
  `NameHasOwner` first; a service that starts later is followed (name changes and
  `PropertiesChanged` tick a re-read). The page says plainly that updates are not available when it is
  not there.
- **Log Out opens the shell's power menu** (`sill power-menu open`, which asks before it logs out). There
  is no bus method for logging out in detent or sill. When the command is missing or fails, a toast says
  to use the power menu of the desktop. Closes if sill offers a bus call for it.
- **"Last checked" is not ticked.** It is worked out when the page draws (on each service change and each
  visit), so "5 minutes ago" does not turn into "6 minutes ago" on its own while the page sits idle.
- **Rollback and the automatic-update options are not on the page** (owner: check daily, ask to install;
  the page states the daily check and has no switch).
- **A second `detent --page X` navigates the running window** (`ActivateAction("open-page", [slug])`, then
  a raise), so the notification's button lands on the page whether or not Settings was open.

## Assistant Model (`detent-model/src/assistant_model/`, `detent-ui/src/assistant_model/`)

Owner decision: the assistant (the planner) has no default model. Nothing is preselected or
recommended; the person chooses, or skips and is asked when a turn needs one.

- **The page and the step.** The page `assistant-model` ("Assistant Model") is listed after
  Intelligence and draws only the assistant's row of inferd's picker. The step `assistant-model-setup`
  is the same row as a short dialogue (heading, one sentence, Skip, Continue; no sidebar). Both open
  through `open-page` (`ActivateAction("open-page", [slug])`) and `detent --page <slug>`; the setup
  step is not a page of the catalogue, so the intents action `settings.page.show` refuses it as an
  unknown page. Like Software Update the page is `Page::App("Assistant Model")`, with the same
  closing condition (quire adding a variant).
- **Which row: the text slot at the balanced tier** (`ai.model.text.balanced`, an older module's
  `ai.model.llm.balanced`). `docket-planner`'s `PlannerModel::new` asks for `Tier::Balanced` ("the
  companion's is Balanced"), and the planner's need is text. One constant, `ASSISTANT_ROWS`, names it
  for the page, the step and the welcome window's runtime choice. Question for docket: if the companion
  ever asks for another tier (`with_tier`), that constant changes with it; nothing tells detent.
- **How sill should know the step was offered once.** Recommend the shell keeps the mark, in its own
  state directory, written when it opens the step (not when a button is pressed, so closing the window
  with its own close button still counts). detent cannot know why the shell opened it; inferd's file
  must not hold it, because an unset model is the very signal that asks again, and `FirstRunMark`
  (`$XDG_STATE_HOME/detent/first-run`) means "the welcome window ran". Recommend sill opens the step
  for both the first run and a turn that needs a model with none set or the chosen one unavailable
  (the one dialogue), and the page only for browsing. Question for the owner if the second should be
  the page instead.
- **Skip and Continue write nothing.** A pick writes at once, as everywhere in Settings (Continue
  waits until the row holds a model or Automatic and no write is in flight). Skip after a pick leaves
  the pick in force; "leaves it unset" holds when nothing was picked. Both call `AppHandle::quit`, so
  when the step is opened in a Settings window that was already running, the whole window closes. The
  harness has no app handle, so the closing itself is unchecked by tests (manual check below).
- **The empty word reads "None", never the module's name for it.** inferd labels it "Catalogue
  default"; detent shows "None" in the menu whatever the module says, shows the button as "Choose..."
  with no item marked while the row holds it, and puts no badge or ordering on any model. Every tier of
  every slot is shown this way, though for the other slots the empty word still means the catalogue's
  choice. Closes when inferd labels the empty word itself (then `NONE_LABEL` goes).
- **Automatic stays a choice** and counts as chosen. Question for the owner: should the assistant's
  menu offer it at all, as it is a rule and not a model.
- **A model that needs an account stays pickable.** Picking it opens the Add Account sheet (accountd's,
  through porter's client) instead of setting; when an account was added the rows are read again and
  the pick is made if the model can now be picked, else dropped. The test for "needs an account" is the
  module's reason text compared whole (`NEEDS_ACCOUNT`, inferd's `cloud::picker::NEEDS_ACCOUNT`).
  Closes when porter-infer exports that text or a typed reason, so detent stops copying it.
- **The welcome window (`detent --first-run`) still has a "Local model" row** that leaves the text slot
  to "Automatic" when untouched. It is a second place that picks the assistant's model and its
  default wording claims one. Recommend removing the row (keeping the local-only switch) once sill
  opens the step; left alone here because the welcome window is another flow with its own tests.
- **The running window does not resize.** `detent --page assistant-model-setup` starts at the step's
  size (440 by 520); a Settings window already open keeps its size and centres the step in it.

## Memory settings that wait for confirmation (`detent-model/src/memory_consent/`, `detent-ui/src/memory/`)

- **The rule is memory's.** A memory setting looser than the shipped one (a retention or pending
  time below the shipped days, `memory.files.at_rest` = plain, `memory.consolidation.apply` = auto
  where review ships) takes effect only after the person confirms it; until then memory keeps the
  shipped value. detent holds no copy of which values are loose: it asks memory.
- **Saving is the confirmation.** After a save of a key in `almanac/settings.toml` (any key that file's
  schema names; `is_memory_setting`), the window reads `Memory1.Control.PendingSettings` and sends
  `ConfirmSettings` with exactly the pair just saved when memory lists that key with that value as
  waiting (`to_confirm`). A value that is not looser is not listed, so nothing is sent. The read
  comes after the file write, and memory answers from the files as they are now.
- **A value typed into the file** shows under its row "Waits for you to confirm. Until then, memory
  keeps <shipped value>." and a Confirm button that sends that one pair. The list is read each time
  a page holding memory settings is shown and after every save and confirmation, so a hand edit
  shows on the next visit to the page, not while the page stays open (the same limit as "External
  edits" below).
- **Words.** The shipped value is said in the row's own words: the schema's label in curly quotes
  for a choice (“Private”), the number and unit for a count (`30 days`). A refusal is one
  calm line under the note: "Memory would not take this confirmation from here." (NotAllowed, or the
  bus refusing), "Memory could not accept this value, so it keeps its own." (Invalid) or "This
  could not be confirmed. Try again in a moment." (anything else). The value keeps waiting.
- **Memory not running.** Neither call starts memory (`NameHasOwner` first): with it absent the page
  has no note and a save sends nothing. A loose value saved then waits, and its note and Confirm
  show the next time the page is shown with memory running.
- **Only Settings may confirm.** memory accepts `ConfirmSettings` from the unit
  `org.quire.Settings.service` (porter role `settings`, app `org.quire.Settings`) alone. A detent
  started any other way (a terminal) gets NotAllowed, and says the line above. Nothing in detent's
  `dist/` needs a new row: it already installs that unit and the D-Bus file that names it. The
  desktop installer must install almanac's `dbus/memory-callers.toml` as
  `/etc/quire/memory-callers.toml`, which carries the Settings row.
- **The proxy.** `org.quire.Memory1.Control` is declared once in `detent-ui/src/memory/proxy.rs`
  (`RemoveSpace`, `ConfirmSettings`, `PendingSettings` and the `Confinement` property) and the
  Spaces page uses it too. almanac's `almanac-dbus` is not a dependency: detent keeps its own
  small proxy, as it had for `RemoveSpace`, and `almanac-core` supplies `WaitingSetting`.

## D21: whether memory keeps to its own folders (`detent-model/src/memory_consent/confinement.rs`, `detent-ui/src/memory/`)

- **The fact is memory's.** almanac (ec45957) serves the read-only property
  `org.quire.Memory1.Control.Confinement` (type `s`) with four stable words: `confined`,
  `partial`, `not-available` and, in test builds only, `off`. It does not change while memory
  runs. detent reads it and decides nothing.
- **Parsed once.** `Confinement::from_word` turns the word into `Confined`, `Partial`,
  `NotAvailable`, `Off` or `Unknown` (any other word, and also "memory did not answer": neither
  gets a line). It is hand-written rather than a `Word` for that catch-all. `Confinement::line`
  is the only place the sentences live.
- **One line, under the Memory section.** `confined`: "Memory keeps itself to its own folders".
  `partial`: "Memory keeps itself to its own folders, though this computer can only enforce part of
  that". `not-available`: "This computer can't keep memory to its own folders". `off` and
  unknown: no line. It is a quiet footnote (`LabelRole::Secondary`, `LabelStyle::Footnote`) in
  the `consent-note` block that already sits under a row, drawn by `SectionView` after the card
  when `tells_confinement` says the section is "Memory" and holds memory settings (so not the
  History section, nor an Advanced group). No new CSS, no colour. The words name no kernel
  feature, bus or protocol.
- **Read like the other memory calls.** `MemoryConsent::look_at_confinement` runs beside
  `refresh` each time a page holding memory settings is shown, through `NameHasOwner` first, so
  opening Settings never starts memory. Memory absent, a failed read, or an older memory without the
  property leaves `Unknown`, so the page has no line. The word is kept in `ConsentState`
  (`confined`, `confinement`), which a new list of waiting values leaves alone.
- **The proxy.** The property is declared in `detent-ui/src/memory/proxy.rs`, the one declaration
  of `Memory1.Control` in detent (almanac's `ControlProxy::confinement()` lives in `almanac-dbus`,
  which detent does not depend on).
- **Tests.** `detent-model/tests/model/memory_consent.rs`: a table of words to values and lines,
  the state keeping the word, and which sections tell. `detent-ui/tests/ui/memory_consent.rs`
  over the fake memory, which now carries a `Confinement` word: confined, part enforced and not
  available show their lines, `off` and an unknown word show none, and no memory shows none.
- Not checked here: these changes were written without a build; this repo's gate compiles and runs
  them. The real memoryd on a real session is a manual check (below).

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

- The Memory section's confinement line against the real memory service: on a normal kernel it
  reads "Memory keeps itself to its own folders"; with memory stopped there is no line. The tests
  use a stand-in service.
- `detent` on a real session: opens, follows the desktop theme, a toggle on the Dock page changes
  `~/.config/sill/settings.toml` and sill's dock reacts. Only the harness has run the window: a
  launch of the binary on scratch directories reached window creation and stopped with "no
  adapter found" (the build sandbox had no Vulkan device), so the real window is unchecked.
- The Software Update page against the real slipway service: Check for Updates, Update Now through the
  download and the install, "Log Out..." opening the shell's power menu, and the notification's button
  opening this page in a running Settings. The tests use a stand-in service and a counted log-out only.
- The assistant's first-run step on a real session: `detent --page assistant-model-setup` closes the
  window on Skip and on Continue, and the same slug sent to a running Settings through `open-page`
  shows the step. The tests have no window to close.
