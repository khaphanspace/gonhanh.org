// GoNhanh Windows Test Harness
// Tests basic DLL loading and FFI calls

using System;
using System.Runtime.InteropServices;

namespace GoNhanh.TestHarness;

class Program
{
    // FFI imports from gonhanh_core.dll
    [DllImport("gonhanh_core.dll", CallingConvention = CallingConvention.Cdecl)]
    static extern void ime_init();

    [DllImport("gonhanh_core.dll", CallingConvention = CallingConvention.Cdecl)]
    static extern void ime_method(byte method);

    [DllImport("gonhanh_core.dll", CallingConvention = CallingConvention.Cdecl)]
    static extern void ime_enabled(bool enabled);

    [DllImport("gonhanh_core.dll", CallingConvention = CallingConvention.Cdecl)]
    static extern void ime_clear();

    [DllImport("gonhanh_core.dll", CallingConvention = CallingConvention.Cdecl)]
    static extern IntPtr ime_key(ushort keycode, bool caps, bool ctrl);

    [DllImport("gonhanh_core.dll", CallingConvention = CallingConvention.Cdecl)]
    static extern void ime_free(IntPtr result);

    // ImeResult structure (matches Rust FFI)
    [StructLayout(LayoutKind.Sequential)]
    struct ImeResult
    {
        [MarshalAs(UnmanagedType.ByValArray, SizeConst = 256)]
        public uint[] chars;      // Unicode chars
        public byte action;       // 0=None, 1=Send, 2=Restore
        public byte backspace;    // Backspace count
        public byte count;        // Char count
        public byte flags;        // Reserved
    }

    static void Main(string[] args)
    {
        Console.WriteLine("GoNhanh Windows Test Harness");
        Console.WriteLine("============================\n");

        try
        {
            // Test 1: Initialize
            Console.Write("Test 1: ime_init() ... ");
            ime_init();
            Console.WriteLine("OK");

            // Test 2: Set method (Telex=0, VNI=1)
            Console.Write("Test 2: ime_method(0) [Telex] ... ");
            ime_method(0);
            Console.WriteLine("OK");

            // Test 3: Enable engine
            Console.Write("Test 3: ime_enabled(true) ... ");
            ime_enabled(true);
            Console.WriteLine("OK");

            // Test 4: Process key (simulate 'a' key, keycode 0)
            Console.Write("Test 4: ime_key('a') ... ");
            IntPtr resultPtr = ime_key(0, false, false);
            if (resultPtr != IntPtr.Zero)
            {
                var result = Marshal.PtrToStructure<ImeResult>(resultPtr);
                Console.WriteLine($"OK (action={result.action}, count={result.count})");
                ime_free(resultPtr);
            }
            else
            {
                Console.WriteLine("OK (null result)");
            }

            // Test 5: Clear buffer
            Console.Write("Test 5: ime_clear() ... ");
            ime_clear();
            Console.WriteLine("OK");

            Console.WriteLine("\n=============================");
            Console.WriteLine("All tests passed!");
            Console.WriteLine("DLL loaded and initialized successfully.");
        }
        catch (DllNotFoundException ex)
        {
            Console.WriteLine($"\nERROR: DLL not found - {ex.Message}");
            Console.WriteLine("Make sure gonhanh_core.dll is in the output directory.");
            Environment.Exit(1);
        }
        catch (Exception ex)
        {
            Console.WriteLine($"\nERROR: {ex.GetType().Name} - {ex.Message}");
            Environment.Exit(1);
        }
    }
}
