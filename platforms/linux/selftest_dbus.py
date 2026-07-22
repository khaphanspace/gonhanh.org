#!/usr/bin/env python3
"""
End-to-end self-test for the gonhanh Fcitx5 addon.

Drives the ACTUAL running fcitx5 daemon + gonhanh addon over DBus — no human
GUI typing. It creates a real input context, activates the `gonhanh` input
method, injects a telex key sequence via ProcessKeyEvent, and reconstructs the
resulting on-screen text exactly like a real editor would:

  * a key fcitx does NOT handle (ProcessKeyEvent -> False) is a normal
    keystroke: the client inserts that character itself;
  * CommitString appends committed text;
  * DeleteSurroundingText removes characters (telex corrects a already-typed
    letter, e.g. "dd" -> delete the 'd', commit "đ");
  * ForwardKey re-injects a key fcitx generated.

gonhanh commits directly (no preedit), so the visible text is fully determined
by those signals plus the passthrough keys.

Run it against a fcitx5 where `gonhanh` is the active input method. The
companion `selftest.sh` spins up a private, isolated fcitx5 for this so it never
touches the user's live input session. Exit 0 iff every telex case converts to
the expected Vietnamese, else non-zero with a per-case diff.
"""
import sys
import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

SERVICE = "org.fcitx.Fcitx5"
IM1_PATH = "/org/freedesktop/portal/inputmethod"
IM1_IFACE = "org.fcitx.Fcitx.InputMethod1"
IC1_IFACE = "org.fcitx.Fcitx.InputContext1"
CTRL_PATH = "/controller"
CTRL_IFACE = "org.fcitx.Fcitx.Controller1"

# key -> (X keysym, X keycode = evdev + 8)
K = {
    'd': (0x64, 40), 'a': (0x61, 38), 's': (0x73, 39), 'n': (0x6e, 57),
    'g': (0x67, 42), 'u': (0x75, 30), 'y': (0x79, 29), 'v': (0x76, 55),
    'i': (0x69, 31), 'e': (0x65, 26), 't': (0x74, 28), 'o': (0x6f, 32),
    'w': (0x77, 25), 'r': (0x72, 27), 'j': (0x6a, 44), ' ': (0x20, 65),
}

# (telex input, expected Vietnamese output)
CASES = [
    ("dd",     "đ"),
    ("aa",     "â"),
    ("as",     "á"),
    ("oo",     "ô"),
    ("ow",     "ơ"),
    ("ddaay",  "đây"),
    ("ddungs", "đúng"),
    ("vieejt", "việt"),
    ("tieengs", "tiếng"),
]


def main():
    DBusGMainLoop(set_as_default=True)
    bus = dbus.SessionBus()

    ctx = GLib.MainContext.default()

    def pump(ms):
        deadline = GLib.get_monotonic_time() + ms * 1000
        while GLib.get_monotonic_time() < deadline:
            while ctx.pending():
                ctx.iteration(False)
            GLib.usleep(1500)

    ctrl = dbus.Interface(bus.get_object(SERVICE, CTRL_PATH), CTRL_IFACE)
    avail = [str(x[0]) for x in ctrl.AvailableInputMethods()]
    if "gonhanh" not in avail:
        print("FAIL: gonhanh not loaded (not in AvailableInputMethods).")
        print("      addon .so missing from fcitx5's addon dir, or it failed to dlopen.")
        return 3

    im1 = dbus.Interface(bus.get_object(SERVICE, IM1_PATH), IM1_IFACE)
    path, _uuid = im1.CreateInputContext([("program", "gonhanh-selftest")])
    ic = dbus.Interface(bus.get_object(SERVICE, path), IC1_IFACE)

    buf = {"s": ""}
    cur_im = {"name": None}

    def on_commit(s):
        buf["s"] += str(s)

    def on_delete(offset, size):
        n = int(size)
        if n > 0:
            buf["s"] = buf["s"][:-n] if n <= len(buf["s"]) else ""

    def on_forward(keyval, state, is_release):
        kv = int(keyval)
        if not bool(is_release) and 0x20 <= kv < 0x7f:
            buf["s"] += chr(kv)

    def on_curim(name, unique, lang):
        cur_im["name"] = str(unique)

    m = bus.add_signal_receiver
    m(on_commit,  signal_name="CommitString",          dbus_interface=IC1_IFACE, path=path)
    m(on_delete,  signal_name="DeleteSurroundingText",  dbus_interface=IC1_IFACE, path=path)
    m(on_forward, signal_name="ForwardKey",             dbus_interface=IC1_IFACE, path=path)
    m(on_curim,   signal_name="CurrentIM",              dbus_interface=IC1_IFACE, path=path)

    ic.SetCapability(dbus.UInt64(0))
    ic.FocusIn()
    pump(200)
    ctrl.Activate()
    pump(120)
    ctrl.SetCurrentIM("gonhanh")
    pump(200)

    active = cur_im["name"] or str(ctrl.CurrentInputMethod())
    if active != "gonhanh":
        print(f"FAIL: could not activate gonhanh for the test context (active IM = {active!r}).")
        return 4

    t = 1
    results = []
    for text_in, expected in CASES:
        ic.Reset()
        pump(40)
        buf["s"] = ""
        for ch in text_in:
            keyval, keycode = K[ch]
            handled = ic.ProcessKeyEvent(keyval, keycode, 0, False, t); t += 1
            pump(35)
            # A key fcitx did not consume is typed by the client itself.
            if not bool(handled) and 0x20 <= keyval < 0x7f:
                buf["s"] += chr(keyval)
            ic.ProcessKeyEvent(keyval, keycode, 0, True, t); t += 1
            pump(12)
        pump(50)
        got = buf["s"]
        results.append((text_in, expected, got, got == expected))

    try:
        ic.FocusOut()
        ic.DestroyIC()
    except Exception:
        pass

    print("=== gonhanh telex E2E selftest (live fcitx5 over DBus) ===")
    allok = True
    for text_in, expected, got, ok in results:
        allok = allok and ok
        print(f"  [{'PASS' if ok else 'FAIL'}] {text_in!r:12} -> {got!r:10} (expected {expected!r})")
    print("=== %s ===" % ("ALL PASS" if allok else "FAILURES PRESENT"))
    return 0 if allok else 1


if __name__ == "__main__":
    sys.exit(main())
