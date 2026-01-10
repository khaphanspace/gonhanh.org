using System.Runtime.InteropServices;

namespace GoNhanh.Core;

/// <summary>
/// FFI bridge to Rust core engine (gonhanh_core.dll).
/// Must match Rust Result struct byte-for-byte.
/// </summary>
public static class RustBridge
{
    private const string DllName = "gonhanh_core";
    private const int MAX_CHARS = 256;

    // ============================================================
    // Result Struct (matches Rust #[repr(C)])
    // ============================================================

    /// <summary>
    /// Engine result - 1028 bytes total.
    /// chars: 256 * 4 = 1024 bytes
    /// action + backspace + count + flags = 4 bytes
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct ImeResult
    {
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = MAX_CHARS)]
        public uint[] chars;  // UTF-32 codepoints
        public byte action;    // 0=None, 1=Send, 2=Restore
        public byte backspace; // Chars to delete
        public byte count;     // Valid output chars
        public byte flags;     // bit 0 = key_consumed
    }

    public enum Action : byte
    {
        None = 0,
        Send = 1,
        Restore = 2
    }

    // ============================================================
    // Core FFI Functions
    // ============================================================

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_init();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr ime_key(ushort key, bool caps, bool ctrl);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern IntPtr ime_key_ext(ushort key, bool caps, bool ctrl, bool shift);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_method(byte method);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_enabled(bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_clear();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_clear_all();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_free(IntPtr result);

    // ============================================================
    // Configuration FFI Functions
    // ============================================================

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_skip_w_shortcut(bool skip);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_bracket_shortcut(bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_esc_restore(bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_free_tone(bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_modern(bool modern);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_english_auto_restore(bool enabled);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_auto_capitalize(bool enabled);

    // ============================================================
    // Shortcut FFI Functions
    // ============================================================

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern void ime_add_shortcut(string trigger, string replacement);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern void ime_remove_shortcut(string trigger);

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern void ime_clear_shortcuts();

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
    public static extern void ime_restore_word(string word);

    // ============================================================
    // Buffer FFI Functions
    // ============================================================

    [DllImport(DllName, CallingConvention = CallingConvention.Cdecl)]
    public static extern long ime_get_buffer(IntPtr output, long maxLen);

    // ============================================================
    // High-Level Wrapper
    // ============================================================

    /// <summary>
    /// Process a key event and return result with proper cleanup.
    /// Thread-safe: Rust engine uses internal mutex.
    /// </summary>
    public static (Action action, byte backspace, string output, bool consumed) ProcessKey(
        ushort keycode, bool caps, bool ctrl, bool shift = false)
    {
        IntPtr ptr = shift
            ? ime_key_ext(keycode, caps, ctrl, shift)
            : ime_key(keycode, caps, ctrl);

        if (ptr == IntPtr.Zero)
            return (Action.None, 0, string.Empty, false);

        try
        {
            var result = Marshal.PtrToStructure<ImeResult>(ptr);

            // Convert UTF-32 to string (handle surrogate pairs for emoji etc.)
            var sb = new System.Text.StringBuilder(result.count);
            for (int i = 0; i < result.count; i++)
            {
                uint codepoint = result.chars[i];

                // Validate codepoint range (U+0000 to U+10FFFF, excluding surrogates)
                if (codepoint > 0x10FFFF || (codepoint >= 0xD800 && codepoint <= 0xDFFF))
                    continue; // Skip invalid codepoints

                if (codepoint <= 0xFFFF)
                {
                    sb.Append((char)codepoint);
                }
                else
                {
                    // Handle supplementary planes (emoji, rare CJK)
                    codepoint -= 0x10000;
                    sb.Append((char)(0xD800 + (codepoint >> 10)));
                    sb.Append((char)(0xDC00 + (codepoint & 0x3FF)));
                }
            }

            return (
                (Action)result.action,
                result.backspace,
                sb.ToString(),
                (result.flags & 0x01) != 0
            );
        }
        finally
        {
            ime_free(ptr);
        }
    }

    /// <summary>
    /// Get current buffer content as string.
    /// </summary>
    public static string GetBuffer()
    {
        IntPtr buffer = Marshal.AllocHGlobal(MAX_CHARS * sizeof(uint));
        try
        {
            long len = ime_get_buffer(buffer, MAX_CHARS);
            if (len <= 0) return string.Empty;

            var sb = new System.Text.StringBuilder((int)len);
            for (int i = 0; i < len; i++)
            {
                uint codepoint = (uint)Marshal.ReadInt32(buffer, i * sizeof(uint));

                // Validate codepoint range
                if (codepoint > 0x10FFFF || (codepoint >= 0xD800 && codepoint <= 0xDFFF))
                    continue;

                if (codepoint <= 0xFFFF)
                {
                    sb.Append((char)codepoint);
                }
                else
                {
                    codepoint -= 0x10000;
                    sb.Append((char)(0xD800 + (codepoint >> 10)));
                    sb.Append((char)(0xDC00 + (codepoint & 0x3FF)));
                }
            }
            return sb.ToString();
        }
        finally
        {
            Marshal.FreeHGlobal(buffer);
        }
    }
}
