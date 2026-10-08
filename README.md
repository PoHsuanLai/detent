# detent

The desktop's Settings app (`org.quire.Settings`): one window on Blitz, drawn with quire's
components. Its rows come from each app's settings schema, and its Accounts and Intelligence
pages talk to the account service and the inference service.

## Install

`scripts/install.sh` puts in place the binary, the desktop entry, the intents manifest, the
D-Bus service file and the systemd user unit `org.quire.Settings.service`.

## Running as Settings

The account service gives the Settings role (reading and changing accounts, sign-in keys and sync
decisions) only to the main process of the user unit `org.quire.Settings.service`. D-Bus
activation starts that unit, so open Settings from the desktop or through an intent. A detent
started any other way (from a terminal, from an app's scope, from another unit) is an ordinary app:
it gets no Settings role, and its Accounts page says it may not be shown the accounts.

## D-Bus surface

On the session bus detent owns `org.quire.Settings` and serves two interfaces:

- `org.quire.IntentProvider1` at `/org/quire/IntentProvider1` (the desktop's intents).
- `org.freedesktop.Application` at `/org/quire/Settings` (D-Bus activation; the desktop entry
  says `DBusActivatable=true`):
  - `Activate(platform_data)` shows and raises the window (`activation-token` is honoured).
  - `Open(uris, platform_data)` answers NotSupported: Settings opens no files.
  - `ActivateAction("open-account", [account object path], platform_data)` shows the Accounts
    page at that account (`/org/quire/Accounts1/account/<segment>`); an unknown account shows the
    list, and an unknown action is logged and ignored.

A second `detent` finds the name taken, calls `Activate` on the running one and exits.
