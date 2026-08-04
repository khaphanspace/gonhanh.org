#!/usr/bin/env bash
# End-to-end self-test for the gonhanh Fcitx5 addon.
#
# Spins up a PRIVATE, isolated fcitx5 in its own dbus session with a throwaway
# config where gonhanh is the only input method, then drives it over DBus
# (selftest_dbus.py) to verify real telex -> Vietnamese conversion. It never
# touches the user's live fcitx5 / input session.
#
# Prereqs: the addon must be installed (make install-user) so gonhanh.so is in
# ~/.local/lib/fcitx5 and libgonhanh_core.so in ~/.local/lib. Needs python3-dbus.
#
# Usage:  ./selftest.sh          # exit 0 = all telex cases pass, non-zero = fail
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PY="$HERE/selftest_dbus.py"

USER_ADDON_DIR="$HOME/.local/lib/fcitx5"
SYS_ADDON_DIR="/usr/lib/x86_64-linux-gnu/fcitx5"
CORE_LIB="$HOME/.local/lib/libgonhanh_core.so"

if [ ! -f "$USER_ADDON_DIR/gonhanh.so" ]; then
  echo "FAIL: $USER_ADDON_DIR/gonhanh.so not found — run 'make install-user' first." >&2
  exit 2
fi
if [ ! -f "$CORE_LIB" ]; then
  echo "FAIL: $CORE_LIB not found — the Rust core is not installed." >&2
  exit 2
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Throwaway fcitx5 config: single group, gonhanh only.
mkdir -p "$TMP/fcfg/fcitx5"
cat > "$TMP/fcfg/fcitx5/profile" <<'EOF'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=gonhanh

[Groups/0/Items/0]
Name=gonhanh
Layout=

[GroupOrder]
0=Default
EOF

export FCITX_ADDON_DIRS="$USER_ADDON_DIR:$SYS_ADDON_DIR"
export LD_LIBRARY_PATH="$HOME/.local/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

# Run everything inside a private DBus session so the real fcitx5 is untouched.
LC_ALL=C.UTF-8 dbus-run-session -- bash -c '
  set -u
  export XDG_CONFIG_HOME="'"$TMP"'/fcfg"
  LOG="'"$TMP"'/fcitx5.log"
  # Disable GUI frontends/UI so it runs headless; only the dbus frontend is needed.
  fcitx5 --disable=wayland,waylandim,xim,x11,kimpanel,notificationitem,clipboard \
         -d >"$LOG" 2>&1 &
  FPID=$!
  # Wait (max ~10s) for fcitx5 to claim the bus name.
  for i in $(seq 1 50); do
    if gdbus call --session --dest org.freedesktop.DBus --object-path /org/freedesktop/DBus \
         --method org.freedesktop.DBus.NameHasOwner org.fcitx.Fcitx5 2>/dev/null | grep -q true; then
      break
    fi
    sleep 0.2
  done
  python3 "'"$PY"'"
  rc=$?
  kill "$FPID" 2>/dev/null
  wait "$FPID" 2>/dev/null
  exit $rc
'
