using System.Runtime.InteropServices;
using GoNhanh.Core;
using Xunit;

namespace GoNhanh.Tests;

public class ImeResultTests
{
    [Fact]
    public void ImeResult_HasCorrectSize()
    {
        // 256 * 4 (uint) + 4 bytes = 1028 bytes
        var size = Marshal.SizeOf<ImeResult>();
        Assert.Equal(1028, size);
    }

    [Fact]
    public void ImeResult_CharsArray_Has256Elements()
    {
        var result = new ImeResult();
        result.chars = new uint[256];
        Assert.Equal(256, result.chars.Length);
    }

    [Fact]
    public void ImeResult_ActionValues_AreCorrect()
    {
        // Action 0 = Noop, 1 = Insert, 2 = Replace
        var result = new ImeResult
        {
            chars = new uint[256],
            action = 0,
            backspace = 0,
            count = 0,
            flags = 0
        };

        Assert.Equal(0, result.action);
    }

    [Fact]
    public void ImeResult_CanStoreVietnameseCharacters()
    {
        var result = new ImeResult
        {
            chars = new uint[256]
        };

        // Vietnamese characters as code points
        result.chars[0] = 0x1EA1; // ạ
        result.chars[1] = 0x1EBF; // ế
        result.chars[2] = 0x1EC9; // ỉ
        result.count = 3;

        Assert.Equal((uint)0x1EA1, result.chars[0]);
        Assert.Equal((uint)0x1EBF, result.chars[1]);
        Assert.Equal((uint)0x1EC9, result.chars[2]);
    }

    [Fact]
    public void ImeResult_Layout_IsSequential()
    {
        // Verify struct layout attribute
        var attr = typeof(ImeResult).GetCustomAttributes(typeof(StructLayoutAttribute), false);
        Assert.NotEmpty(attr);

        var layout = (StructLayoutAttribute)attr[0];
        Assert.Equal(LayoutKind.Sequential, layout.Value);
    }
}
