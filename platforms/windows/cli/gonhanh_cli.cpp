// `gonhanh` / `gn`: control the running Gõ Nhanh from the command line (issue #433).
//
// Same commands as the macOS and Linux CLI, following the macism / im-select convention so
// Vim plugins (im-select.nvim, VSCodeVim, Obsidian Vim Motions...) can switch on mode changes:
//
//   gonhanh          print "vi" or "en"
//   gonhanh vi|en    switch, prints nothing
//   gonhanh on|off|toggle|telex|vni|status   switch and print a status line
//
// Exit code 0 on success; 1 with a message on stderr if the command is invalid or Gõ Nhanh
// is not running (except `en`: without Gõ Nhanh typing is already English). Talks to the app
// with one SendMessageTimeout (control_protocol.h).

#include "control_protocol.h"

#include <string>
#include <string_view>

#ifndef APP_VERSION_STRING
#define APP_VERSION_STRING "0.0.0"
#endif

namespace {

using gonhanh::control::Command;

constexpr wchar_t kUsage[] =
    L"Gõ Nhanh - Vietnamese Input Method\n"
    L"\n"
    L"Cách dùng: gonhanh [lệnh] (hoặc: gn [lệnh])\n"
    L"\n"
    L"  (không tham số)  In trạng thái hiện tại: \"vi\" hoặc \"en\"\n"
    L"  vi               Bật gõ tiếng Việt (im lặng, dùng cho Vim/script)\n"
    L"  en               Tắt gõ tiếng Việt (im lặng, dùng cho Vim/script)\n"
    L"  on               Bật Gõ Nhanh và hiện trạng thái\n"
    L"  off              Tắt Gõ Nhanh và hiện trạng thái\n"
    L"  toggle           Đảo trạng thái bật/tắt\n"
    L"  telex            Chuyển sang Telex và bật Gõ Nhanh\n"
    L"  vni              Chuyển sang VNI và bật Gõ Nhanh\n"
    L"  status           Xem trạng thái riêng của Gõ Nhanh\n"
    L"  version          Xem phiên bản\n"
    L"  help             Hiển thị trợ giúp\n";

// Console: WriteConsoleW (correct Unicode regardless of the code page).
// Pipe or file (Vim plugins, scripts): UTF-8 bytes.
void Write(DWORD stdHandle, std::wstring_view text) {
    HANDLE h = GetStdHandle(stdHandle);
    if (!h || h == INVALID_HANDLE_VALUE) return;
    DWORD mode = 0, written = 0;
    if (GetConsoleMode(h, &mode)) {
        WriteConsoleW(h, text.data(), static_cast<DWORD>(text.size()), &written, nullptr);
        return;
    }
    int len = WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), nullptr, 0,
                                  nullptr, nullptr);
    if (len <= 0) return;
    std::string utf8(static_cast<size_t>(len), '\0');
    WideCharToMultiByte(CP_UTF8, 0, text.data(), static_cast<int>(text.size()), utf8.data(), len, nullptr,
                        nullptr);
    WriteFile(h, utf8.data(), static_cast<DWORD>(utf8.size()), &written, nullptr);
}

void Out(std::wstring_view text) { Write(STD_OUTPUT_HANDLE, text); }

int Fail(std::wstring_view message) {
    Write(STD_ERROR_HANDLE, std::wstring(L"Lỗi: ") + std::wstring(message) + L"\n");
    return 1;
}

std::wstring StatusLine(LRESULT state) {
    const wchar_t* method = (state & gonhanh::control::kMethodVni) ? L"vni" : L"telex";
    if (state & gonhanh::control::kEnabled) return std::wstring(L"● BẬT │ ") + method + L" │ gonhanh\n";
    return std::wstring(L"○ TẮT │ ") + method + L"\n";
}

std::wstring Version() {
    std::wstring v;
    for (const char* p = APP_VERSION_STRING; *p; ++p) v += static_cast<wchar_t>(*p);
    return L"Gõ Nhanh v" + v + L"\n";
}

// Send one command to the running app. Returns its state bits, or 0 with *error set
// (and *notRunning when there is no Gõ Nhanh to talk to).
LRESULT Send(Command command, std::wstring* error, bool* notRunning) {
    *notRunning = false;
    HWND app = FindWindowExW(HWND_MESSAGE, nullptr, gonhanh::control::kWindowClass, nullptr);
    if (!app) {
        *notRunning = true;
        *error = L"Gõ Nhanh chưa chạy.";
        return 0;
    }
    DWORD_PTR result = 0;
    if (!SendMessageTimeoutW(app, gonhanh::control::kMessage, static_cast<WPARAM>(command),
                             gonhanh::control::kProtocolVersion, SMTO_ABORTIFHUNG | SMTO_BLOCK, 1000,
                             &result)) {
        DWORD code = GetLastError();
        *error = code == ERROR_ACCESS_DENIED
                     ? L"Gõ Nhanh đang chạy với quyền Administrator và chưa cho phép dòng lệnh. Hãy cập nhật "
                       L"Gõ Nhanh hoặc chạy lệnh này với quyền Administrator."
                     : L"Không thể giao tiếp với Gõ Nhanh (mã lỗi " + std::to_wstring(code) + L").";
        return 0;
    }
    if (!(static_cast<LRESULT>(result) & gonhanh::control::kOk)) {
        *error = L"Gõ Nhanh đang chạy là bản cũ, chưa hỗ trợ dòng lệnh. Hãy cập nhật Gõ Nhanh.";
        return 0;
    }
    return static_cast<LRESULT>(result);
}

}  // namespace

int wmain(int argc, wchar_t** argv) {
    if (argc > 2) {
        std::wstring all;
        for (int i = 1; i < argc; ++i) all += (i > 1 ? L" " : L"") + std::wstring(argv[i]);
        Write(STD_ERROR_HANDLE, kUsage);
        return Fail(L"Lệnh không hợp lệ: " + all);
    }

    std::wstring_view arg = argc == 2 ? argv[1] : L"";
    if (arg == L"help" || arg == L"-h" || arg == L"--help" || arg == L"/?") {
        Out(kUsage);
        return 0;
    }
    if (arg == L"version" || arg == L"-v" || arg == L"--version") {
        Out(Version());
        return 0;
    }

    struct Entry {
        std::wstring_view name;
        Command command;
        bool printState;   // "vi" / "en"
        bool printStatus;  // status line
    };
    static constexpr Entry kCommands[] = {
        {L"", Command::Get, true, false},
        {L"vi", Command::Vi, false, false},
        {L"en", Command::En, false, false},
        {L"on", Command::Vi, false, true},
        {L"off", Command::En, false, true},
        {L"toggle", Command::Toggle, false, true},
        {L"telex", Command::Telex, false, true},
        {L"vni", Command::Vni, false, true},
        {L"status", Command::Get, false, true},
    };

    for (const Entry& e : kCommands) {
        if (arg != e.name) continue;
        std::wstring error;
        bool notRunning = false;
        LRESULT state = Send(e.command, &error, &notRunning);
        // Without Gõ Nhanh every key is already English: `en` (sent by Vim on each Esc) has
        // nothing to do, so it succeeds instead of flooding the editor with errors.
        if (!state && notRunning && e.command == Command::En && !e.printStatus) return 0;
        if (!state) return Fail(error);
        if (e.printState) Out((state & gonhanh::control::kEnabled) ? L"vi\n" : L"en\n");
        if (e.printStatus) Out(StatusLine(state));
        return 0;
    }

    Write(STD_ERROR_HANDLE, kUsage);
    return Fail(L"Lệnh không hợp lệ: " + std::wstring(arg));
}
