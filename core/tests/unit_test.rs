//! Unit Tests - Character & Word transformations
//! All tests defined as data arrays for maximum coverage with minimal code

mod common;
use common::{telex, vni};

// ============================================================
// TELEX: SINGLE VOWELS WITH ALL MARKS
// ============================================================

const TELEX_VOWELS: &[(&str, &str)] = &[
    // a
    ("a", "a"),
    ("as", "á"),
    ("af", "à"),
    ("ar", "ả"),
    ("ax", "ã"),
    ("aj", "ạ"),
    // e
    ("e", "e"),
    ("es", "é"),
    ("ef", "è"),
    ("er", "ẻ"),
    ("ex", "ẽ"),
    ("ej", "ẹ"),
    // i
    ("i", "i"),
    ("is", "í"),
    ("if", "ì"),
    ("ir", "ỉ"),
    ("ix", "ĩ"),
    ("ij", "ị"),
    // o
    ("o", "o"),
    ("os", "ó"),
    ("of", "ò"),
    ("or", "ỏ"),
    ("ox", "õ"),
    ("oj", "ọ"),
    // u
    ("u", "u"),
    ("us", "ú"),
    ("uf", "ù"),
    ("ur", "ủ"),
    ("ux", "ũ"),
    ("uj", "ụ"),
    // y
    ("y", "y"),
    ("ys", "ý"),
    ("yf", "ỳ"),
    ("yr", "ỷ"),
    ("yx", "ỹ"),
    ("yj", "ỵ"),
];

const TELEX_MODIFIED_VOWELS: &[(&str, &str)] = &[
    // â (circumflex)
    ("aa", "â"),
    ("aas", "ấ"),
    ("aaf", "ầ"),
    ("aar", "ẩ"),
    ("aax", "ẫ"),
    ("aaj", "ậ"),
    // ê
    ("ee", "ê"),
    ("ees", "ế"),
    ("eef", "ề"),
    ("eer", "ể"),
    ("eex", "ễ"),
    ("eej", "ệ"),
    // ô
    ("oo", "ô"),
    ("oos", "ố"),
    ("oof", "ồ"),
    ("oor", "ổ"),
    ("oox", "ỗ"),
    ("ooj", "ộ"),
    // ă (breve)
    ("aw", "ă"),
    ("aws", "ắ"),
    ("awf", "ằ"),
    ("awr", "ẳ"),
    ("awx", "ẵ"),
    ("awj", "ặ"),
    // ơ (horn)
    ("ow", "ơ"),
    ("ows", "ớ"),
    ("owf", "ờ"),
    ("owr", "ở"),
    ("owx", "ỡ"),
    ("owj", "ợ"),
    // ư (horn)
    ("uw", "ư"),
    ("uws", "ứ"),
    ("uwf", "ừ"),
    ("uwr", "ử"),
    ("uwx", "ữ"),
    ("uwj", "ự"),
    // đ
    ("dd", "đ"),
    ("DD", "Đ"),
    ("Dd", "Đ"),
];

const TELEX_REVERT: &[(&str, &str)] = &[
    // Mark revert
    ("ass", "as"),
    ("aff", "af"),
    ("arr", "ar"),
    ("axx", "ax"),
    ("ajj", "aj"),
    // Tone revert
    ("aaa", "aa"),
    ("eee", "ee"),
    ("ooo", "oo"),
    ("aww", "aw"),
    ("oww", "ow"),
    ("uww", "uw"),
];

const TELEX_UPPERCASE: &[(&str, &str)] = &[
    ("As", "Á"),
    ("AS", "Á"),
    ("Aa", "Â"),
    ("AA", "Â"),
    ("Aw", "Ă"),
    ("AW", "Ă"),
    ("Ow", "Ơ"),
    ("Uw", "Ư"),
];

const TELEX_DELAYED: &[(&str, &str)] = &[
    ("tuw", "tư"),
    ("tow", "tơ"),
    ("taw", "tă"),
    ("tungw", "tưng"),
    ("tongw", "tơng"),
    ("tuow", "tươ"),
    ("truwowng", "trương"),
];

// ============================================================
// VNI: SINGLE VOWELS WITH ALL MARKS
// ============================================================

const VNI_VOWELS: &[(&str, &str)] = &[
    // a: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    ("a", "a"),
    ("a1", "á"),
    ("a2", "à"),
    ("a3", "ả"),
    ("a4", "ã"),
    ("a5", "ạ"),
    // e
    ("e", "e"),
    ("e1", "é"),
    ("e2", "è"),
    ("e3", "ẻ"),
    ("e4", "ẽ"),
    ("e5", "ẹ"),
    // i
    ("i", "i"),
    ("i1", "í"),
    ("i2", "ì"),
    ("i3", "ỉ"),
    ("i4", "ĩ"),
    ("i5", "ị"),
    // o
    ("o", "o"),
    ("o1", "ó"),
    ("o2", "ò"),
    ("o3", "ỏ"),
    ("o4", "õ"),
    ("o5", "ọ"),
    // u
    ("u", "u"),
    ("u1", "ú"),
    ("u2", "ù"),
    ("u3", "ủ"),
    ("u4", "ũ"),
    ("u5", "ụ"),
    // y
    ("y", "y"),
    ("y1", "ý"),
    ("y2", "ỳ"),
    ("y3", "ỷ"),
    ("y4", "ỹ"),
    ("y5", "ỵ"),
];

const VNI_MODIFIED_VOWELS: &[(&str, &str)] = &[
    // â: 6=circumflex
    ("a6", "â"),
    ("a61", "ấ"),
    ("a62", "ầ"),
    ("a63", "ẩ"),
    ("a64", "ẫ"),
    ("a65", "ậ"),
    // ê
    ("e6", "ê"),
    ("e61", "ế"),
    ("e62", "ề"),
    ("e63", "ể"),
    ("e64", "ễ"),
    ("e65", "ệ"),
    // ô
    ("o6", "ô"),
    ("o61", "ố"),
    ("o62", "ồ"),
    ("o63", "ổ"),
    ("o64", "ỗ"),
    ("o65", "ộ"),
    // ơ: 7=horn
    ("o7", "ơ"),
    ("o71", "ớ"),
    ("o72", "ờ"),
    ("o73", "ở"),
    ("o74", "ỡ"),
    ("o75", "ợ"),
    // ư
    ("u7", "ư"),
    ("u71", "ứ"),
    ("u72", "ừ"),
    ("u73", "ử"),
    ("u74", "ữ"),
    ("u75", "ự"),
    // ă: 8=breve
    ("a8", "ă"),
    ("a81", "ắ"),
    ("a82", "ằ"),
    ("a83", "ẳ"),
    ("a84", "ẵ"),
    ("a85", "ặ"),
    // đ: 9
    ("d9", "đ"),
    ("D9", "Đ"),
];

const VNI_REVERT: &[(&str, &str)] = &[
    ("a11", "a1"),
    ("a22", "a2"),
    ("a33", "a3"),
    ("a44", "a4"),
    ("a55", "a5"),
    ("a66", "a6"),
    ("e66", "e6"),
    ("o66", "o6"),
    ("o77", "o7"),
    ("u77", "u7"),
    ("a88", "a8"),
];

const VNI_UPPERCASE: &[(&str, &str)] = &[
    ("A1", "Á"),
    ("A6", "Â"),
    ("O7", "Ơ"),
    ("U7", "Ư"),
    ("A8", "Ă"),
];

const VNI_DELAYED: &[(&str, &str)] = &[
    ("tu72", "từ"),
    ("to61", "tố"),
    ("ta81", "tắ"),
    ("d9ung1", "đúng"),
    ("du9ng1", "đúng"),
    ("dung91", "đúng"),
    ("dung19", "đúng"),
    ("D9ung1", "Đúng"),
    ("Du9ng1", "Đúng"),
    ("Dung91", "Đúng"),
];

// ============================================================
// WORDS: Common Vietnamese words
// ============================================================

const TELEX_WORDS: &[(&str, &str)] = &[
    // Single vowel
    ("mej", "mẹ"),
    ("bos", "bó"),
    ("cos", "có"),
    ("laf", "là"),
    ("ddi", "đi"),
    // Two vowels - closed syllable
    ("toans", "toán"),
    ("hoanf", "hoàn"),
    ("tieens", "tiến"),
    ("muoons", "muốn"),
    ("bieenr", "biển"),
    ("nguoonf", "nguồn"),
    ("cuoocj", "cuộc"),
    ("thuoocj", "thuộc"),
    // Two vowels - open syllable
    ("hoaf", "hoà"),
    ("hoas", "hoá"),
    ("quyf", "quỳ"),
    ("quys", "quý"),
    ("mais", "mái"),
    ("maif", "mài"),
    ("ddois", "đói"),
    ("tuis", "túi"),
    // Compound ươ
    ("nguwowif", "người"),
    ("muwowif", "mười"),
    ("truwowngf", "trường"),
    ("dduwowngf", "đường"),
    ("ruwowuj", "rượu"),
    ("buwowms", "bướm"),
    ("nuwowcs", "nước"),
    // Compound iê
    ("vieetj", "việt"),
    ("tieengs", "tiếng"),
    ("bieenr", "biển"),
    ("ddieeuf", "điều"),
    // Compound uô
    ("muoons", "muốn"),
    ("cuoocj", "cuộc"),
    ("buoonf", "buồn"),
    ("thuoocj", "thuộc"),
    // Three vowels
    ("khuyeens", "khuyến"),
    ("nguyeenx", "nguyễn"),
    ("loaij", "loại"),
    ("xoaif", "xoài"),
    // đ words
    ("ddeens", "đến"),
    ("ddangf", "đàng"),
    ("ddoongf", "đồng"),
    ("ddepj", "đẹp"),
    ("ddor", "đỏ"),
    // Pronouns
    ("tooi", "tôi"),
    ("banj", "bạn"),
    ("chij", "chị"),
    ("hoj", "họ"),
    ("minhf", "mình"),
    // Verbs
    ("awn", "ăn"),
    ("uoongs", "uống"),
    ("ngur", "ngủ"),
    ("lafm", "làm"),
    ("nois", "nói"),
    ("bieets", "biết"),
    ("hieeur", "hiểu"),
    ("yeeu", "yêu"),
    ("thichs", "thích"),
    // Numbers
    ("mootj", "một"),
    ("boons", "bốn"),
    ("nawm", "năm"),
    ("saus", "sáu"),
    ("bayr", "bảy"),
    ("tams", "tám"),
    ("chins", "chín"),
    ("trawm", "trăm"),
    // Adjectives
    ("toots", "tốt"),
    ("xaaus", "xấu"),
    ("lowns", "lớn"),
    ("nhor", "nhỏ"),
    ("daif", "dài"),
    ("ngawns", "ngắn"),
    ("lanhj", "lạnh"),
    // Uppercase
    ("Chaof", "Chào"),
    ("CHAOF", "CHÀO"),
    ("Vieetj", "Việt"),
    ("DDaats", "Đất"),
    ("DDAATS", "ĐẤT"),
];

const VNI_WORDS: &[(&str, &str)] = &[
    // Pronouns
    ("to6i", "tôi"),
    ("ba5n", "bạn"),
    ("chi5", "chị"),
    ("no1", "nó"),
    ("ho5", "họ"),
    // Verbs
    ("la2", "là"),
    ("co1", "có"),
    ("d9i", "đi"),
    ("d9e61n", "đến"),
    ("ve62", "về"),
    ("a8n", "ăn"),
    ("uo61ng", "uống"),
    ("ngu3", "ngủ"),
    ("la2m", "làm"),
    ("no1i", "nói"),
    ("bie61t", "biết"),
    ("hie63u", "hiểu"),
    ("ye6u", "yêu"),
    // Compound ươ
    ("ngu7o7i2", "người"),
    ("mu7o7i2", "mười"),
    ("tru7o7ng2", "trường"),
    ("d9u7o7ng2", "đường"),
    ("lu7o7i4", "lưỡi"),
    // Compound iê
    ("vie65t", "việt"),
    ("tie61ng", "tiếng"),
    ("bie63n", "biển"),
    ("d9ie62u", "điều"),
    // Compound uô
    ("muo61n", "muốn"),
    ("cuo65c", "cuộc"),
    ("buo62n", "buồn"),
    ("thuo65c", "thuộc"),
    // Three vowels
    ("khuye63n", "khuyển"),
    ("nguye64n", "nguyễn"),
    ("ngoa1i", "ngoái"),
    ("ru7o7u5", "rượu"),
    // Uppercase
    ("Cha2o", "Chào"),
    ("CHA2O", "CHÀO"),
    ("Ngu7o7i2", "Người"),
    ("Vie65t", "Việt"),
    ("D9a61t", "Đất"),
    // Delayed tone
    ("toi6", "tôi"),
    ("toi61", "tối"),
    ("nguoi7", "ngươi"),
    ("nguoi72", "người"),
    ("muon6", "muôn"),
    ("muon61", "muốn"),
];

// ============================================================
// CONSONANT-ONLY & EDGE CASES
// ============================================================

const TELEX_EDGE: &[(&str, &str)] = &[
    // Consonant only
    ("bcd", "bcd"),
    ("xyz", "xyz"),
    // Mark without vowel
    ("bs", "bs"),
    ("ts", "ts"),
    // Consonant clusters
    ("nguyeenx", "nguyễn"),
    ("nhuwngx", "những"),
    ("phaatj", "phật"),
    ("khoongf", "khồng"),
    ("ghees", "ghế"),
    ("truwowcs", "trước"),
    // dd→đ edge cases
    ("ddf", "đf"),
];

// ============================================================
// TEST FUNCTIONS
// ============================================================

#[test]
fn telex_single_vowels() {
    telex(TELEX_VOWELS);
}

#[test]
fn telex_modified_vowels() {
    telex(TELEX_MODIFIED_VOWELS);
}

#[test]
fn telex_revert() {
    telex(TELEX_REVERT);
}

#[test]
fn telex_uppercase() {
    telex(TELEX_UPPERCASE);
}

#[test]
fn telex_delayed_input() {
    telex(TELEX_DELAYED);
}

#[test]
fn telex_words() {
    telex(TELEX_WORDS);
}

#[test]
fn telex_edge_cases() {
    telex(TELEX_EDGE);
}

#[test]
fn vni_single_vowels() {
    vni(VNI_VOWELS);
}

#[test]
fn vni_modified_vowels() {
    vni(VNI_MODIFIED_VOWELS);
}

#[test]
fn vni_revert() {
    vni(VNI_REVERT);
}

#[test]
fn vni_uppercase() {
    vni(VNI_UPPERCASE);
}

#[test]
fn vni_delayed_input() {
    vni(VNI_DELAYED);
}

#[test]
fn vni_words() {
    vni(VNI_WORDS);
}
