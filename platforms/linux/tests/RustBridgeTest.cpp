// Unit tests for RustBridge UTF-8 conversion
// Tests codePointToUtf8() function for Vietnamese character handling

#include <gtest/gtest.h>
#include "../src/RustBridge.h"

// =============================================================================
// UTF-8 Conversion Tests - ASCII
// =============================================================================

TEST(RustBridgeTest, Utf8AsciiCharacters) {
    // ASCII characters (0x00-0x7F) should be single-byte UTF-8
    EXPECT_EQ(RustBridge::codePointToUtf8(0x41), "A");
    EXPECT_EQ(RustBridge::codePointToUtf8(0x61), "a");
    EXPECT_EQ(RustBridge::codePointToUtf8(0x30), "0");
    EXPECT_EQ(RustBridge::codePointToUtf8(0x20), " ");
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0A), "\n");
}

// =============================================================================
// UTF-8 Conversion Tests - Vietnamese Characters
// =============================================================================

TEST(RustBridgeTest, Utf8VietnameseVowels) {
    // Vietnamese vowels with diacritics (2-byte and 3-byte UTF-8)

    // á (U+00E1) - 2-byte: C3 A1
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00E1), "\xC3\xA1");

    // à (U+00E0) - 2-byte: C3 A0
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00E0), "\xC3\xA0");

    // ả (U+1EA3) - 3-byte: E1 BA A3
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EA3), "\xE1\xBA\xA3");

    // ã (U+00E3) - 2-byte: C3 A3
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00E3), "\xC3\xA3");

    // ạ (U+1EA1) - 3-byte: E1 BA A1
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EA1), "\xE1\xBA\xA1");
}

TEST(RustBridgeTest, Utf8VietnameseCircumflex) {
    // Vietnamese circumflex vowels

    // â (U+00E2) - 2-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00E2), "\xC3\xA2");

    // ê (U+00EA) - 2-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00EA), "\xC3\xAA");

    // ô (U+00F4) - 2-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00F4), "\xC3\xB4");

    // ấ (U+1EA5) - 3-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EA5), "\xE1\xBA\xA5");

    // ề (U+1EC1) - 3-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EC1), "\xE1\xBB\x81");
}

TEST(RustBridgeTest, Utf8VietnameseHorn) {
    // Vietnamese horn vowels (ư, ơ)

    // ư (U+01B0) - 2-byte: C6 B0
    EXPECT_EQ(RustBridge::codePointToUtf8(0x01B0), "\xC6\xB0");

    // ơ (U+01A1) - 2-byte: C6 A1
    EXPECT_EQ(RustBridge::codePointToUtf8(0x01A1), "\xC6\xA1");

    // ứ (U+1EE9) - 3-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EE9), "\xE1\xBB\xA9");

    // ờ (U+1EDD) - 3-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EDD), "\xE1\xBB\x9D");
}

TEST(RustBridgeTest, Utf8VietnameseBreve) {
    // Vietnamese breve vowel (ă)

    // ă (U+0103) - 2-byte: C4 83
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0103), "\xC4\x83");

    // ắ (U+1EAF) - 3-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EAF), "\xE1\xBA\xAF");

    // ằ (U+1EB1) - 3-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EB1), "\xE1\xBA\xB1");
}

TEST(RustBridgeTest, Utf8VietnameseDBar) {
    // Vietnamese đ

    // đ (U+0111) - 2-byte: C4 91
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0111), "\xC4\x91");

    // Đ (U+0110) - 2-byte: C4 90
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0110), "\xC4\x90");
}

// =============================================================================
// UTF-8 Conversion Tests - Edge Cases
// =============================================================================

TEST(RustBridgeTest, Utf8BoundaryValues) {
    // Test boundary values for each UTF-8 encoding length

    // 1-byte boundary: U+007F (DEL)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x007F), "\x7F");

    // 2-byte boundary: U+0080 (first 2-byte)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0080), "\xC2\x80");

    // 2-byte boundary: U+07FF (last 2-byte)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x07FF), "\xDF\xBF");

    // 3-byte boundary: U+0800 (first 3-byte)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0800), "\xE0\xA0\x80");

    // 3-byte boundary: U+FFFF (last 3-byte, excluding surrogates)
    EXPECT_EQ(RustBridge::codePointToUtf8(0xFFFF), "\xEF\xBF\xBF");

    // 4-byte boundary: U+10000 (first 4-byte)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x10000), "\xF0\x90\x80\x80");

    // 4-byte boundary: U+10FFFF (last valid Unicode)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x10FFFF), "\xF4\x8F\xBF\xBF");
}

TEST(RustBridgeTest, Utf8InvalidCodepoints) {
    // Invalid codepoints should return replacement character U+FFFD
    const std::string replacement = "\xEF\xBF\xBD";  // U+FFFD

    // Surrogate pair range (U+D800-U+DFFF) is invalid in UTF-8
    EXPECT_EQ(RustBridge::codePointToUtf8(0xD800), replacement);
    EXPECT_EQ(RustBridge::codePointToUtf8(0xDBFF), replacement);
    EXPECT_EQ(RustBridge::codePointToUtf8(0xDC00), replacement);
    EXPECT_EQ(RustBridge::codePointToUtf8(0xDFFF), replacement);

    // Beyond valid Unicode range (> U+10FFFF)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x110000), replacement);
    EXPECT_EQ(RustBridge::codePointToUtf8(0x200000), replacement);
    EXPECT_EQ(RustBridge::codePointToUtf8(0xFFFFFFFF), replacement);
}

TEST(RustBridgeTest, Utf8ZeroCodepoint) {
    // U+0000 (NUL) should be single-byte
    EXPECT_EQ(RustBridge::codePointToUtf8(0x0000), std::string(1, '\0'));
}

// =============================================================================
// UTF-8 Conversion Tests - Common Vietnamese Words
// =============================================================================

TEST(RustBridgeTest, Utf8CommonVietnameseChars) {
    // Test characters from common Vietnamese words

    // "việt" - v, i, ệ, t
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EC7), "\xE1\xBB\x87");  // ệ

    // "nam" - n, a, m (all ASCII)
    EXPECT_EQ(RustBridge::codePointToUtf8(0x6E), "n");
    EXPECT_EQ(RustBridge::codePointToUtf8(0x61), "a");
    EXPECT_EQ(RustBridge::codePointToUtf8(0x6D), "m");

    // "nước" - n, ư, ớ, c
    EXPECT_EQ(RustBridge::codePointToUtf8(0x01B0), "\xC6\xB0");      // ư
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1EDB), "\xE1\xBB\x9B");  // ớ

    // "cộng" - c, ộ, n, g
    EXPECT_EQ(RustBridge::codePointToUtf8(0x1ED9), "\xE1\xBB\x99");  // ộ

    // "hòa" - h, ò, a
    EXPECT_EQ(RustBridge::codePointToUtf8(0x00F2), "\xC3\xB2");      // ò
}

// =============================================================================
// Main
// =============================================================================

int main(int argc, char **argv) {
    testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
