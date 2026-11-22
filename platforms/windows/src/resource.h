#pragma once

// App version: set by CMake from the release tag (GONHANH_VERSION); fallback for other builds
#ifndef APP_VERSION_STRING
#define APP_VERSION_STRING "0.0.0"
#endif
#ifndef APP_VERSION_MAJOR
#define APP_VERSION_MAJOR 0
#define APP_VERSION_MINOR 0
#define APP_VERSION_PATCH 0
#define APP_VERSION_BUILD 0
#endif

// Icons (IDI_APPICON=1 required for exe icon)
#define IDI_APPICON         1
#define IDI_TRAY_ICON       101
#define IDI_APP_LOGO        102

// PNG Resources
#define IDR_LOGO_PNG        103

// Menus
#define IDM_ENABLE          201
#define IDM_TELEX           202
#define IDM_VNI             203
#define IDM_SETTINGS        204
#define IDM_EXIT            206

// Settings window controls
#define IDC_CHK_ENABLED     301
#define IDC_CMB_METHOD      302
#define IDC_CHK_W_SHORTCUT  303
#define IDC_CHK_BRACKET     304
#define IDC_CHK_ESC_RESTORE 305
#define IDC_CHK_AUTOSTART   306
#define IDC_CHK_PERAPP      307
#define IDC_CHK_AUTORESTORE 308
#define IDC_CHK_SOUND       309
#define IDC_CHK_MODERN      310
#define IDC_CHK_CAPITALIZE  311
#define IDC_BTN_SHORTCUTS   312
#define IDC_HOTKEY          313
#define IDC_BTN_APPLY       314
#define IDC_CHK_FOREIGN     315


// Dialog IDs
#define IDD_SETTINGS        501

// Messages
#define WM_TRAYICON         (WM_USER + 1)
#define WM_TOGGLE_IME       (WM_USER + 2)   // Posted by the keyboard hook on Ctrl+Space
