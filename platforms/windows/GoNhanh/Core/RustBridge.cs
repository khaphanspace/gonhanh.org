using System.Runtime.InteropServices;
using System.Text;

namespace GoNhanh.Core;

/// <summary>
/// High-performance P/Invoke bridge to Rust core library (gonhanh_core.dll)
/// FFI contract matches core/src/lib.rs exports
/// Optimized with cached StringBuilder for minimal allocations
/// </summary>
public static class RustBridge
{
    private const string DllName = "gonhanh_core.dll";

    #region Native Imports

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_init();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_clear();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_free(IntPtr result);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_method(byte method);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_enabled([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_modern([MarshalAs(UnmanagedType.U1)] bool modern);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_skip_w_shortcut([MarshalAs(UnmanagedType.U1)] bool skip);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_esc_restore([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_english_auto_restore([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_auto_capitalize([MarshalAs(UnmanagedType.U1)] bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr ime_key(ushort keycode, [MarshalAs(UnmanagedType.U1)] bool shift, [MarshalAs(UnmanagedType.U1)] bool capslock);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    private static extern void ime_add_shortcut(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string trigger,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string replacement);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    private static extern void ime_remove_shortcut([MarshalAs(UnmanagedType.LPUTF8Str)] string trigger);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    private static extern void ime_clear_shortcuts();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    private static extern int version_compare(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string v1,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string v2);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    private static extern int version_has_update(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string current,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string latest);

    #endregion

    #region Cached Objects

    // Thread-local StringBuilder to avoid allocations on each GetText() call
    [ThreadStatic]
    private static StringBuilder? _cachedStringBuilder;

    private static StringBuilder GetStringBuilder()
    {
        _cachedStringBuilder ??= new StringBuilder(64);
        _cachedStringBuilder.Clear();
        return _cachedStringBuilder;
    }

    #endregion

    #region Public API

    /// <summary>
    /// Initialize the IME engine. Call once at startup.
    /// </summary>
    public static void Initialize()
    {
        ime_init();
    }

    /// <summary>
    /// Clear the typing buffer.
    /// </summary>
    public static void Clear()
    {
        ime_clear();
    }

    /// <summary>
    /// Set input method (Telex=0, VNI=1)
    /// </summary>
    public static void SetMethod(InputMethod method)
    {
        ime_method((byte)method);
    }

    /// <summary>
    /// Enable or disable IME processing
    /// </summary>
    public static void SetEnabled(bool enabled)
    {
        ime_enabled(enabled);
    }

    /// <summary>
    /// Set tone style (modern=true: hòa, old=false: hoà)
    /// </summary>
    public static void SetModernTone(bool modern)
    {
        ime_modern(modern);
    }

    /// <summary>
    /// Skip W shortcut at word start (W -> Ư)
    /// When false: typing W at word start produces Ư
    /// When true: typing W produces normal W
    /// </summary>
    public static void SetSkipWShortcut(bool skip)
    {
        try { ime_skip_w_shortcut(skip); } catch { }
    }

    /// <summary>
    /// Enable ESC key to restore original text
    /// </summary>
    public static void SetEscRestore(bool enabled)
    {
        try { ime_esc_restore(enabled); } catch { }
    }

    /// <summary>
    /// Enable automatic restoration of English words
    /// </summary>
    public static void SetEnglishAutoRestore(bool enabled)
    {
        try { ime_english_auto_restore(enabled); } catch { }
    }

    /// <summary>
    /// Enable auto-capitalization after sentence-ending punctuation
    /// </summary>
    public static void SetAutoCapitalize(bool enabled)
    {
        try { ime_auto_capitalize(enabled); } catch { }
    }

    /// <summary>
    /// Add a text abbreviation shortcut
    /// </summary>
    public static void AddShortcut(string trigger, string replacement)
    {
        if (string.IsNullOrEmpty(trigger) || string.IsNullOrEmpty(replacement)) return;
        try { ime_add_shortcut(trigger, replacement); } catch { }
    }

    /// <summary>
    /// Remove a text abbreviation shortcut
    /// </summary>
    public static void RemoveShortcut(string trigger)
    {
        if (string.IsNullOrEmpty(trigger)) return;
        try { ime_remove_shortcut(trigger); } catch { }
    }

    /// <summary>
    /// Clear all shortcuts
    /// </summary>
    public static void ClearShortcuts()
    {
        try { ime_clear_shortcuts(); } catch { }
    }

    /// <summary>
    /// Sync shortcuts from a list - clears existing and adds enabled ones
    /// </summary>
    public static void SyncShortcuts(IEnumerable<(string Trigger, string Replacement, bool Enabled)> shortcuts)
    {
        ClearShortcuts();
        foreach (var (trigger, replacement, enabled) in shortcuts)
        {
            if (enabled)
            {
                AddShortcut(trigger, replacement);
            }
        }
    }

    /// <summary>
    /// Compare two version strings using Rust core
    /// Returns: -1 if v1 < v2, 0 if equal, 1 if v1 > v2, -99 on error
    /// </summary>
    public static int CompareVersions(string v1, string v2)
    {
        if (string.IsNullOrEmpty(v1) || string.IsNullOrEmpty(v2)) return -99;
        try { return version_compare(v1, v2); } catch { return -99; }
    }

    /// <summary>
    /// Check if update is available
    /// Returns: true if latest > current
    /// </summary>
    public static bool HasUpdate(string current, string latest)
    {
        if (string.IsNullOrEmpty(current) || string.IsNullOrEmpty(latest)) return false;
        try { return version_has_update(current, latest) == 1; } catch { return false; }
    }

    /// <summary>
    /// Process a keystroke and get the result
    /// </summary>
    public static ImeResult ProcessKey(ushort keycode, bool shift, bool capslock)
    {
        IntPtr ptr = ime_key(keycode, shift, capslock);
        if (ptr == IntPtr.Zero)
        {
            return ImeResult.Empty;
        }

        try
        {
            var native = Marshal.PtrToStructure<NativeResult>(ptr);
            return ImeResult.FromNative(native, GetStringBuilder());
        }
        finally
        {
            ime_free(ptr);
        }
    }

    #endregion
}

/// <summary>
/// Input method type
/// </summary>
public enum InputMethod : byte
{
    Telex = 0,
    VNI = 1
}

/// <summary>
/// IME action type
/// </summary>
public enum ImeAction : byte
{
    None = 0,    // No action needed
    Send = 1,    // Send text replacement
    Restore = 2  // Restore original text
}

/// <summary>
/// Native result structure from Rust (must match core/src/lib.rs)
/// </summary>
[StructLayout(LayoutKind.Sequential)]
internal struct NativeResult
{
    [MarshalAs(UnmanagedType.ByValArray, SizeConst = 32)]
    public uint[] chars;
    public byte action;
    public byte backspace;
    public byte count;
    public byte _pad;
}

/// <summary>
/// Managed IME result with optimized string conversion
/// </summary>
public readonly struct ImeResult
{
    public readonly ImeAction Action;
    public readonly byte Backspace;
    public readonly byte Count;
    private readonly string _text;

    public static readonly ImeResult Empty = new(ImeAction.None, 0, 0, string.Empty);

    private ImeResult(ImeAction action, byte backspace, byte count, string text)
    {
        Action = action;
        Backspace = backspace;
        Count = count;
        _text = text;
    }

    internal static ImeResult FromNative(NativeResult native, StringBuilder sb)
    {
        string text = string.Empty;
        if (native.count > 0 && native.chars != null)
        {
            for (int i = 0; i < native.count && i < native.chars.Length; i++)
            {
                if (native.chars[i] > 0)
                {
                    sb.Append(char.ConvertFromUtf32((int)native.chars[i]));
                }
            }
            text = sb.ToString();
        }

        return new ImeResult(
            (ImeAction)native.action,
            native.backspace,
            native.count,
            text
        );
    }

    /// <summary>
    /// Get the result text as a string
    /// </summary>
    public string GetText() => _text;
}
