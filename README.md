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
