//! One Task Scheduler task per user covers two settings:
//! - "Khởi động cùng Windows": the task has a logon trigger.
//! - "Chạy với quyền admin": the task runs with the user's highest privileges, so GoNhanh can type into
//!   elevated apps (Task Manager, admin terminals). Starting GoNhanh by hand then goes through the task,
//!   which gives admin rights with no UAC prompt.
//!
//! Creating an admin task (or changing one) needs admin rights once, so it asks UAC when GoNhanh is not elevated.

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_CANCELLED, HANDLE, HWND};
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, INFINITE};
use windows_sys::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::services::{log, registry as reg};
use crate::win::{is_elevated, wide};

/// Where older versions registered autostart; removed once the task takes over
const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "GoNhanh";

/// Passed by the task, so an instance it started never hands off to the task again
pub const FROM_TASK_ARG: &str = "--from-task";

pub enum Outcome {
    Done,
    /// The user said no to UAC
    Cancelled,
    Failed,
}

/// Make the task match the two settings (create, change or delete it). `owner` parents the UAC prompt.
pub fn sync(auto_start: bool, admin: bool, owner: HWND) -> Outcome {
    let outcome = if auto_start || admin {
        create(auto_start, admin, owner)
    } else {
        delete(owner)
    };
    if matches!(outcome, Outcome::Done) {
        reg::delete_value(RUN_KEY, RUN_VALUE);
    }
    outcome
}

/// Start GoNhanh through the task (with admin rights when the task has them, no UAC)
pub fn start_via_task() -> bool {
    let args = format!(r#"/Run /TN "{}""#, task_name());
    matches!(schtasks(&args, false, null_mut()), Outcome::Done)
}

/// One task per Windows user, so two accounts on one PC don't overwrite each other
fn task_name() -> String {
    format!("GoNhanh_{}", std::env::var("USERNAME").unwrap_or_default())
}

fn create(auto_start: bool, admin: bool, owner: HWND) -> Outcome {
    let Ok(exe) = std::env::current_exe() else {
        return Outcome::Failed;
    };
    let user = format!(
        r"{}\{}",
        std::env::var("USERDOMAIN").unwrap_or_default(),
        std::env::var("USERNAME").unwrap_or_default()
    );
    let xml = task_xml(&exe.display().to_string(), &user, auto_start, admin);
    // schtasks reads UTF-16 task XML (with BOM) reliably
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
    let file = std::env::temp_dir().join("GoNhanh-task.xml");
    if std::fs::write(&file, bytes).is_err() {
        log::write("autostart: cannot write task XML");
        return Outcome::Failed;
    }
    let args = format!(
        r#"/Create /TN "{}" /XML "{}" /F"#,
        task_name(),
        file.display()
    );
    let outcome = run_with_rights(&args, admin, owner);
    let _ = std::fs::remove_file(&file);
    outcome
}

fn delete(owner: HWND) -> Outcome {
    let name = task_name();
    // The task is readable by its own user, so this needs no UAC (and skips it when only the old Run entry exists)
    if !matches!(
        schtasks(&format!(r#"/Query /TN "{name}""#), false, owner),
        Outcome::Done
    ) {
        return Outcome::Done;
    }
    run_with_rights(&format!(r#"/Delete /TN "{name}" /F"#), false, owner)
}

/// Run schtasks with admin rights when `needs_admin`; otherwise try without first and ask UAC only if
/// that is refused (a task created as admin can only be changed as admin)
fn run_with_rights(args: &str, needs_admin: bool, owner: HWND) -> Outcome {
    if is_elevated() {
        return schtasks(args, false, owner);
    }
    if !needs_admin {
        if let Outcome::Done = schtasks(args, false, owner) {
            return Outcome::Done;
        }
    }
    schtasks(args, true, owner)
}

/// Normal priority (the default 7 is below normal, too low for a keyboard hook), no battery or
/// run-time limits. Parallel: our own mutex keeps one GoNhanh, and a restart must not be ignored
/// while the old instance is still closing.
fn task_xml(exe: &str, user: &str, auto_start: bool, admin: bool) -> String {
    let (exe, user) = (xml_escape(exe), xml_escape(user));
    let triggers = if auto_start {
        format!(
            r#"
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>"#
        )
    } else {
        String::new()
    };
    let run_level = if admin {
        "HighestAvailable"
    } else {
        "LeastPrivilege"
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Gõ Nhanh: khởi động cùng Windows, chạy với quyền admin</Description>
  </RegistrationInfo>{triggers}
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>{run_level}</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>Parallel</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
      <Arguments>{FROM_TASK_ARG}</Arguments>
    </Exec>
  </Actions>
</Task>
"#
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Run schtasks.exe hidden (through UAC when `elevate`) and wait for it
fn schtasks(args: &str, elevate: bool, owner: HWND) -> Outcome {
    let system = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    let (verb, file, params) = (
        wide(if elevate { "runas" } else { "open" }),
        wide(&format!("{system}\\System32\\schtasks.exe")),
        wide(args),
    );
    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI;
    info.hwnd = owner;
    info.lpVerb = verb.as_ptr();
    info.lpFile = file.as_ptr();
    info.lpParameters = params.as_ptr();
    info.nShow = SW_HIDE;
    if unsafe { ShellExecuteExW(&mut info) } == 0 {
        let err = unsafe { GetLastError() };
        if err == ERROR_CANCELLED {
            return Outcome::Cancelled;
        }
        log::write(&format!("autostart: cannot start schtasks ({err})"));
        return Outcome::Failed;
    }
    if info.hProcess.is_null() {
        return Outcome::Failed;
    }
    wait_pumping(info.hProcess);
    let mut code = 1;
    unsafe {
        GetExitCodeProcess(info.hProcess, &mut code);
        CloseHandle(info.hProcess);
    }
    if code == 0 {
        Outcome::Done
    } else {
        if log::enabled() {
            log::write(&format!("autostart: schtasks {args} → exit {code}"));
        }
        Outcome::Failed
    }
}

/// Wait while still dispatching messages: the keyboard hook runs on this thread, and Windows drops
/// a low-level hook that stops answering
fn wait_pumping(process: HANDLE) {
    let mut quit = None;
    unsafe {
        // 1 = a message arrived; 0 = the process ended; anything else = give up waiting
        while MsgWaitForMultipleObjects(1, &process, 0, INFINITE, QS_ALLINPUT) == 1 {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    quit = Some(msg.wParam as i32);
                    continue;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if let Some(code) = quit {
            PostQuitMessage(code); // leave it for the main loop
        }
    }
}
