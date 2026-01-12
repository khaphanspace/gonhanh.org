using System;
using System.Runtime.InteropServices;
using System.Text;

namespace GoNhanh.Core;

/// <summary>
/// Result struct returned by ime_key(). Must match Rust layout exactly.
/// Rust: pub struct Result { chars: [u32; 256], action: u8, backspace: u8, count: u8, flags: u8 }
/// </summary>
[StructLayout(LayoutKind.Sequential)]
public struct ImeResult
{
    /// <summary>UTF-32 codepoints to insert (max 256)</summary>
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 256)]
    public uint[] chars;

    /// <summary>Action: 0=None (pass-through), 1=Send (replace), 2=Restore</summary>
    public byte action;

    /// <summary>Number of characters to backspace before inserting</summary>
    public byte backspace;

    /// <summary>Number of valid codepoints in chars array</summary>
    public byte count;

    /// <summary>Flags: bit 0 = key consumed by shortcut</summary>
    public byte flags;

    /// <summary>Check if key was consumed (don't pass through)</summary>
    public bool KeyConsumed => (flags & 0x01) != 0;

    /// <summary>Convert chars to UTF-16 string</summary>
    public string GetText()
    {
        if (chars == null || count == 0) return string.Empty;
        var sb = new StringBuilder(count);
        for (int i = 0; i < count; i++)
        {
            sb.Append(char.ConvertFromUtf32((int)chars[i]));
        }
        return sb.ToString();
    }
}

/// <summary>Action types returned by engine</summary>
public enum ImeAction : byte
{
    /// <summary>Pass keystroke through unchanged</summary>
    None = 0,
    /// <summary>Send backspaces + replacement text</summary>
    Send = 1,
    /// <summary>Restore original input (ESC key)</summary>
    Restore = 2
}

/// <summary>Input method types</summary>
public enum InputMethod : byte
{
    Telex = 0,
    VNI = 1
}

/// <summary>
/// P/Invoke bindings to gonhanh_core.dll (Rust IME engine)
/// </summary>
public static class RustBridge
{
    private const string DLL = "gonhanh_core.dll";
    private const CallingConvention CC = CallingConvention.Cdecl;

    // ========== Core Functions ==========

    /// <summary>Initialize engine. Call once at app start.</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_init();

    /// <summary>Process key event. Returns pointer to ImeResult.</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern IntPtr ime_key(ushort key, [MarshalAs(UnmanagedType.U1)] bool caps,
        [MarshalAs(UnmanagedType.U1)] bool ctrl);

    /// <summary>Process key with shift parameter (for VNI Shift+number).</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern IntPtr ime_key_ext(ushort key, [MarshalAs(UnmanagedType.U1)] bool caps,
        [MarshalAs(UnmanagedType.U1)] bool ctrl, [MarshalAs(UnmanagedType.U1)] bool shift);

    /// <summary>Free result pointer. Must call after each ime_key().</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_free(IntPtr result);

    /// <summary>Set input method (0=Telex, 1=VNI).</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_method(byte method);

    /// <summary>Enable/disable engine.</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_enabled([MarshalAs(UnmanagedType.U1)] bool enabled);

    /// <summary>Clear buffer on word boundary.</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_clear();

    /// <summary>Clear all state including history.</summary>
    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_clear_all();

    // ========== Engine Options ==========

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_skip_w_shortcut([MarshalAs(UnmanagedType.U1)] bool skip);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_bracket_shortcut([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_esc_restore([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_free_tone([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_modern([MarshalAs(UnmanagedType.U1)] bool modern);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_english_auto_restore([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_auto_capitalize([MarshalAs(UnmanagedType.U1)] bool enabled);

    // ========== Shortcuts ==========

    [DllImport(DLL, CallingConvention = CC, CharSet = CharSet.Ansi)]
    public static extern void ime_add_shortcut(
        [MarshalAs(UnmanagedType.LPStr)] string trigger,
        [MarshalAs(UnmanagedType.LPStr)] string replacement);

    [DllImport(DLL, CallingConvention = CC, CharSet = CharSet.Ansi)]
    public static extern void ime_remove_shortcut([MarshalAs(UnmanagedType.LPStr)] string trigger);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern void ime_clear_shortcuts();

    // ========== Word Restore ==========

    [DllImport(DLL, CallingConvention = CC, CharSet = CharSet.Ansi)]
    public static extern void ime_restore_word([MarshalAs(UnmanagedType.LPStr)] string word);

    [DllImport(DLL, CallingConvention = CC)]
    public static extern long ime_get_buffer(IntPtr buffer, long maxLen);
}

/// <summary>
/// High-level wrapper for Rust IME engine
/// </summary>
public sealed class ImeEngine : IDisposable
{
    private static ImeEngine? _instance;
    private bool _initialized;
    private bool _disposed;

    public static ImeEngine Instance => _instance ??= new ImeEngine();

    private ImeEngine() { }

    /// <summary>Initialize the engine (call once)</summary>
    public void Initialize()
    {
        if (_initialized) return;
        RustBridge.ime_init();
        _initialized = true;
    }

    /// <summary>Process a key event</summary>
    public ImeResult? ProcessKey(ushort keycode, bool caps, bool ctrl, bool shift = false)
    {
        if (!_initialized) return null;

        IntPtr ptr = shift
            ? RustBridge.ime_key_ext(keycode, caps, ctrl, shift)
            : RustBridge.ime_key(keycode, caps, ctrl);

        if (ptr == IntPtr.Zero) return null;

        try
        {
            return Marshal.PtrToStructure<ImeResult>(ptr);
        }
        finally
        {
            RustBridge.ime_free(ptr);
        }
    }

    /// <summary>Set input method</summary>
    public void SetMethod(InputMethod method)
    {
        RustBridge.ime_method((byte)method);
    }

    /// <summary>Enable/disable engine</summary>
    public void SetEnabled(bool enabled)
    {
        RustBridge.ime_enabled(enabled);
    }

    /// <summary>Clear buffer (call on word boundary)</summary>
    public void Clear()
    {
        RustBridge.ime_clear();
    }

    /// <summary>Clear all state (call on cursor change)</summary>
    public void ClearAll()
    {
        RustBridge.ime_clear_all();
    }

    // Options
    public void SetSkipWShortcut(bool skip) => RustBridge.ime_skip_w_shortcut(skip);
    public void SetBracketShortcut(bool enabled) => RustBridge.ime_bracket_shortcut(enabled);
    public void SetEscRestore(bool enabled) => RustBridge.ime_esc_restore(enabled);
    public void SetFreeTone(bool enabled) => RustBridge.ime_free_tone(enabled);
    public void SetModernTone(bool modern) => RustBridge.ime_modern(modern);
    public void SetEnglishAutoRestore(bool enabled) => RustBridge.ime_english_auto_restore(enabled);
    public void SetAutoCapitalize(bool enabled) => RustBridge.ime_auto_capitalize(enabled);

    // Shortcuts
    public void AddShortcut(string trigger, string replacement)
        => RustBridge.ime_add_shortcut(trigger, replacement);
    public void RemoveShortcut(string trigger) => RustBridge.ime_remove_shortcut(trigger);
    public void ClearShortcuts() => RustBridge.ime_clear_shortcuts();

    // Word restore
    public void RestoreWord(string word) => RustBridge.ime_restore_word(word);

    public void Dispose()
    {
        if (_disposed) return;
        // Rust engine uses global state, no explicit cleanup needed
        _disposed = true;
    }
}
