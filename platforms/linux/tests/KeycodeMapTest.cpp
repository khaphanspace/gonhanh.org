// Unit tests for KeycodeMap.h
// Tests X11/Wayland keysym to macOS keycode mapping

#include <gtest/gtest.h>
#include "../src/KeycodeMap.h"

using namespace KeycodeMap;

// =============================================================================
// Letter Key Mapping Tests
// =============================================================================

TEST(KeycodeMapTest, LowercaseLetters) {
    // Test all lowercase letters map correctly
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_a), MacKey::A);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_b), MacKey::B);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_c), MacKey::C);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_d), MacKey::D);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_e), MacKey::E);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_f), MacKey::F);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_g), MacKey::G);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_h), MacKey::H);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_i), MacKey::I);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_j), MacKey::J);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_k), MacKey::K);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_l), MacKey::L);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_m), MacKey::M);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_n), MacKey::N);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_o), MacKey::O);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_p), MacKey::P);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_q), MacKey::Q);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_r), MacKey::R);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_s), MacKey::S);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_t), MacKey::T);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_u), MacKey::U);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_v), MacKey::V);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_w), MacKey::W);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_x), MacKey::X);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_y), MacKey::Y);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_z), MacKey::Z);
}

TEST(KeycodeMapTest, UppercaseLetters) {
    // Test uppercase letters map to same keycode as lowercase
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_A), MacKey::A);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_B), MacKey::B);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_S), MacKey::S);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Z), MacKey::Z);
}

// =============================================================================
// Number Key Mapping Tests
// =============================================================================

TEST(KeycodeMapTest, NumberKeys) {
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_0), MacKey::N0);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_1), MacKey::N1);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_2), MacKey::N2);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_3), MacKey::N3);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_4), MacKey::N4);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_5), MacKey::N5);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_6), MacKey::N6);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_7), MacKey::N7);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_8), MacKey::N8);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_9), MacKey::N9);
}

TEST(KeycodeMapTest, ShiftedNumberSymbols) {
    // Shifted number keys should map to same keycode (for VNI mode)
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_exclam), MacKey::N1);      // !
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_at), MacKey::N2);          // @
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_numbersign), MacKey::N3);  // #
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_dollar), MacKey::N4);      // $
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_percent), MacKey::N5);     // %
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_asciicircum), MacKey::N6); // ^
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_ampersand), MacKey::N7);   // &
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_asterisk), MacKey::N8);    // *
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_parenleft), MacKey::N9);   // (
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_parenright), MacKey::N0);  // )
}

// =============================================================================
// Punctuation Key Mapping Tests
// =============================================================================

TEST(KeycodeMapTest, PunctuationKeys) {
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_space), MacKey::SPACE);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Return), MacKey::RETURN);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Tab), MacKey::TAB);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_BackSpace), MacKey::DELETE);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Escape), MacKey::ESC);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_comma), MacKey::COMMA);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_period), MacKey::DOT);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_slash), MacKey::SLASH);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_semicolon), MacKey::SEMICOLON);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_apostrophe), MacKey::QUOTE);
}

TEST(KeycodeMapTest, BracketKeys) {
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_bracketleft), MacKey::LBRACKET);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_bracketright), MacKey::RBRACKET);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_braceleft), MacKey::LBRACKET);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_braceright), MacKey::RBRACKET);
}

// =============================================================================
// Arrow Key Mapping Tests
// =============================================================================

TEST(KeycodeMapTest, ArrowKeys) {
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Left), MacKey::LEFT);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Right), MacKey::RIGHT);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Up), MacKey::UP);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Down), MacKey::DOWN);
}

// =============================================================================
// Unknown Key Tests
// =============================================================================

TEST(KeycodeMapTest, UnknownKeysReturnUnknown) {
    // Function keys, multimedia keys, etc. should return UNKNOWN
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_F1), MacKey::UNKNOWN);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_F12), MacKey::UNKNOWN);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Home), MacKey::UNKNOWN);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_End), MacKey::UNKNOWN);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Insert), MacKey::UNKNOWN);
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_Delete), MacKey::UNKNOWN);
    EXPECT_EQ(keysymToMacKeycode(0x12345), MacKey::UNKNOWN);  // Invalid keysym
}

// =============================================================================
// Break Key Detection Tests
// =============================================================================

TEST(KeycodeMapTest, BreakKeysDetected) {
    // Word boundary keys should be detected
    EXPECT_TRUE(isBreakKey(XKB_KEY_space));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Tab));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Return));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Escape));
    EXPECT_TRUE(isBreakKey(XKB_KEY_comma));
    EXPECT_TRUE(isBreakKey(XKB_KEY_period));
    EXPECT_TRUE(isBreakKey(XKB_KEY_slash));
    EXPECT_TRUE(isBreakKey(XKB_KEY_semicolon));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Left));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Right));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Up));
    EXPECT_TRUE(isBreakKey(XKB_KEY_Down));
}

TEST(KeycodeMapTest, NonBreakKeysNotDetected) {
    // Letters and numbers are NOT break keys
    EXPECT_FALSE(isBreakKey(XKB_KEY_a));
    EXPECT_FALSE(isBreakKey(XKB_KEY_A));
    EXPECT_FALSE(isBreakKey(XKB_KEY_z));
    EXPECT_FALSE(isBreakKey(XKB_KEY_0));
    EXPECT_FALSE(isBreakKey(XKB_KEY_9));
}

// =============================================================================
// Letter Key Detection Tests
// =============================================================================

TEST(KeycodeMapTest, LetterKeysDetected) {
    EXPECT_TRUE(isLetterKey(XKB_KEY_a));
    EXPECT_TRUE(isLetterKey(XKB_KEY_A));
    EXPECT_TRUE(isLetterKey(XKB_KEY_z));
    EXPECT_TRUE(isLetterKey(XKB_KEY_Z));
    EXPECT_TRUE(isLetterKey(XKB_KEY_m));
    EXPECT_TRUE(isLetterKey(XKB_KEY_M));
}

TEST(KeycodeMapTest, NonLetterKeysNotDetected) {
    EXPECT_FALSE(isLetterKey(XKB_KEY_0));
    EXPECT_FALSE(isLetterKey(XKB_KEY_9));
    EXPECT_FALSE(isLetterKey(XKB_KEY_space));
    EXPECT_FALSE(isLetterKey(XKB_KEY_comma));
    EXPECT_FALSE(isLetterKey(XKB_KEY_Return));
}

// =============================================================================
// Number Key Detection Tests
// =============================================================================

TEST(KeycodeMapTest, NumberKeysDetected) {
    EXPECT_TRUE(isNumberKey(XKB_KEY_0));
    EXPECT_TRUE(isNumberKey(XKB_KEY_1));
    EXPECT_TRUE(isNumberKey(XKB_KEY_5));
    EXPECT_TRUE(isNumberKey(XKB_KEY_9));
}

TEST(KeycodeMapTest, NonNumberKeysNotDetected) {
    EXPECT_FALSE(isNumberKey(XKB_KEY_a));
    EXPECT_FALSE(isNumberKey(XKB_KEY_space));
    EXPECT_FALSE(isNumberKey(XKB_KEY_exclam));  // Shifted numbers are not "number keys"
}

// =============================================================================
// Vietnamese IME Specific Tests
// =============================================================================

TEST(KeycodeMapTest, TelexToneKeys) {
    // Telex tone keys: s, f, r, x, j
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_s), MacKey::S);  // sắc
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_f), MacKey::F);  // huyền
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_r), MacKey::R);  // hỏi
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_x), MacKey::X);  // ngã
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_j), MacKey::J);  // nặng
}

TEST(KeycodeMapTest, TelexVowelKeys) {
    // Telex vowel modifier keys: a, e, o, w
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_a), MacKey::A);  // â (aa)
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_e), MacKey::E);  // ê (ee)
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_o), MacKey::O);  // ô (oo)
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_w), MacKey::W);  // ư, ơ (uw, ow)
}

TEST(KeycodeMapTest, VNIToneKeys) {
    // VNI uses number keys 1-5 for tones
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_1), MacKey::N1);  // sắc
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_2), MacKey::N2);  // huyền
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_3), MacKey::N3);  // hỏi
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_4), MacKey::N4);  // ngã
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_5), MacKey::N5);  // nặng
}

TEST(KeycodeMapTest, VNIVowelKeys) {
    // VNI uses 6, 7, 8 for vowel modifications
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_6), MacKey::N6);  // â, ô, ê
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_7), MacKey::N7);  // ư, ơ
    EXPECT_EQ(keysymToMacKeycode(XKB_KEY_8), MacKey::N8);  // ă
}

// =============================================================================
// Main
// =============================================================================

int main(int argc, char **argv) {
    testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
