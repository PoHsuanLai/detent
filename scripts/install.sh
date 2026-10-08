#!/usr/bin/env bash
# Installs detent: the binary, its desktop entry, its intents manifest, the D-Bus service file
# that starts it when an intent addresses `org.quire.Settings`, and the systemd user unit that file
# names (the unit accountd and inferd know Settings by).
#
# usage: scripts/install.sh
# environment:
#   PREFIX       where it goes (default /usr)
#   DESTDIR      a staging root the whole tree is put under (default none)
#   DETENT_BIN   the binary to install (default target/release/detent beside this repository)
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
prefix=${PREFIX:-/usr}
dest=${DESTDIR:-}
bin=${DETENT_BIN:-$repo/target/release/detent}
share=$dest$prefix/share

install -Dm755 "$bin" "$dest$prefix/bin/detent"
install -Dm644 "$repo/dist/org.quire.Settings.desktop" "$share/applications/org.quire.Settings.desktop"
# `$XDG_DATA_DIRS/quire/intents/<AppName>.toml`: what intentd reads (docket actions.md section 3.3).
install -Dm644 "$repo/dist/intents/org.quire.Settings.toml" "$share/quire/intents/org.quire.Settings.toml"
# `dbus-1/services/<AppName>.service` beside it, with the binary's installed path.
mkdir -p "$share/dbus-1/services"
sed "s|^Exec=.*|Exec=$prefix/bin/detent|" "$repo/dist/dbus-1/services/org.quire.Settings.service" \
  >"$share/dbus-1/services/org.quire.Settings.service"
chmod 644 "$share/dbus-1/services/org.quire.Settings.service"
# The user unit the D-Bus file names (`SystemdService=`): porter gives Settings its role only to this
# unit's main process.
install -Dm644 "$repo/dist/systemd/org.quire.Settings.service" "$dest$prefix/lib/systemd/user/org.quire.Settings.service"
sed -i "s|^ExecStart=.*|ExecStart=$prefix/bin/detent|" "$dest$prefix/lib/systemd/user/org.quire.Settings.service"
