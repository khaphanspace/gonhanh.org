// `gonhanh` / `gn`: control the running Gõ Nhanh from the command line (issue #433).
//
// Same commands as the macOS and Linux CLI, following the macism / im-select convention so
// Vim plugins (im-select.nvim, VSCodeVim, Obsidian Vim Motions...) can switch on mode changes:
//
//   gonhanh          print "vi" or "en"
//   gonhanh vi|en    switch, prints nothing
//   gonhanh on|off|toggle|telex|vni|status   switch and print a status line
//
// Windows extras: `start` / `quit` open or close the app, `completion` prints a PowerShell
// Tab-completion script.
//
// Exit code 0 on success; 1 with a message on stderr if the command is invalid or Gõ Nhanh
// is not running (except `en`: without Gõ Nhanh typing is already English). Talks to the app
// with one SendMessageTimeout (control_protocol.h).

#include "control_protocol.h"

#include <cstdio>
#include <string>
#include <string_view>

#ifndef APP_VERSION_STRING
#define APP_VERSION_STRING "0.0.0"
#endif

namespace {

using gonhanh::control::Command;

// Commands as shown by `help` and offered by Tab completion
struct HelpEntry {
    const wchar_t* name;
    const wchar_t* description;
};
constexpr HelpEntry kHelp[] = {
    {L"vi", L"Bật gõ tiếng Việt (im lặng, dùng cho Vim/script)"},
    {L"en", L"Tắt gõ tiếng Việt (im lặng, dùng cho Vim/script)"},
    {L"on", L"Bật Gõ Nhanh và hiện trạng thái"},
    {L"off", L"Tắt Gõ Nhanh và hiện trạng thái"},
    {L"toggle", L"Đảo trạng thái bật/tắt"},
    {L"telex", L"Chuyển sang Telex và bật Gõ Nhanh"},
    {L"vni", L"Chuyển sang VNI và bật Gõ Nhanh"},
    {L"status", L"Xem trạng thái riêng của Gõ Nhanh"},
    {L"start", L"Mở Gõ Nhanh nếu chưa chạy"},
    {L"quit", L"Thoát Gõ Nhanh"},
    {L"completion", L"In script gợi ý lệnh (Tab) cho PowerShell"},
    {L"version", L"Xem phiên bản"},
    {L"help", L"Hiển thị trợ giúp"},
};

std::wstring Usage() {
    std::wstring s =
        L"Gõ Nhanh - Vietnamese Input Method\n"
        L"\n"
        L"Cách dùng: gonhanh [lệnh] (hoặc: gn [lệnh])\n"
        L"\n"
        L"  (không tham số)  In trạng thái hiện tại: \"vi\" hoặc \"en\"\n";
    for (const HelpEntry& e : kHelp) {
        std::wstring name = e.name;
        s += L"  " + name + std::wstring(name.size() < 17 ? 17 - name.size() : 1, L' ') + e.description + L"\n";
    }
    return s;
}

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

HWND FindApp() {
    return FindWindowExW(HWND_MESSAGE, nullptr, gonhanh::control::kWindowClass, nullptr);
}

// Send one command to the running app. Returns its state bits, or 0 with *error set
// (and *notRunning when there is no Gõ Nhanh to talk to).
LRESULT Send(Command command, std::wstring* error, bool* notRunning) {
    *notRunning = false;
    HWND app = FindApp();
    if (!app) {
        *notRunning = true;
        *error = L"Gõ Nhanh chưa chạy. Mở bằng: gn start";
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
        *error = L"Gõ Nhanh đang chạy là bản cũ, chưa hỗ trợ lệnh này. Hãy cập nhật Gõ Nhanh.";
        return 0;
    }
    return static_cast<LRESULT>(result);
}

// GoNhanh.exe in the folder above this CLI's bin\ folder
std::wstring AppExePath() {
    wchar_t self[MAX_PATH];
    DWORD n = GetModuleFileNameW(nullptr, self, MAX_PATH);
    if (n == 0 || n >= MAX_PATH) return {};
    std::wstring path(self, n);
    for (int i = 0; i < 2; ++i) {  // drop "gn.exe", then "bin"
        size_t slash = path.find_last_of(L'\\');
        if (slash == std::wstring::npos) return {};
        path.resize(slash);
    }
    return path + L"\\GoNhanh.exe";
}

int Start() {
    if (FindApp()) {
        Out(L"Gõ Nhanh đang chạy.\n");
        return 0;
    }
    std::wstring exe = AppExePath();
    if (exe.empty() || GetFileAttributesW(exe.c_str()) == INVALID_FILE_ATTRIBUTES) {
        return Fail(L"Không tìm thấy " + (exe.empty() ? std::wstring(L"GoNhanh.exe") : exe));
    }
    std::wstring dir = exe.substr(0, exe.find_last_of(L'\\'));
    std::wstring cmdLine = L"\"" + exe + L"\"";
    STARTUPINFOW si = {};
    si.cb = sizeof(si);
    PROCESS_INFORMATION pi = {};
    // No inherited handles: a caller reading our output through a pipe (Vim, scripts) must not
    // wait for the app, which keeps running.
    if (!CreateProcessW(exe.c_str(), cmdLine.data(), nullptr, nullptr, FALSE, 0, nullptr, dir.c_str(), &si,
                        &pi)) {
        return Fail(L"Không mở được Gõ Nhanh (mã lỗi " + std::to_wstring(GetLastError()) + L").");
    }
    CloseHandle(pi.hThread);

    // Return once it answers, so `gn start && gn vi` works. The wait is bounded by wall-clock
    // time: a window that exists but doesn't answer makes each Send() take its 1 s timeout.
    const ULONGLONG deadline = GetTickCount64() + 10000;
    int result = -1;
    while (result < 0) {
        std::wstring error;
        bool notRunning = false;
        if (Send(Command::Get, &error, &notRunning)) {
            result = 0;
        } else if (WaitForSingleObject(pi.hProcess, 50) == WAIT_OBJECT_0) {
            DWORD code = 0;
            GetExitCodeProcess(pi.hProcess, &code);
            // Exit code 0 right away: another instance won the single-instance race
            result = code == 0 && FindApp() ? 0 : Fail(L"Gõ Nhanh thoát ngay sau khi mở.");
        } else if (GetTickCount64() >= deadline) {
            result = Fail(L"Gõ Nhanh không phản hồi sau 10 giây.");
        }
    }
    CloseHandle(pi.hProcess);
    if (result == 0) Out(L"Đã mở Gõ Nhanh.\n");
    return result;
}

int Quit() {
    HWND app = FindApp();
    if (!app) {
        Out(L"Gõ Nhanh chưa chạy.\n");
        return 0;
    }
    DWORD pid = 0;
    GetWindowThreadProcessId(app, &pid);
    HANDLE process = OpenProcess(SYNCHRONIZE, FALSE, pid);  // may fail for an elevated app

    std::wstring error;
    bool notRunning = false;
    if (!Send(Command::Quit, &error, &notRunning) && !notRunning) {
        if (process) CloseHandle(process);
        return Fail(error);
    }

    bool exited = false;
    if (process) {
        exited = WaitForSingleObject(process, 5000) == WAIT_OBJECT_0;
        CloseHandle(process);
    } else {
        for (int i = 0; i < 100 && !exited; ++i) {
            exited = !FindApp();
            if (!exited) Sleep(50);
        }
    }
    if (!exited) return Fail(L"Gõ Nhanh chưa thoát sau 5 giây.");
    Out(L"Đã thoát Gõ Nhanh.\n");
    return 0;
}

// PowerShell string literal for `text`, ASCII only so it survives any console code page when the
// script is piped into Invoke-Expression: other characters become \uXXXX for [regex]::Unescape.
std::wstring PsLiteral(const wchar_t* text) {
    std::wstring escaped;
    for (const wchar_t* p = text; *p; ++p) {
        wchar_t c = *p;
        if (c == L'\'') {
            escaped += L"''";
        } else if (c == L'\\') {
            escaped += L"\\\\";
        } else if (c < 0x20 || c > 0x7E) {
            wchar_t buf[8];
            swprintf_s(buf, L"\\u%04x", static_cast<unsigned>(c));
            escaped += buf;
        } else {
            escaped += c;
        }
    }
    return L"[regex]::Unescape('" + escaped + L"')";
}

int Completion() {
    std::wstring s =
        L"# Go Nhanh: Tab completion for gn / gonhanh in PowerShell (ASCII only on purpose).\n"
        L"# Add this line to your $PROFILE (notepad $PROFILE):\n"
        L"#   gn completion | Out-String | Invoke-Expression\n"
        L"Register-ArgumentCompleter -Native -CommandName 'gn', 'gonhanh', 'gn.exe', 'gonhanh.exe' -ScriptBlock {\n"
        L"    param($wordToComplete, $commandAst, $cursorPosition)\n"
        L"    $count = $commandAst.CommandElements.Count\n"
        L"    # Only the first argument: gn <Tab>\n"
        L"    if ($count -gt 2 -or ($count -eq 2 -and $wordToComplete -eq '')) { return }\n"
        L"    @(\n";
    for (const HelpEntry& e : kHelp) {
        s += std::wstring(L"        ,@('") + e.name + L"', " + PsLiteral(e.description) + L")\n";
    }
    s +=
        L"    ) | Where-Object { $_[0] -like \"$wordToComplete*\" } | ForEach-Object {\n"
        L"        [System.Management.Automation.CompletionResult]::new($_[0], $_[0], 'ParameterValue', $_[1])\n"
        L"    }\n"
        L"}\n";
    Out(s);
    return 0;
}

}  // namespace

int wmain(int argc, wchar_t** argv) {
    if (argc > 2) {
        std::wstring all;
        for (int i = 1; i < argc; ++i) all += (i > 1 ? L" " : L"") + std::wstring(argv[i]);
        Write(STD_ERROR_HANDLE, Usage());
        return Fail(L"Lệnh không hợp lệ: " + all);
    }

    std::wstring_view arg = argc == 2 ? argv[1] : L"";
    if (arg == L"help" || arg == L"-h" || arg == L"--help" || arg == L"/?") {
        Out(Usage());
        return 0;
    }
    if (arg == L"version" || arg == L"-v" || arg == L"--version") {
        Out(Version());
        return 0;
    }
    if (arg == L"start") return Start();
    if (arg == L"quit") return Quit();
    if (arg == L"completion") return Completion();

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

    Write(STD_ERROR_HANDLE, Usage());
    return Fail(L"Lệnh không hợp lệ: " + std::wstring(arg));
}
