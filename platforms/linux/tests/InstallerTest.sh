#!/usr/bin/env bash
set -euo pipefail

# Keep installer writes inside each fixture HOME (CI runners export XDG_CONFIG_HOME).
unset XDG_CONFIG_HOME

LINUX_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

make_fixture() {
    local root="$1"
    mkdir -p "$root/lib" "$root/share/fcitx5/addon" "$root/share/fcitx5/inputmethod"
    printf 'addon' > "$root/lib/gonhanh.so"
    printf 'core' > "$root/lib/libgonhanh_core.so"
    cp "$LINUX_DIR/data/gonhanh-addon.conf" "$root/share/fcitx5/addon/gonhanh.conf"
    cp "$LINUX_DIR/data/gonhanh.conf" "$root/share/fcitx5/inputmethod/gonhanh.conf"
    cp "$LINUX_DIR/scripts/install.sh" "$root/install.sh"
    cp "$LINUX_DIR/scripts/gonhanh-cli.sh" "$root/gonhanh-cli.sh"
    printf '9.9.9\n' > "$root/VERSION"
}

assert_once() {
    local pattern="$1" file="$2"
    local count
    count="$(awk -v pattern="$pattern" '$0 == pattern { count++ } END { print count + 0 }' "$file")"
    if [[ "$count" != "1" ]]; then
        printf 'Expected exactly one %q in %s, found %s:\n' "$pattern" "$file" "$count" >&2
        sed -n '1,160p' "$file" >&2
        return 1
    fi
}

PACKAGE="$TEST_ROOT/gonhanh-linux"
make_fixture "$PACKAGE"

# Existing Bamboo profile remains byte-for-byte present around the appended item.
HOME_ONE="$TEST_ROOT/home-existing"
mkdir -p "$HOME_ONE/.config/fcitx5"
cat > "$HOME_ONE/.config/fcitx5/profile" <<'EOF'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=bamboo

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=bamboo
Layout=

[GroupOrder]
0=Default
EOF
GONHANH_SKIP_RESTART=1 HOME="$HOME_ONE" bash "$PACKAGE/install.sh"
GONHANH_SKIP_RESTART=1 HOME="$HOME_ONE" bash "$PACKAGE/install.sh"
assert_once "Name=bamboo" "$HOME_ONE/.config/fcitx5/profile"
assert_once "Name=gonhanh" "$HOME_ONE/.config/fcitx5/profile"
assert_once "# >>> Gõ Nhanh Fcitx5 >>>" "$HOME_ONE/.profile"
[[ -x "$HOME_ONE/.local/bin/gn" ]]
[[ "$(HOME="$HOME_ONE" "$HOME_ONE/.local/bin/gn" version)" == "Gõ Nhanh v9.9.9" ]]

# CLI selects GoNhanh specifically instead of reporting another active IM as GoNhanh.
FAKE_BIN="$TEST_ROOT/fake-bin"
mkdir -p "$FAKE_BIN"
cat > "$FAKE_BIN/fcitx5-remote" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
    -n) sed -n '1p' "$HOME/current-im" 2>/dev/null || true ;;
    -s) printf '%s\n' "$2" > "$HOME/current-im"; printf '2\n' > "$HOME/fcitx-state" ;;
    -c) printf '1\n' > "$HOME/fcitx-state" ;;
    -r) ;;
    "") sed -n '1p' "$HOME/fcitx-state" 2>/dev/null || printf '1\n' ;;
    *) exit 2 ;;
esac
EOF
chmod +x "$FAKE_BIN/fcitx5-remote"
printf 'bamboo\n' > "$HOME_ONE/current-im"
printf '2\n' > "$HOME_ONE/fcitx-state"
CLI_PATH="$HOME_ONE/.local/bin/gn"
status_output="$(PATH="$FAKE_BIN:$PATH" HOME="$HOME_ONE" "$CLI_PATH" status)"
[[ "$status_output" == *"○ TẮT"* && "$status_output" == *"bamboo"* ]]
PATH="$FAKE_BIN:$PATH" HOME="$HOME_ONE" "$CLI_PATH" on >/dev/null
[[ "$(sed -n '1p' "$HOME_ONE/current-im")" == "gonhanh" ]]
PATH="$FAKE_BIN:$PATH" HOME="$HOME_ONE" "$CLI_PATH" vni >/dev/null
[[ "$(sed -n '1p' "$HOME_ONE/.config/gonhanh/method")" == "vni" ]]
PATH="$FAKE_BIN:$PATH" HOME="$HOME_ONE" "$CLI_PATH" off >/dev/null
[[ "$(sed -n '1p' "$HOME_ONE/fcitx-state")" == "1" ]]
if PATH="$FAKE_BIN:$PATH" HOME="$HOME_ONE" "$CLI_PATH" unknown >/dev/null 2>&1; then
    printf 'Unknown CLI command unexpectedly succeeded\n' >&2
    exit 1
fi

# Empty and malformed profiles are preserved and receive one valid GoNhanh item.
for case_name in empty malformed; do
    case_home="$TEST_ROOT/home-$case_name"
    mkdir -p "$case_home/.config/fcitx5"
    if [[ "$case_name" == "malformed" ]]; then
        printf 'user-owned malformed line\n[Partial\n' > "$case_home/.config/fcitx5/profile"
    fi
    GONHANH_SKIP_RESTART=1 HOME="$case_home" bash "$PACKAGE/install.sh"
    assert_once "Name=gonhanh" "$case_home/.config/fcitx5/profile"
    if [[ "$case_name" == "malformed" ]]; then
        sed -n '/^user-owned malformed line$/p' "$case_home/.config/fcitx5/profile" | awk 'NF { ok=1 } END { exit(ok ? 0 : 1) }'
    fi
done

# XDG_CONFIG_HOME is shared by the CLI and the Fcitx engine method loader.
HOME_XDG="$TEST_ROOT/home-xdg"
XDG_DIR="$TEST_ROOT/xdg-config"
mkdir -p "$HOME_XDG"
GONHANH_SKIP_RESTART=1 HOME="$HOME_XDG" XDG_CONFIG_HOME="$XDG_DIR" bash "$PACKAGE/install.sh"
XDG_CLI="$HOME_XDG/.local/bin/gn"
PATH="$FAKE_BIN:$PATH" HOME="$HOME_XDG" XDG_CONFIG_HOME="$XDG_DIR" "$XDG_CLI" vni >/dev/null
[[ "$(sed -n '1p' "$XDG_DIR/gonhanh/method")" == "vni" ]]
[[ -f "$XDG_DIR/fcitx5/profile" ]]
GONHANH_SKIP_RESTART=1 PATH="$FAKE_BIN:$PATH" HOME="$HOME_XDG" XDG_CONFIG_HOME="$XDG_DIR" "$XDG_CLI" uninstall >/dev/null

# A custom prefix remains self-describing: gn finds version/uninstaller without
# requiring GONHANH_PREFIX to remain exported, and ~/.profile uses that prefix.
HOME_PREFIX="$TEST_ROOT/home-prefix"
CUSTOM_PREFIX="$TEST_ROOT/custom-prefix"
mkdir -p "$HOME_PREFIX"
GONHANH_SKIP_RESTART=1 GONHANH_PREFIX="$CUSTOM_PREFIX" HOME="$HOME_PREFIX" bash "$PACKAGE/install.sh"
CUSTOM_CLI="$CUSTOM_PREFIX/bin/gn"
[[ "$(HOME="$HOME_PREFIX" "$CUSTOM_CLI" version)" == "Gõ Nhanh v9.9.9" ]]
expected_path="export PATH=$CUSTOM_PREFIX/bin:\$PATH"
assert_once "$expected_path" "$HOME_PREFIX/.profile"

# An unmatched managed-block start marker is user data, not permission to
# discard the rest of ~/.profile during uninstall.
cat > "$HOME_PREFIX/.profile" <<'EOF'
export USER_SETTING=keep-before
# >>> Gõ Nhanh Fcitx5 >>>
export USER_SETTING_AFTER=keep-after
EOF
cp "$HOME_PREFIX/.profile" "$TEST_ROOT/unmatched-profile.expected"
GONHANH_SKIP_RESTART=1 PATH="$FAKE_BIN:$PATH" HOME="$HOME_PREFIX" "$CUSTOM_CLI" uninstall >/dev/null
cmp "$TEST_ROOT/unmatched-profile.expected" "$HOME_PREFIX/.profile"
[[ ! -e "$CUSTOM_CLI" ]]

# Uninstall removes only GoNhanh-owned state and leaves Bamboo/profile content.
GONHANH_SKIP_RESTART=1 PATH="$FAKE_BIN:$PATH" HOME="$HOME_ONE" "$HOME_ONE/.local/bin/gn" uninstall
assert_once "Name=bamboo" "$HOME_ONE/.config/fcitx5/profile"
if sed -n '/^Name=gonhanh$/p' "$HOME_ONE/.config/fcitx5/profile" | awk 'NF { found=1 } END { exit(found ? 0 : 1) }'; then
    printf 'GoNhanh profile entry survived uninstall\n' >&2
    exit 1
fi
[[ ! -e "$HOME_ONE/.local/bin/gn" ]]

printf 'Installer tests passed\n'
