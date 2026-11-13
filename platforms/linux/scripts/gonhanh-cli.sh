#!/usr/bin/env bash
set -euo pipefail

SOURCE="${BASH_SOURCE[0]}"
while [[ -L "$SOURCE" ]]; do
    DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
    SOURCE="$(readlink "$SOURCE")"
    [[ "$SOURCE" != /* ]] && SOURCE="$DIR/$SOURCE"
done
SCRIPT_DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
PREFIX="${GONHANH_PREFIX:-$(cd "$SCRIPT_DIR/.." && pwd)}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
STATE_DIR="$PREFIX/share/gonhanh"
METHOD_FILE="$CONFIG_HOME/gonhanh/method"
VERSION="$(sed -n '1p' "$STATE_DIR/version" 2>/dev/null || true)"
VERSION="${VERSION:-unknown}"

usage() {
    cat <<'EOF'
Gõ Nhanh - Vietnamese Input Method

Cách dùng: gonhanh [lệnh] (hoặc: gn [lệnh])

  (không tham số)  In trạng thái hiện tại: "vi" hoặc "en"
  vi               Bật gõ tiếng Việt (im lặng, dùng cho Vim/script)
  en               Tắt gõ tiếng Việt (im lặng, dùng cho Vim/script)
  on               Chọn và bật Gõ Nhanh
  off              Tắt Gõ Nhanh nếu đang được chọn
  toggle           Đảo trạng thái bật/tắt
  telex            Chuyển sang Telex và bật Gõ Nhanh
  vni              Chuyển sang VNI và bật Gõ Nhanh
  status           Xem trạng thái riêng của Gõ Nhanh
  update           Cài bản phát hành Linux mới nhất
  uninstall        Gỡ Gõ Nhanh, giữ nguyên input method khác
  version          Xem phiên bản
  help             Hiển thị trợ giúp
EOF
}

fail() { printf 'Lỗi: %s\n' "$*" >&2; exit 1; }

require_remote() {
    command -v fcitx5-remote >/dev/null 2>&1 ||
        fail "Không tìm thấy fcitx5-remote. Hãy cài Fcitx5 và đăng nhập lại."
}

FCITX_STATE=""

read_state() {
    require_remote
    FCITX_STATE="$(fcitx5-remote 2>/dev/null)" ||
        fail "Fcitx5 chưa chạy. Hãy khởi động Fcitx5 hoặc đăng nhập lại."
}

current_im() {
    fcitx5-remote -n 2>/dev/null || true
}

is_active() {
    [[ "$FCITX_STATE" == "2" && "$(current_im)" == "gonhanh" ]]
}

show_status() {
    read_state
    local method active_name
    method="$(sed -n '1p' "$METHOD_FILE" 2>/dev/null || true)"
    method="${method:-telex}"
    active_name="$(current_im)"
    if [[ "$FCITX_STATE" == "2" && "$active_name" == "gonhanh" ]]; then
        printf '● BẬT │ %s │ gonhanh\n' "$method"
    elif [[ -n "$active_name" ]]; then
        printf '○ TẮT │ %s │ input method hiện tại: %s\n' "$method" "$active_name"
    else
        printf '○ TẮT │ %s\n' "$method"
    fi
}

activate() {
    require_remote
    fcitx5-remote -s gonhanh >/dev/null 2>&1 ||
        fail "Fcitx5 chưa nhận addon gonhanh. Hãy đăng nhập lại hoặc chạy fcitx5 -r."
}

set_method() {
    local method="$1"
    mkdir -p "$(dirname "$METHOD_FILE")"
    printf '%s\n' "$method" > "$METHOD_FILE"
    require_remote
    fcitx5-remote -r >/dev/null 2>&1 || fail "Không thể tải lại Fcitx5"
    sleep 0.2
    activate
    show_status
}

if [[ $# -gt 1 ]]; then
    usage >&2
    fail "Lệnh không hợp lệ: $*"
fi

case "${1:-}" in
    "")
        read_state
        if is_active; then
            printf 'vi\n'
        else
            printf 'en\n'
        fi
        ;;
    vi)
        activate
        ;;
    en)
        read_state
        if is_active; then
            fcitx5-remote -c >/dev/null 2>&1 || fail "Không thể tắt Gõ Nhanh"
        fi
        ;;
    on)
        activate
        show_status
        ;;
    off)
        read_state
        if is_active; then
            fcitx5-remote -c >/dev/null 2>&1 || fail "Không thể tắt Gõ Nhanh"
        fi
        show_status
        ;;
    toggle)
        read_state
        if is_active; then
            fcitx5-remote -c >/dev/null 2>&1 || fail "Không thể tắt Gõ Nhanh"
        else
            activate
        fi
        show_status
        ;;
    telex|vni)
        set_method "$1"
        ;;
    status)
        show_status
        ;;
    version|-v|--version)
        printf 'Gõ Nhanh v%s\n' "$VERSION"
        ;;
    update)
        command -v curl >/dev/null 2>&1 || fail "Không tìm thấy curl"
        curl -fsSL https://raw.githubusercontent.com/khaphanspace/gonhanh.org/main/scripts/setup/linux.sh | bash
        ;;
    uninstall)
        [[ -x "$STATE_DIR/install.sh" ]] || fail "Không tìm thấy trình gỡ cài đặt"
        exec "$STATE_DIR/install.sh" --uninstall
        ;;
    help|-h|--help)
        usage
        ;;
    *)
        usage >&2
        fail "Lệnh không hợp lệ: $1"
        ;;
esac
