#pragma once

namespace gonhanh {

// Start Gõ Nhanh when this user signs in.
//
// On: a Task Scheduler logon task for this user that runs this exe. Explorer starts the
// HKCU\...\Run entries one after another, each waiting for the previous one, well after sign-in
// (about 80 s with a handful of apps), so an input method registered there comes up last. A
// logon task starts at sign-in, in parallel. If the task can't be registered, falls back to Run.
// Off: removes the task and the Run entry.
//
// Returns at once: a background thread does the work. The task is only rewritten when it is
// missing, disabled or runs another exe (GoNhanh moved), and a state already requested in this
// process is not applied again.
void SetAutoStart(bool on);

}  // namespace gonhanh
