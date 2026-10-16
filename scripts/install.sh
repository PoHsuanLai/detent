#!/usr/bin/env bash
# Installs detent: the binary, its desktop entry, its intents manifest, the D-Bus service file
# that starts it when an intent addresses `org.quire.Settings`, and the systemd user unit that file
# names (the unit accountd and inferd know Settings by).
#
# usage: scripts/install.sh
# environment:
#   PREFIX       where it goes (default /usr). A PREFIX under $HOME (for example ~/.local) puts the
#                systemd user unit and the D-Bus service file in the per-user data directory,
#                ${XDG_DATA_HOME:-$HOME/.local/share}/systemd/user and .../dbus-1/services, because
#                systemd and the session bus search those and not $PREFIX/lib/systemd/user or
#                $PREFIX/share/dbus-1/services. Any other prefix (the default /usr)
#                keeps $PREFIX/lib/systemd/user and $PREFIX/share/dbus-1/services.
#   DESTDIR      a staging root the whole tree is put under (default none)
#   DETENT_BIN   the binary to install (default target/release/detent beside this repository)
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
prefix=${PREFIX:-/usr}
dest=${DESTDIR:-}
bin=${DETENT_BIN:-$repo/target/release/detent}
# `$prefix` as the replacement text of a `sed s|..|..|`: a `\`, a `|` or a `&` in it would otherwise
# be read as sed syntax and write a corrupt unit or service file.
sedprefix=$(printf '%s' "$prefix" | sed -e 's/[\\|&]/\\&/g')
share=$dest$prefix/share
# Where systemd and the session bus look: for a prefix under $HOME, the per-user data directory.
home=${HOME:-}
case $prefix/ in
  "${home:-/nonexistent}"/*)
    userdata=${XDG_DATA_HOME:-$home/.local/share}
    unitdir=$dest$userdata/systemd/user
    servicedir=$dest$userdata/dbus-1/services
    ;;
  *)
    unitdir=$dest$prefix/lib/systemd/user
    servicedir=$share/dbus-1/services
    ;;
esac

install -Dm755 "$bin" "$dest$prefix/bin/detent"
install -Dm644 "$repo/dist/org.quire.Settings.desktop" "$share/applications/org.quire.Settings.desktop"
# `$XDG_DATA_DIRS/quire/intents/<AppName>.toml`: what intentd reads (docket actions.md section 3.3).
install -Dm644 "$repo/dist/intents/org.quire.Settings.toml" "$share/quire/intents/org.quire.Settings.toml"
# `dbus-1/services/<AppName>.service` beside it, with the binary's installed path.
mkdir -p "$servicedir"
sed "s|^Exec=.*|Exec=$sedprefix/bin/detent|" "$repo/dist/dbus-1/services/org.quire.Settings.service" \
  >"$servicedir/org.quire.Settings.service"
chmod 644 "$servicedir/org.quire.Settings.service"
# The user unit the D-Bus file names (`SystemdService=`): porter gives Settings its role only to this
# unit's main process.
install -Dm644 "$repo/dist/systemd/org.quire.Settings.service" "$unitdir/org.quire.Settings.service"
sed -i "s|^ExecStart=.*|ExecStart=$sedprefix/bin/detent|" "$unitdir/org.quire.Settings.service"
