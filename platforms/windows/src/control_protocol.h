#pragma once

// Control protocol between GoNhanh.exe and the `gonhanh` / `gn` command line tool (issue #433).
//
// The CLI finds the app's hidden message-only window and sends it one message with
// SendMessageTimeout: wParam = command, lParam = protocol version. The app applies the command
// on its UI thread (same path as the tray and Ctrl+Space) and returns its state in the
// LRESULT, so a call is one synchronous round trip (well under a millisecond).

#include <windows.h>

namespace gonhanh::control {

// Class of the app's hidden message-only window (also used for single-instance activation)
inline constexpr wchar_t kWindowClass[] = L"GoNhanhHidden";

inline constexpr UINT kMessage = WM_APP + 60;
inline constexpr LPARAM kProtocolVersion = 1;

enum class Command : WPARAM {
    Get = 1,     // read state only
    Vi = 2,      // Vietnamese on
    En = 3,      // Vietnamese off (English)
    Toggle = 4,
    Telex = 5,   // switch to Telex and turn Vietnamese on
    Vni = 6,     // switch to VNI and turn Vietnamese on
    Quit = 7,    // exit the app (replies first, then leaves its message loop)
};

// LRESULT bits. kOk is always set when the app handled the request, so 0 means
// "not handled" (wrong protocol version, unknown command).
inline constexpr LRESULT kOk = 0x100;
inline constexpr LRESULT kEnabled = 0x1;
inline constexpr LRESULT kMethodVni = 0x2;

}  // namespace gonhanh::control
