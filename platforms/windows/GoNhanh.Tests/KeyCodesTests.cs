using GoNhanh.Core;
using Xunit;

namespace GoNhanh.Tests;

public class KeyCodesTests
{
    [Theory]
    [InlineData(0x41, 0)] // A
    [InlineData(0x5A, 6)] // Z
    [InlineData(0x30, 29)] // 0
    [InlineData(0x39, 25)] // 9
    public void ToMacKeycode_Letters_MapsCorrectly(int vk, ushort expected)
    {
        var result = KeyCodes.ToMacKeycode(vk);
        Assert.Equal(expected, result);
    }

    [Theory]
    [InlineData(0xDB, 33)] // [ -> ư
    [InlineData(0xDD, 30)] // ] -> ơ
    [InlineData(0x20, 49)] // Space
    [InlineData(0x08, 51)] // Backspace
    public void ToMacKeycode_SpecialKeys_MapsCorrectly(int vk, ushort expected)
    {
        var result = KeyCodes.ToMacKeycode(vk);
        Assert.Equal(expected, result);
    }

    [Fact]
    public void ToMacKeycode_UnmappedKey_Returns65535()
    {
        // F13 is not mapped
        var result = KeyCodes.ToMacKeycode(0x7C);
        Assert.Equal((ushort)0xFFFF, result);
    }

    [Fact]
    public void AllLetters_AreMapped()
    {
        // A-Z (0x41-0x5A)
        for (int vk = 0x41; vk <= 0x5A; vk++)
        {
            var result = KeyCodes.ToMacKeycode(vk);
            Assert.NotEqual((ushort)0xFFFF, result);
        }
    }

    [Fact]
    public void AllNumbers_AreMapped()
    {
        // 0-9 (0x30-0x39)
        for (int vk = 0x30; vk <= 0x39; vk++)
        {
            var result = KeyCodes.ToMacKeycode(vk);
            Assert.NotEqual((ushort)0xFFFF, result);
        }
    }
}
