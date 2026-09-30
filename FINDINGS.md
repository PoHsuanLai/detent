# Findings

Open items and standing facts. An entry names the condition that closes it.

## quire gaps (detent does not work around them by editing quire)

### KeyKinds with no quire component

`ds_settings::schema::KeyKind` has nine kinds; seven have a component (Fixed: `Label`, Toggle:
`Toggle`, Segmented: `SegmentedControl`, Menu: `PopUpButton`, Bounded: `Slider`, Text: `TextField`,
Colour: `RadioGroup` with `Arrangement::Swatches`). Two have none and render read-only with an
"Edit in file" note (`detent-ui/src/controls/file_only.rs`):

- **Shortcut** (a captured key chord): quire has no key-capture field. Closes when quire adds one.
- **List** (`List(Box<KeyKind>)`, a rows editor): quire has no rows editor. Today three sill keys
  are lists (`control_center.modules`, `widgets.center`, `widgets.desktop_widgets`).

Colour is drawn with what quire has and it fits badly: a `Choice` swatch names a CSS custom property
(`swatch_var(Accent)`), so only the eight accents exist as swatches. detent draws those eight for a
Colour key and for any Menu or Segmented key whose words are all accent names
(`appearance.accent`), and stores the accent's word. A free hex colour has no control. No Colour
key exists in any schema yet. Closes when quire has a colour well or a swatch that takes a value.

### Two-word enums that are not on and off

design/22 section 9.1 makes every two-variant enum a `Toggle`. That reads well for `on`/`off`,
`natural`/`traditional` and `show`/`hide`, and badly for `ring`/`bar`, `top_right`/`bottom_right`,
`sheet`/`popover`, `cosmic`/`shell` (about a third of sill's toggles). detent follows the table
and picks the on side with `TogglePair::of` (the first word, unless it is `off`, `hide`, `never`,
`none` or `nothing`). `SegmentedControl` takes two segments, so the table could send a pair with
no off-word to it. Closes when design/22 says so.

### Untyped settings I/O

`ds_settings::Store::load` and `save` need a typed `SettingsDoc` with a `const FILE`, and the lenient
reader (`lenient::read`) is crate-private. A schema-driven app knows no struct. `detent-files`
therefore re-implements, small and tested, the two rules that matter: the lenient overlay (a bad
value costs its own key, unknown keys reported by dotted path cut at the first unknown table) and the
atomic temp-and-rename write (`.part` beside the file, as `Store::save` does). It returns
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

## palmrest has no schema

palmrest ships no `--write-schema`, so `gestures.toml` has no keys in detent: Mouse & Gestures shows
only sill's `scroll.*` and `scrollbar.*` (one Basic key, natural scrolling). design/22 section 5 also
lists `swipe.*`, `tap.*`, `rejection`, `foreign_output` and the gesture remap table for this page.
Closes when palmrest writes a schema (`ds_settings::schema::maybe_write_schema`).

## Not in v1

- **Live modules** (design/22 section 9.4, `org.quire.SettingsModule1` over D-Bus): Wi-Fi, Bluetooth,
  sound, displays' live state, accounts. No page for them, no D-Bus client in detent. Accounts shows
  only `session.*` keys from the file. Closes when sill's services serve `Describe/Get/Set/Changed`.
- **External edits**: values are read once at start. A change made by another program or an editor
  shows after a restart (only `quire/appearance.toml` is watched, by `use_environment`, so the window
  follows the theme live). `Store::watch` is typed, so a schema-driven watch needs the same
  untyped API as above.
- **Unknown and invalid keys** are printed on stderr at load (`detent-ui/src/files.rs`) and never
  written back. There is no in-window notice.
- A search hit is highlighted and its disclosure opened, but the page is not scrolled to it.
- The window's size and the sidebar's width are not remembered; there are no back and forward buttons.

## Manual checks

- `detent` on a real session: opens, follows the desktop theme, a toggle on the Dock page changes
  `~/.config/sill/settings.toml` and sill's dock reacts. Only the harness and one scratch-environment
  launch have been run here.
