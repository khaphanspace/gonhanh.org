//! Typing Tests - Real-world typing scenarios, sentences, behaviors

mod common;
use common::{telex, vni};

// ============================================================
// BACKSPACE & CORRECTIONS
// ============================================================

const TELEX_BACKSPACE: &[(&str, &str)] = &[
    ("vieet<s", "viế"),
    ("chaof<o", "chào"),
    ("toi<as", "toá"),
    ("a<b", "b"),
    ("ab<<cd", "cd"),
    ("abcd<<<", "a"),
    ("vieets<<<ng", "vng"),
];

const VNI_BACKSPACE: &[(&str, &str)] = &[("a1<a2", "à"), ("o6<o7", "ơ")];

// ============================================================
// TYPOS & WRONG ORDER
// ============================================================

const TELEX_TYPOS: &[(&str, &str)] = &[
    // Wrong order - mark before vowel
    ("sa", "sa"),
    ("as", "á"),
    // Double mark → revert
    ("ass", "as"),
    ("aff", "af"),
    ("arr", "ar"),
    // Double tone → revert
    ("aaa", "aa"),
    ("ooo", "oo"),
    ("aww", "aw"),
    // Change mark mid-word
    ("asf", "à"),
    ("afs", "á"),
    // Tone then mark
    ("aas", "ấ"),
    ("ees", "ế"),
    ("oos", "ố"),
    // Mark then tone
    ("asa", "ấ"),
    ("oso", "ố"),
];

// ============================================================
// RAPID TYPING PATTERNS
// ============================================================

const TELEX_RAPID: &[(&str, &str)] = &[
    ("ngoafif", "ngoài"),
    ("nguwowif", "người"),
    // Common words typed fast
    ("truwowngf", "trường"),
    ("dduwowcj", "được"),
    ("suwowngs", "sướng"),
    ("buwowms", "bướm"),
    ("vieetj", "việt"),
    ("tieengs", "tiếng"),
    ("muoons", "muốn"),
    ("cuoocj", "cuộc"),
    ("thuoocj", "thuộc"),
    // ============================================================
    // "ưu" vowel cluster - horn on FIRST u (DELAYED modifier pattern)
    // ============================================================
    // 2-char
    ("uuw", "ưu"), // advantage prefix
    // 3-char (consonant + ưu)
    ("luuw", "lưu"), // save/store
    ("huuw", "hưu"), // retire
    ("suuw", "sưu"), // collect (sưu tầm)
    // 3-char with mark (consonant + ứu/ừu/ửu/ữu/ựu)
    ("cuuws", "cứu"), // rescue
    ("luuws", "lứu"), // (test pattern)
    ("luuwf", "lừu"), // (test pattern)
    ("luuwr", "lửu"), // (test pattern)
    ("luuwx", "lữu"), // (test pattern)
    ("luuwj", "lựu"), // pomegranate
    // 4-char (2-consonant cluster + ưu)
    ("nguuw", "ngưu"), // ox/cow (牛)
    ("khuuw", "khưu"), // (test pattern)
    ("truuw", "trưu"), // (test pattern)
    ("thuuw", "thưu"), // (test pattern)
    // 4-char with mark
    ("nguuws", "ngứu"), // ngứu: delayed pattern
    ("khuuwf", "khừu"), // khừu: delayed pattern
    // 5-char "nghiêu" pattern (ngh + iê + u)
    ("nghieeu", "nghiêu"), // nghiêu: valid ngh + front vowels
    // Alternative typing (inline modifier)
    ("luwu", "lưu"),   // inline
    ("cuwus", "cứu"),  // inline with mark
    ("nguwu", "ngưu"), // inline 4-char
];

const VNI_RAPID: &[(&str, &str)] = &[
    ("ngu7o72i2", "người"),
    ("to6i1", "tối"),
    // "ưu" vowel cluster - horn on FIRST u (not second)
    ("luu7", "lưu"),  // save/store - DELAYED modifier
    ("lu7u", "lưu"),  // inline modifier
    ("uu7", "ưu"),    // advantage prefix - DELAYED
    ("luu71", "lứu"), // with mark - DELAYED
    ("huu7", "hưu"),  // retire - DELAYED
    ("cuu71", "cứu"), // save/rescue - DELAYED
];

// ============================================================
// CAPITALIZATION
// ============================================================

const TELEX_CAPS: &[(&str, &str)] = &[
    ("viEets", "viẾt"),
    ("VIEETJ", "VIỆT"),
    ("VIEETS", "VIẾT"),
    ("DDUWOWNGF", "ĐƯỜNG"),
    ("DDUWOWCJ", "ĐƯỢC"),
    ("TRUWOWNGF", "TRƯỜNG"),
    ("NGUWOWIF", "NGƯỜI"),
];

const VNI_CAPS: &[(&str, &str)] = &[
    ("VIE65T", "VIỆT"),
    ("D9U7O7NG2", "ĐƯỜNG"),
    ("D9U7O7C5", "ĐƯỢC"),
    ("TRU7O7NG2", "TRƯỜNG"),
];

// ============================================================
// GREETINGS
// ============================================================

const TELEX_GREETINGS: &[(&str, &str)] = &[
    ("xin chaof", "xin chào"),
    ("tamj bieetj", "tạm biệt"),
    ("camr own", "cảm ơn"),
    ("xin looxix", "xin lỗi"),
];

const VNI_GREETINGS: &[(&str, &str)] = &[
    ("xin cha2o", "xin chào"),
    ("ta5m bie65t", "tạm biệt"),
    ("ca3m o7n", "cảm ơn"),
];

// ============================================================
// PROVERBS (TỤC NGỮ)
// ============================================================

const TELEX_PROVERBS: &[(&str, &str)] = &[
    ("hocj mootj bieets muwowif", "học một biết mười"),
    (
        "ddi mootj ngayf ddangf hocj mootj sangf khoon",
        "đi một ngày đàng học một sàng khôn",
    ),
    ("toots goox hown ddepj nguwowif", "tốt gỗ hơn đẹp người"),
    ("uoongs nuwowcs nhows nguoonf", "uống nước nhớ nguồn"),
    ("nuwowcs chayr ddas monf", "nước chảy đá mòn"),
];

const VNI_PROVERBS: &[(&str, &str)] = &[
    ("ho5c mo65t bie61t mu7o7i2", "học một biết mười"),
    ("uo61ng nu7o71c nho71 nguo62n", "uống nước nhớ nguồn"),
    ("to61t go64 ho7n d9e5p ngu7o7i2", "tốt gỗ hơn đẹp người"),
    ("nu7o71c cha3y d9a1 mo2n", "nước chảy đá mòn"),
];

// ============================================================
// IDIOMS (THÀNH NGỮ)
// ============================================================

const TELEX_IDIOMS: &[(&str, &str)] = &[
    ("an cuw lacj nghieepj", "an cư lạc nghiệp"),
    ("ddoongf taam hieepj luwcj", "đồng tâm hiệp lực"),
    ("thowif gian laf tieenf bacj", "thời gian là tiền bạc"),
];

// ============================================================
// DAILY CONVERSATIONS
// ============================================================

const TELEX_DAILY: &[(&str, &str)] = &[
    (
        "hoom nay thowif tieets thees naof",
        "hôm nay thời tiết thế nào",
    ),
    ("banj ddi ddaau vaayj", "bạn đi đâu vậy"),
    ("tooi ddang ddi lafm", "tôi đang đi làm"),
    ("mootj ly caf phee nhes", "một ly cà phê nhé"),
    ("bao nhieeu tieenf", "bao nhiêu tiền"),
];

const VNI_DAILY: &[(&str, &str)] = &[
    (
        "ho6m nay tho7i2 tie61t the61 na2o",
        "hôm nay thời tiết thế nào",
    ),
    ("ba5n d9i d9a6u va65y", "bạn đi đâu vậy"),
    ("bao nhie6u tie62n", "bao nhiêu tiền"),
];

// ============================================================
// FOOD
// ============================================================

const TELEX_FOOD: &[(&str, &str)] = &[
    ("cho tooi xem thuwcj ddown", "cho tôi xem thực đơn"),
    (
        "tooi muoons goij mootj phaanf phowr",
        "tôi muốn gọi một phần phở",
    ),
    ("ddoof awn raats ngon", "đồ ăn rất ngon"),
    ("tinhs tieenf nhes", "tính tiền nhé"),
];

// ============================================================
// EXPRESSIONS
// ============================================================

const TELEX_EXPRESSIONS: &[(&str, &str)] = &[
    ("khoong sao", "không sao"),
    ("dduwowcj roofif", "được rồi"),
    ("binhf thuwowngf", "bình thường"),
    ("sao cungx dduwowcj", "sao cũng được"),
    ("tuyeetj vowif", "tuyệt vời"),
    ("ddepj quas", "đẹp quá"),
];

// ============================================================
// POETRY (TRUYỆN KIỀU)
// ============================================================

const TELEX_POETRY: &[(&str, &str)] = &[
    (
        "trawm nawm trong coix nguwowif ta",
        "trăm năm trong cõi người ta",
    ),
    (
        "chuwx taif chuwx meenhj kheos laf ghets nhau",
        "chữ tài chữ mệnh khéo là ghét nhau",
    ),
];

// ============================================================
// LONG SENTENCES
// ============================================================

const TELEX_LONG: &[(&str, &str)] = &[
    (
        "vieetj nam laf mootj quoocs gia nawmf owr ddoong nam as",
        "việt nam là một quốc gia nằm ở đông nam á",
    ),
    (
        "thur ddoo cura vieetj nam laf thanhf phoos haf nooij",
        "thủ đô của việt nam là thành phố hà nội",
    ),
];

const VNI_LONG: &[(&str, &str)] = &[
    (
        "vie65t nam la2 mo65t quo61c gia na82m o73 d9o6ng nam a1",
        "việt nam là một quốc gia nằm ở đông nam á",
    ),
    (
        "thu3 d9o6 cu3a vie65t nam la2 tha2nh pho61 ha2 no65i",
        "thủ đô của việt nam là thành phố hà nội",
    ),
];

// ============================================================
// MIXED CASE SENTENCES
// ============================================================

const TELEX_MIXED_CASE: &[(&str, &str)] = &[
    ("Xin chaof", "Xin chào"),
    ("Vieetj Nam", "Việt Nam"),
    ("VIEETJ NAM", "VIỆT NAM"),
    ("Thanhf phoos Hoof Chis Minh", "Thành phố Hồ Chí Minh"),
];

const VNI_MIXED_CASE: &[(&str, &str)] = &[
    ("Xin cha2o", "Xin chào"),
    ("Vie65t Nam", "Việt Nam"),
    ("Tha2nh pho61 Ho62 Chi1 Minh", "Thành phố Hồ Chí Minh"),
];

// ============================================================
// COMMON ISSUES - Real bugs found in production
// ============================================================

const TELEX_COMMON_ISSUES: &[(&str, &str)] = &[
    // Issue 2.1: Dính chữ (aa -> aâ instead of â)
    ("aa", "â"),
    ("ee", "ê"),
    ("oo", "ô"),
    ("dd", "đ"),
    ("DD", "Đ"),
    // Issue 2.4: Lặp chữ (được -> đđược)
    ("dduwowcj", "được"),
    // Issue #14: Alternative typing with wo → ươ compound
    ("ddwocj", "được"),
    // Issue #29: uw + o should form ươ compound (u with horn + o → ươ)
    ("dduwocj", "được"),
    ("nuwocs", "nước"),
    ("suwongs", "sướng"),
    ("truwongf", "trường"),
    ("nguwoif", "người"),
    ("nwocj", "nược"),
    ("swongs", "sướng"),
    ("bwomf", "bườm"),
    ("twoir", "tưởi"),
    ("ddif", "đì"),
    ("ddi", "đi"),
    ("ddang", "đang"),
    ("ddaauf", "đầu"),
    // Issue 2.4: Mất dấu (trường -> trương)
    ("truwowngf", "trường"),
    ("dduwowngf", "đường"),
    ("nguwowif", "người"),
    ("muwowif", "mười"),
    // Letter vs modifier ambiguity
    ("sa", "sa"),
    ("as", "á"),
    ("sas", "sá"),
    ("sass", "sas"),
    ("fa", "fa"),
    ("af", "à"),
    // Long compound words
    ("nghieeng", "nghiêng"),
    ("khuyeens", "khuyến"),
    ("nguoongf", "nguồng"),
    // ươu triphthong (hươu = deer)
    ("huouw", "hươu"),
    ("ruwowuj", "rượu"),
];

const VNI_COMMON_ISSUES: &[(&str, &str)] = &[
    // Not sticky
    ("a6", "â"),
    ("e6", "ê"),
    ("o6", "ô"),
    ("d9", "đ"),
    ("D9", "Đ"),
    // No double đ
    ("d9u7o7c5", "được"),
    ("d9i", "đi"),
    ("d9ang", "đang"),
    // Preserve tone mark
    ("tru7o7ng2", "trường"),
    ("d9u7o7ng2", "đường"),
    ("ngu7o7i2", "người"),
    // Real words with ươ
    ("nu7o7c1", "nước"),
    ("bu7o7m1", "bướm"),
    ("su7o7ng1", "sướng"),
    ("lu7o7ng2", "lường"),
    ("thu7o7ng2", "thường"),
    ("hu7o7ng1", "hướng"),
    ("vu7o7n2", "vườn"),
    // Real words with ua vs qua
    ("mua2", "mùa"),
    ("chua1", "chúa"),
    ("rua2", "rùa"),
    ("lua1", "lúa"),
    ("su7a4", "sữa"),
    ("qua1", "quá"),
    ("qua3", "quả"),
    ("qua2", "quà"),
    // Real words with iê
    ("vie65t", "việt"),
    ("tie61ng", "tiếng"),
    ("bie63n", "biển"),
    ("mie61ng", "miếng"),
    ("die64n", "diễn"),
    ("kie63m", "kiểm"),
    ("tie62n", "tiền"),
    ("hie63u", "hiểu"),
    // Mixed common words
    ("co1", "có"),
    ("kho6ng", "không"),
    ("la2", "là"),
    ("d9i", "đi"),
    ("ve62", "về"),
    ("a8n", "ăn"),
    ("o6ng1", "ống"),
    ("ba2n", "bàn"),
    ("nha2", "nhà"),
    ("hoc5", "học"),
    // ươu triphthong (hươu = deer)
    ("huou7", "hươu"),
    ("ruo7u5", "rượu"),
];

// ============================================================
// MARK REPOSITIONING - Complex diacritic interactions
// ============================================================

const VNI_MARK_REPOSITION: &[(&str, &str)] = &[
    // ua patterns
    ("ua27", "ừa"),
    ("ua2", "ùa"),
    ("ua7", "ưa"),
    // oa patterns
    ("oa26", "oầ"),
    ("o6a2", "ồa"),
    ("oa2", "oà"),
    // uo compound with marks
    ("uo71", "ướ"),
    ("uo72", "ườ"),
    ("uo73", "ưở"),
    ("uo74", "ưỡ"),
    ("uo75", "ượ"),
    ("uo17", "ướ"),
    ("uo27", "ườ"),
    ("u7o71", "ướ"),
    ("u7o72", "ườ"),
    // ua vs qua
    ("ua1", "úa"),
    ("ua2", "ùa"),
    ("qua1", "quá"),
    ("qua2", "quà"),
    ("u7a1", "ứa"),
    ("u7a2", "ừa"),
    ("ua17", "ứa"),
    ("ua27", "ừa"),
];

const TELEX_MARK_REPOSITION: &[(&str, &str)] = &[
    ("uafw", "uằ"),
    ("uwaf", "ừa"),
    ("oafw", "oằ"),
    // ươ compound
    ("uwows", "ướ"),
    ("uwowf", "ườ"),
    ("uwowr", "ưở"),
    ("uwowx", "ưỡ"),
    ("uwowj", "ượ"),
    ("uows", "ướ"),
    ("uowf", "ườ"),
    // Real words
    ("nuwowcs", "nước"),
    ("buwowms", "bướm"),
    ("suwowngs", "sướng"),
    ("luwowngf", "lường"),
    ("dduwowngf", "đường"),
    ("truwowngf", "trường"),
    ("thuwowngf", "thường"),
    ("huwowngs", "hướng"),
    ("vuwownf", "vườn"),
    // ua vs qua
    ("muaf", "mùa"),
    ("chuas", "chúa"),
    ("chuwa", "chưa"), // horn on u, not breve on a
    ("thuwa", "thưa"), // horn on u
    ("muwa", "mưa"),   // rain - horn on u
    ("ruaf", "rùa"),
    ("luas", "lúa"),
    ("suwax", "sữa"),
    ("quas", "quá"),
    ("quar", "quả"),
    ("quaf", "quà"),
    // iê words
    ("vieetj", "việt"),
    ("tieengs", "tiếng"),
    ("bieenr", "biển"),
    ("mieengs", "miếng"),
    ("dieenx", "diễn"),
    ("kieemr", "kiểm"),
    ("tieenf", "tiền"),
    ("hieeur", "hiểu"),
];

// ============================================================
// DELAYED INPUT PATTERNS
// ============================================================

const TELEX_DELAYED_PATTERNS: &[(&str, &str)] = &[
    ("tungw", "tưng"),
    ("tongw", "tơng"),
    ("tangw", "tăng"),
    ("tuow", "tươ"),
    ("nguoiw", "ngươi"),
    // ua + w -> ưa (horn on u, not breve on a)
    ("chuaw", "chưa"),
    ("thuaw", "thưa"),
    ("muaw", "mưa"),
];

const VNI_DELAYED_PATTERNS: &[(&str, &str)] = &[
    ("tung7", "tưng"),
    ("tong7", "tơng"),
    ("tang8", "tăng"),
    ("dung9", "đung"),
    ("Dung9", "Đung"),
];

// ============================================================
// DELAYED TONE MARKS - Typing tone after completing the word
// ============================================================

const TELEX_DELAYED_TONE: &[(&str, &str)] = &[
    // Single vowel words - tone at end
    ("bas", "bá"),
    ("caf", "cà"),
    ("mar", "mả"),
    ("lax", "lã"),
    ("taj", "tạ"),
    // oa/oe patterns - tone on second vowel
    ("hoaf", "hoà"),
    ("loas", "loá"),
    ("hoej", "hoẹ"),
    // ai/ao/au patterns - tone on first vowel
    ("mais", "mái"),
    ("laof", "lào"),
    ("daur", "dảu"),
    ("tais", "tái"),
    ("caof", "cào"),
    ("baur", "bảu"),
    // Common single vowel words with delayed tone
    ("lams", "lám"),
    ("lamf", "làm"),
    ("cons", "cón"),
    ("bonf", "bòn"),
    // Words with final consonant - tone on vowel before final
    ("bangs", "báng"),
    ("dangf", "dàng"),
    ("mangr", "mảng"),
    ("tangx", "tãng"),
    ("sangj", "sạng"),
    // Words with circumflex (ô/ê/â) - need double vowel first
    ("khoongf", "khồng"),
    ("hoongf", "hồng"),
    ("coongs", "cống"),
    ("toongs", "tống"),
    // Words with horn (ư/ơ) - uw/ow
    ("tuwf", "từ"),
    ("cuwf", "cừ"),
    ("tows", "tớ"),
    ("howf", "hờ"),
];

const VNI_DELAYED_TONE: &[(&str, &str)] = &[
    // Single vowel words - tone at end
    ("ba1", "bá"),
    ("ca2", "cà"),
    ("ma3", "mả"),
    ("la4", "lã"),
    ("ta5", "tạ"),
    // oa/oe patterns - tone on second vowel
    ("hoa2", "hoà"),
    ("loa1", "loá"),
    ("hoe5", "hoẹ"),
    // ai/ao/au patterns - tone on first vowel
    ("mai1", "mái"),
    ("lao2", "lào"),
    ("dau3", "dảu"),
    ("tai1", "tái"),
    ("cao2", "cào"),
    ("bau3", "bảu"),
    // Common single vowel words with delayed tone
    ("lam1", "lám"),
    ("lam2", "làm"),
    ("con1", "cón"),
    ("bon2", "bòn"),
    // Words with final consonant - tone on vowel before final
    ("bang1", "báng"),
    ("dang2", "dàng"),
    ("mang3", "mảng"),
    ("tang4", "tãng"),
    ("sang5", "sạng"),
    // Words with circumflex (ô/ê/â) - need mark first (6)
    ("khong62", "khồng"),
    ("hong62", "hồng"),
    ("cong61", "cống"),
    ("tong61", "tống"),
    // Words with horn (ư/ơ) - need mark (7)
    ("tu72", "từ"),
    ("cu72", "cừ"),
    ("to71", "tớ"),
    ("ho72", "hờ"),
    // Words with ươ - delayed marks then tone (duong + 9 + 7 + 2)
    ("duong972", "đường"),
    ("truong72", "trường"),
    ("nuoc71", "nước"),
    ("nguoi72", "người"),
    // Words with đ - delayed (d + 9)
    ("di9", "đi"),
    ("do91", "đó"),
    ("dang92", "đàng"),
];

// ============================================================
// SWITCHING DIACRITICS - circumflex ↔ horn
// ============================================================

const VNI_SWITCH_DIACRITICS: &[(&str, &str)] = &[
    // Single vowel: switch circumflex ↔ horn (last modifier wins)
    ("o67", "ơ"),  // ô + 7 → ơ (switch to horn)
    ("o76", "ô"),  // ơ + 6 → ô (switch to circumflex)
    ("o676", "ô"), // multiple switches, last wins
    ("o767", "ơ"), // multiple switches, last wins
    // Compound vowel: switch between ươ and uô
    ("uo76", "uô"), // ươ + 6 → uô (switch to circumflex)
    ("uo67", "ươ"), // uô + 7 → ươ (switch to horn)
    // Real words: last modifier wins
    ("buong76", "buông"),  // buơng + 6 → buông (last is circumflex)
    ("buong67", "buơng"),  // buông + 7 → buơng (last is horn)
    ("buong767", "buơng"), // can switch multiple times
    ("buong676", "buông"), // can switch multiple times
];

const TELEX_SWITCH_DIACRITICS: &[(&str, &str)] = &[
    // Single vowel: switch circumflex ↔ horn (last modifier wins)
    ("oow", "ơ"),  // ô + w → ơ (switch to horn)
    ("owo", "ô"),  // ơ + o → ô (switch to circumflex)
    ("oowo", "ô"), // multiple switches, last wins
    ("owow", "ơ"), // multiple switches, last wins
    // Compound vowel: switch between ươ and uô
    ("uowo", "uô"), // ươ + o → uô (switch to circumflex)
    ("uoow", "ươ"), // uô + w → ươ (switch to horn)
];

// ============================================================
// TEST FUNCTIONS
// ============================================================

#[test]
fn telex_backspace() {
    telex(TELEX_BACKSPACE);
}

#[test]
fn vni_backspace() {
    vni(VNI_BACKSPACE);
}

#[test]
fn telex_typos() {
    telex(TELEX_TYPOS);
}

#[test]
fn telex_rapid_typing() {
    telex(TELEX_RAPID);
}

#[test]
fn vni_rapid_typing() {
    vni(VNI_RAPID);
}

#[test]
fn telex_capitalization() {
    telex(TELEX_CAPS);
}

#[test]
fn vni_capitalization() {
    vni(VNI_CAPS);
}

#[test]
fn telex_greetings() {
    telex(TELEX_GREETINGS);
}

#[test]
fn vni_greetings() {
    vni(VNI_GREETINGS);
}

#[test]
fn telex_proverbs() {
    telex(TELEX_PROVERBS);
}

#[test]
fn vni_proverbs() {
    vni(VNI_PROVERBS);
}

#[test]
fn telex_idioms() {
    telex(TELEX_IDIOMS);
}

#[test]
fn telex_daily_conversations() {
    telex(TELEX_DAILY);
}

#[test]
fn vni_daily_conversations() {
    vni(VNI_DAILY);
}

#[test]
fn telex_food() {
    telex(TELEX_FOOD);
}

#[test]
fn telex_expressions() {
    telex(TELEX_EXPRESSIONS);
}

#[test]
fn telex_poetry() {
    telex(TELEX_POETRY);
}

#[test]
fn telex_long_sentences() {
    telex(TELEX_LONG);
}

#[test]
fn vni_long_sentences() {
    vni(VNI_LONG);
}

#[test]
fn telex_mixed_case() {
    telex(TELEX_MIXED_CASE);
}

#[test]
fn vni_mixed_case() {
    vni(VNI_MIXED_CASE);
}

#[test]
fn telex_common_issues() {
    telex(TELEX_COMMON_ISSUES);
}

#[test]
fn vni_common_issues() {
    vni(VNI_COMMON_ISSUES);
}

#[test]
fn vni_mark_repositioning() {
    vni(VNI_MARK_REPOSITION);
}

#[test]
fn telex_mark_repositioning() {
    telex(TELEX_MARK_REPOSITION);
}

#[test]
fn telex_delayed_patterns() {
    telex(TELEX_DELAYED_PATTERNS);
}

#[test]
fn vni_delayed_patterns() {
    vni(VNI_DELAYED_PATTERNS);
}

#[test]
fn telex_delayed_tone() {
    telex(TELEX_DELAYED_TONE);
}

#[test]
fn vni_delayed_tone() {
    vni(VNI_DELAYED_TONE);
}

#[test]
fn telex_switch_diacritics() {
    telex(TELEX_SWITCH_DIACRITICS);
}

#[test]
fn vni_switch_diacritics() {
    vni(VNI_SWITCH_DIACRITICS);
}
