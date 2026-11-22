#include "autostart.h"

#include <windows.h>
#include <sddl.h>
#include <taskschd.h>
#include <wrl/client.h>

#include <mutex>
#include <string>
#include <thread>
#include <vector>

using Microsoft::WRL::ComPtr;

namespace gonhanh {

namespace {

constexpr wchar_t kRunKey[] = L"Software\\Microsoft\\Windows\\CurrentVersion\\Run";
constexpr wchar_t kRunValue[] = L"GoNhanh";

std::wstring ExePath() {
    wchar_t path[MAX_PATH];
    DWORD n = GetModuleFileNameW(nullptr, path, MAX_PATH);
    return (n > 0 && n < MAX_PATH) ? std::wstring(path, n) : std::wstring();
}

class Bstr {
public:
    explicit Bstr(const std::wstring& s) : b_(SysAllocStringLen(s.data(), static_cast<UINT>(s.size()))) {}
    ~Bstr() { SysFreeString(b_); }
    Bstr(const Bstr&) = delete;
    Bstr& operator=(const Bstr&) = delete;
    operator BSTR() const { return b_; }

private:
    BSTR b_;
};

// VARIANT holding a string (VT_BSTR), or VT_EMPTY for an empty string
class VariantStr {
public:
    explicit VariantStr(const std::wstring& s = {}) {
        VariantInit(&v_);
        if (!s.empty()) {
            v_.vt = VT_BSTR;
            v_.bstrVal = SysAllocStringLen(s.data(), static_cast<UINT>(s.size()));
        }
    }
    ~VariantStr() { VariantClear(&v_); }
    VariantStr(const VariantStr&) = delete;
    VariantStr& operator=(const VariantStr&) = delete;
    operator VARIANT() const { return v_; }

private:
    VARIANT v_;
};

// This user's SID ("S-1-5-21-...") and account name ("PC\name")
bool CurrentUser(std::wstring* sid, std::wstring* account) {
    HANDLE token = nullptr;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &token)) return false;
    DWORD size = 0;
    GetTokenInformation(token, TokenUser, nullptr, 0, &size);
    std::vector<BYTE> buf(size);
    bool ok = size > 0 && GetTokenInformation(token, TokenUser, buf.data(), size, &size);
    CloseHandle(token);
    if (!ok) return false;
    PSID psid = reinterpret_cast<TOKEN_USER*>(buf.data())->User.Sid;

    wchar_t* sidText = nullptr;
    if (!ConvertSidToStringSidW(psid, &sidText)) return false;
    *sid = sidText;
    LocalFree(sidText);

    wchar_t name[256], domain[256];
    DWORD nameLen = 256, domainLen = 256;
    SID_NAME_USE use;
    if (LookupAccountSidW(nullptr, psid, name, &nameLen, domain, &domainLen, &use)) {
        *account = std::wstring(domain) + L"\\" + name;
    } else {
        *account = *sid;
    }
    return true;
}

void SetRunEntry(bool on, const std::wstring& exe) {
    HKEY key;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, kRunKey, 0, KEY_QUERY_VALUE | KEY_SET_VALUE, &key) != ERROR_SUCCESS) {
        return;
    }
    if (on && !exe.empty()) {
        std::wstring quoted = L"\"" + exe + L"\"";
        wchar_t current[MAX_PATH + 3] = {};
        DWORD size = sizeof(current) - sizeof(wchar_t);  // keep a terminator
        DWORD type = 0;
        bool same = RegQueryValueExW(key, kRunValue, nullptr, &type, reinterpret_cast<LPBYTE>(current), &size) ==
                        ERROR_SUCCESS &&
                    type == REG_SZ && _wcsicmp(current, quoted.c_str()) == 0;
        if (!same) {
            RegSetValueExW(key, kRunValue, 0, REG_SZ, reinterpret_cast<const BYTE*>(quoted.c_str()),
                           static_cast<DWORD>((quoted.length() + 1) * sizeof(wchar_t)));
        }
    } else {
        RegDeleteValueW(key, kRunValue);
    }
    RegCloseKey(key);
}

// The registered task is enabled and runs `exe`
bool TaskIsCurrent(ITaskFolder* root, const std::wstring& name, const std::wstring& exe) {
    ComPtr<IRegisteredTask> task;
    if (FAILED(root->GetTask(Bstr(name), &task))) return false;
    VARIANT_BOOL enabled = VARIANT_FALSE;
    if (FAILED(task->get_Enabled(&enabled)) || !enabled) return false;
    ComPtr<ITaskDefinition> def;
    ComPtr<IActionCollection> actions;
    ComPtr<IAction> action;
    ComPtr<IExecAction> exec;
    if (FAILED(task->get_Definition(&def)) || FAILED(def->get_Actions(&actions)) ||
        FAILED(actions->get_Item(1, &action)) || FAILED(action.As(&exec))) {
        return false;
    }
    BSTR path = nullptr;
    if (FAILED(exec->get_Path(&path)) || !path) return false;
    bool same = _wcsicmp(path, exe.c_str()) == 0;
    SysFreeString(path);
    return same;
}

bool RegisterTask(ITaskService* service, ITaskFolder* root, const std::wstring& name, const std::wstring& exe,
                  const std::wstring& account) {
    ComPtr<ITaskDefinition> def;
    if (FAILED(service->NewTask(0, &def))) return false;

    ComPtr<IRegistrationInfo> info;
    if (SUCCEEDED(def->get_RegistrationInfo(&info))) {
        info->put_Author(Bstr(L"Gõ Nhanh"));
        info->put_Description(Bstr(L"Khởi động Gõ Nhanh khi đăng nhập (bật/tắt trong Cài đặt của Gõ Nhanh)."));
    }

    // Runs as this user, in their session, without elevation
    ComPtr<IPrincipal> principal;
    if (FAILED(def->get_Principal(&principal))) return false;
    principal->put_UserId(Bstr(account));
    principal->put_LogonType(TASK_LOGON_INTERACTIVE_TOKEN);
    principal->put_RunLevel(TASK_RUNLEVEL_LUA);

    // The defaults suit maintenance jobs, not an app that runs all day: they would skip the
    // start on battery, kill it after 72 hours and run it at low priority (7, which also lowers
    // its I/O and memory priority). 4 is normal; the app raises its own CPU priority.
    ComPtr<ITaskSettings> settings;
    if (FAILED(def->get_Settings(&settings))) return false;
    settings->put_DisallowStartIfOnBatteries(VARIANT_FALSE);
    settings->put_StopIfGoingOnBatteries(VARIANT_FALSE);
    settings->put_ExecutionTimeLimit(Bstr(L"PT0S"));
    settings->put_Priority(4);
    settings->put_MultipleInstances(TASK_INSTANCES_IGNORE_NEW);
    settings->put_StartWhenAvailable(VARIANT_FALSE);
    settings->put_AllowDemandStart(VARIANT_TRUE);
    settings->put_Enabled(VARIANT_TRUE);
    ComPtr<IIdleSettings> idle;
    if (SUCCEEDED(settings->get_IdleSettings(&idle))) idle->put_StopOnIdleEnd(VARIANT_FALSE);

    // When this user signs in, with no delay
    ComPtr<ITriggerCollection> triggers;
    ComPtr<ITrigger> trigger;
    ComPtr<ILogonTrigger> logon;
    if (FAILED(def->get_Triggers(&triggers)) || FAILED(triggers->Create(TASK_TRIGGER_LOGON, &trigger)) ||
        FAILED(trigger.As(&logon))) {
        return false;
    }
    logon->put_UserId(Bstr(account));

    ComPtr<IActionCollection> actions;
    ComPtr<IAction> action;
    ComPtr<IExecAction> exec;
    if (FAILED(def->get_Actions(&actions)) || FAILED(actions->Create(TASK_ACTION_EXEC, &action)) ||
        FAILED(action.As(&exec))) {
        return false;
    }
    exec->put_Path(Bstr(exe));
    size_t slash = exe.find_last_of(L'\\');
    if (slash != std::wstring::npos) exec->put_WorkingDirectory(Bstr(exe.substr(0, slash)));

    ComPtr<IRegisteredTask> registered;
    return SUCCEEDED(root->RegisterTaskDefinition(Bstr(name), def.Get(), TASK_CREATE_OR_UPDATE,
                                                  VariantStr(account), VariantStr(),
                                                  TASK_LOGON_INTERACTIVE_TOKEN, VariantStr(), &registered));
}

// Applies `on` to the logon task. Returns true when the task now starts this exe at sign-in.
bool ApplyTask(bool on, const std::wstring& exe) {
    std::wstring sid, account;
    if (!CurrentUser(&sid, &account)) return false;
    // Task names are machine-wide: one task per user
    const std::wstring name = L"GoNhanh autostart " + sid;

    HRESULT init = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
    bool ok = false;
    {
        ComPtr<ITaskService> service;
        ComPtr<ITaskFolder> root;
        if (SUCCEEDED(CoCreateInstance(CLSID_TaskScheduler, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&service))) &&
            SUCCEEDED(service->Connect(VariantStr(), VariantStr(), VariantStr(), VariantStr())) &&
            SUCCEEDED(service->GetFolder(Bstr(L"\\"), &root))) {
            if (on) {
                ok = TaskIsCurrent(root.Get(), name, exe) || RegisterTask(service.Get(), root.Get(), name, exe, account);
            } else {
                root->DeleteTask(Bstr(name), 0);
            }
        }
    }
    if (SUCCEEDED(init)) CoUninitialize();
    return ok;
}

void Apply(bool on) {
    std::wstring exe = ExePath();
    bool task = !exe.empty() && ApplyTask(on, exe);
    // Run entry only when the task couldn't be registered (also removes a Run entry left by
    // earlier versions once the task exists)
    SetRunEntry(on && !task, exe);
}

}  // namespace

// Task Scheduler calls can take seconds (3 s seen deleting a task). The caller is the UI thread,
// which also runs the keyboard hook: blocking it stalls typing and can get the hook removed by
// Windows. So a worker applies the latest requested state.
void SetAutoStart(bool on) {
    static std::mutex lock;
    static int requested = -1;  // -1 before the first call
    static int applied = -1;
    static bool working = false;
    {
        std::lock_guard<std::mutex> guard(lock);
        if (requested == static_cast<int>(on)) return;
        requested = on;
        if (working) return;  // the running worker picks up the new state
        working = true;
    }
    std::thread([] {
        for (;;) {
            int want;
            {
                std::lock_guard<std::mutex> guard(lock);
                want = requested;
                if (want == applied) {
                    working = false;
                    return;
                }
            }
            Apply(want != 0);
            std::lock_guard<std::mutex> guard(lock);
            applied = want;
        }
    }).detach();
}

}  // namespace gonhanh
